//! Integration seam over divvun-runtime and the hfst generator. The legacy
//! system shelled out to `preprocess`, xfst `lookup`, `lookup2cg` and
//! `vislcg3` with shared temp files; every one of those process invocations
//! maps to a method here so the linguistic contract stays in one place and
//! the pipeline logic never spawns processes. Wire format out of these
//! methods matches what the shell tools produced, so callers translated
//! from the legacy code parse the same streams.
//!
//! Two model sources back the seam:
//! - `TEAKSTA_BUNDLE`: a divvun-runtime `.drb` exposing the named pipelines
//!   `tokenize`, `analyze` (tokeniser → mwe-dis → disambiguator → the
//!   konteaksta syntax grammar, emitting a raw CG stream) and `sentences`
//!   (sentence surface texts).
//! - `TEAKSTA_GENERATOR`: a normative generator `.hfstol` (generator-gt-norm)
//!   used in-process for word-form generation, replacing the inverted-FST
//!   `lookup` invocation.
//!
//! Every method blocks on an internally owned single-thread tokio runtime;
//! callers on an async executor must reach this seam through a blocking
//! section (e.g. `spawn_blocking`), never directly from a worker thread.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use anyhow::{Context as _, Result, anyhow, bail};
use divvun_runtime::ast::PipelineHandle;
use divvun_runtime::bundle::Bundle;
use divvun_runtime::modules::PipelineValue;
use futures_util::StreamExt;
use hfst::hfst_flag_diacritics::FdOperation;
use hfst::hfst_input_stream::HfstInputStream;
use hfst::hfst_transducer::AnyTransducer;
use hfst::transducer::IStream;

/// Environment variable naming the `.drb` bundle backing the pipelines.
pub const BUNDLE_ENV: &str = "TEAKSTA_BUNDLE";
/// Environment variable naming the normative generator `.hfstol`.
pub const GENERATOR_ENV: &str = "TEAKSTA_GENERATOR";

/// Why the normative generator could not be used. Every variant describes a
/// fault in the transducer or its configuration, never an input the
/// generator simply has no form for: an unknown form is an empty result,
/// not an error, which is what keeps the `input+?` echo meaning "no such
/// word form" and nothing else.
#[derive(Debug, thiserror::Error)]
pub enum GeneratorError {
    #[error("{GENERATOR_ENV} is not set; point it at generator-gt-norm.hfstol")]
    Unset,
    #[error("reading generator transducer {path}: {message}")]
    Read { path: String, message: String },
    #[error("opening generator transducer {path}: {message}")]
    Open { path: String, message: String },
    #[error("generator {path} is not an optimized-lookup transducer")]
    NotOptimizedLookup { path: String },
    #[error("generator lookup failed for {input:?}: {message}")]
    Lookup { input: String, message: String },
}

/// One pipeline's execution slot: `None` until its handle has been built,
/// and emptied again when a run panics (see [`lock_slot`]). Slots are handed
/// out of the registry by clone, so running a pipeline holds no registry
/// lock — `PipelineHandle` is neither cloneable nor safe to `forward`
/// through concurrently, so each one keeps its own lock instead.
type HandleSlot = Arc<Mutex<Option<PipelineHandle>>>;

pub struct MorphoPipeline {
    runtime: tokio::runtime::Runtime,
    handles: Mutex<HashMap<&'static str, HandleSlot>>,
    generator: OnceLock<Mutex<AnyTransducer>>,
}

static SHARED: OnceLock<MorphoPipeline> = OnceLock::new();

impl MorphoPipeline {
    /// Process-wide pipeline instance. The legacy code kept one singleton
    /// analysis engine per (language, topic); the Rust side shares one
    /// bundle-backed pipeline set and one generator transducer.
    pub fn shared() -> &'static MorphoPipeline {
        SHARED.get_or_init(|| MorphoPipeline {
            runtime: tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("tokio runtime for the morpho seam"),
            handles: Mutex::new(HashMap::new()),
            generator: OnceLock::new(),
        })
    }

    /// Runs the named bundle pipeline over `input` and returns every value
    /// it streams (batch producers like sentence splitting emit one value
    /// per element). Handles are created lazily per pipeline name and
    /// reused; the bundle is reopened per handle, which keeps creation
    /// simple at the cost of a slower first call per pipeline.
    fn run(&self, pipeline: &'static str, input: String) -> Result<Vec<PipelineValue>> {
        let bundle_path = std::env::var(BUNDLE_ENV)
            .map_err(|_| anyhow!("{BUNDLE_ENV} is not set; point it at the sme .drb bundle"))?;
        let slot = self.slot(pipeline);
        // The registry lock is already released here: creating and running a
        // pipeline reaches into divvun-runtime, cg3 and hfst, and a panic
        // down there must cost this pipeline alone rather than poisoning the
        // registry every other pipeline is looked up through.
        let mut slot = lock_slot(&slot);
        self.runtime.block_on(async {
            if slot.is_none() {
                let bundle = Bundle::from_bundle_named(&bundle_path, pipeline)
                    .await
                    .with_context(|| {
                        format!("loading pipeline {pipeline:?} from bundle {bundle_path:?}")
                    })?;
                let handle = bundle
                    .create(serde_json::json!({}))
                    .await
                    .with_context(|| format!("creating pipeline {pipeline:?}"))?;
                *slot = Some(handle);
            }
            let handle = slot.as_mut().expect("pipeline handle created above");
            let mut stream = handle.forward(PipelineValue::String(input)).await;
            let mut values = Vec::new();
            while let Some(item) = stream.next().await {
                values.push(item.map_err(|e| anyhow!("pipeline {pipeline:?} failed: {e}"))?);
            }
            if values.is_empty() {
                bail!("pipeline {pipeline:?} produced no output");
            }
            Ok(values)
        })
    }

    /// The execution slot for a pipeline, created empty on first mention.
    /// The registry lock covers this lookup and nothing else; a poisoned
    /// registry is recovered rather than treated as fatal, since the map
    /// holds names and slot handles that no pipeline run can tear.
    fn slot(&self, pipeline: &'static str) -> HandleSlot {
        let mut registry = self
            .handles
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Arc::clone(registry.entry(pipeline).or_default())
    }

    /// Runs a pipeline whose result is a single value and renders it as a
    /// string.
    fn run_to_string(&self, pipeline: &'static str, input: String) -> Result<String> {
        let mut values = self.run(pipeline, input)?;
        if values.len() != 1 {
            bail!(
                "pipeline {pipeline:?} returned {} values where one was expected",
                values.len()
            );
        }
        match values.remove(0) {
            PipelineValue::String(s) => Ok(s),
            PipelineValue::Json(j) => Ok(serde_json::to_string(&j)?),
            other => bail!("pipeline {pipeline:?} returned an unsupported value kind: {other:?}"),
        }
    }

    /// North Sámi tokenisation. Replaces `cat <file> | preprocess
    /// --abbr=abbr.txt`: takes raw text, returns one surface token per
    /// element exactly as the preprocess output had one token per line.
    /// The bundle's `tokenize` pipeline emits a CG cohort stream; the
    /// surface forms are lifted from the `"<form>"` cohort lines.
    pub fn tokenize(&self, text: &str) -> Result<Vec<String>> {
        let out = self.run_to_string("tokenize", text.to_string())?;
        Ok(out
            .lines()
            .filter_map(|line| {
                line.strip_prefix("\"<")
                    .and_then(|rest| rest.strip_suffix(">\""))
                    .map(str::to_string)
            })
            .collect())
    }

    /// Morphological analysis + CG3 disambiguation + the konteaksta syntax
    /// grammar. Replaces `cat <tokens> | lookup <analyser.xfst> | lookup2cg
    /// | vislcg3 -g disambiguator.cg3 | vislcg3 -g konteaksta.cg3`. Input is
    /// one token per element; output is the raw CG stream (`"<token>"`
    /// cohort lines followed by tab-indented reading lines). The bundle's
    /// `analyze` pipeline tokenises internally, so the tokens are re-joined
    /// with newlines; the pmhfst tokeniser treats each line as one unit,
    /// matching the legacy one-token-per-line input contract.
    pub fn analyze_disambiguate(&self, tokens: &[String]) -> Result<String> {
        self.run_to_string("analyze", tokens.join("\n"))
    }

    /// Word-form generation through the normative generator. Replaces
    /// `cat <input> | lookup <generator FST>`: input is lookup wire format
    /// (one `lemma+Tag+Tag` per line), output is the raw lookup output
    /// stream — for every non-empty input line, one `input<TAB>surface`
    /// line per generated form (or `input<TAB>input+?` when generation
    /// fails), then a blank line closing the block. Non-lexical marker
    /// lines (the `ñôŃßĘńŠē` sentinel, `Word <begin> <end>` records) fail
    /// generation and are therefore echoed in their failure line, which is
    /// exactly the echo the legacy readers key on. A generator that cannot
    /// be loaded or read is an error, never that echo: the `+?` line states
    /// that the generator knows no form for the input, and a broken
    /// generator knows nothing about any input.
    pub fn generate(&self, input: &str) -> Result<String> {
        let transducer = self.generator()?;
        let mut out = String::new();
        for line in input.lines() {
            if line.is_empty() {
                out.push('\n');
                continue;
            }
            let surfaces = lookup_surfaces(transducer, line)?;
            if surfaces.is_empty() {
                out.push_str(line);
                out.push('\t');
                out.push_str(line);
                out.push_str("+?\n");
            } else {
                for surface in surfaces {
                    out.push_str(line);
                    out.push('\t');
                    out.push_str(&surface);
                    out.push('\n');
                }
            }
            out.push('\n');
        }
        Ok(out)
    }

    /// Sentence segmentation over raw text, returning byte spans. Replaces
    /// the OpenNLP English sentence detector the legacy pipeline ran over
    /// North Sámi text. The bundle's `sentences` pipeline returns sentence
    /// surface texts; each is mapped back to a byte span by locating its
    /// words sequentially in the input.
    pub fn sentence_spans(&self, text: &str) -> Result<Vec<(usize, usize)>> {
        let values = self.run("sentences", text.to_string())?;
        let mut sentences: Vec<String> = Vec::new();
        for value in values {
            match value {
                PipelineValue::String(s) => sentences.push(s),
                PipelineValue::Json(j) => sentences.extend(
                    serde_json::from_value::<Vec<String>>(j)
                        .context("parsing sentence list from pipeline JSON output")?,
                ),
                other => {
                    bail!("pipeline \"sentences\" returned an unsupported value kind: {other:?}")
                }
            }
        }
        let mut spans = Vec::new();
        let mut cursor = 0usize;
        for sentence in &sentences {
            let words: Vec<&str> = sentence.split_whitespace().collect();
            if words.is_empty() {
                continue;
            }
            let Some(begin) = find_from(text, cursor, words[0]) else {
                continue;
            };
            let mut end = begin + words[0].len();
            for word in &words[1..] {
                if let Some(pos) = find_from(text, end, word) {
                    end = pos + word.len();
                }
            }
            spans.push((begin, end));
            cursor = end;
        }
        Ok(spans)
    }

    /// The generator transducer, read from `TEAKSTA_GENERATOR` on first use.
    /// Only a load that succeeded is remembered: a variable that is unset,
    /// or names a file that cannot be read, fails this call alone, so an
    /// operator who repairs the environment gets generation back on the next
    /// request instead of after a restart. Two callers racing the first load
    /// may both read the file; the loser's transducer is dropped.
    fn generator(&self) -> Result<&Mutex<AnyTransducer>, GeneratorError> {
        if let Some(transducer) = self.generator.get() {
            return Ok(transducer);
        }
        let _ = self.generator.set(Mutex::new(load_generator()?));
        Ok(self.generator.get().expect("generator stored above"))
    }
}

/// Locks a pipeline's slot, recovering from poisoning. The handle a panicked
/// run left behind is dropped rather than reused: its stream was abandoned
/// mid-flight, and output the abandoned run never consumed would otherwise
/// surface as the next caller's result. The next run pays one bundle load to
/// rebuild it. Clearing the poison keeps that cost to the one run that
/// followed the panic.
fn lock_slot(slot: &Mutex<Option<PipelineHandle>>) -> MutexGuard<'_, Option<PipelineHandle>> {
    match slot.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            slot.clear_poison();
            let mut guard = poisoned.into_inner();
            *guard = None;
            guard
        }
    }
}

/// Reads the normative generator named by `TEAKSTA_GENERATOR`.
fn load_generator() -> Result<AnyTransducer, GeneratorError> {
    let path = std::env::var(GENERATOR_ENV).map_err(|_| GeneratorError::Unset)?;
    let bytes = std::fs::read(&path).map_err(|e| GeneratorError::Read {
        path: path.clone(),
        message: e.to_string(),
    })?;
    let input = IStream::new_owned(std::io::Cursor::new(bytes));
    let mut stream = HfstInputStream::new_istream(input).map_err(|e| GeneratorError::Open {
        path: path.clone(),
        message: e.to_string(),
    })?;
    let transducer = stream.read().map_err(|e| GeneratorError::Read {
        path: path.clone(),
        message: e.to_string(),
    })?;
    match &transducer {
        AnyTransducer::OlW(_) | AnyTransducer::OlU(_) => Ok(transducer),
        _ => Err(GeneratorError::NotOptimizedLookup { path }),
    }
}

/// Flag-diacritic-aware lookup returning the surface string of each result
/// path (non-diacritic symbols only). No result means the generator has no
/// form for `input`; every other outcome is an error, so a caller cannot
/// read a broken transducer as a word the generator does not know.
fn lookup_surfaces(
    transducer: &Mutex<AnyTransducer>,
    input: &str,
) -> Result<Vec<String>, GeneratorError> {
    // The transducer is only ever read through here, so a panic that
    // poisoned it cannot have left it half-written.
    let mut guard = transducer
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let paths = match &mut *guard {
        AnyTransducer::OlW(t) => t.lookup_fd_string(input, -1, 10.0),
        AnyTransducer::OlU(t) => t.lookup_fd_string(input, -1, 10.0),
        // `load_generator` stores no other kind, so this arm only fires if
        // the two ever disagree about what an optimized-lookup transducer is.
        _ => {
            return Err(GeneratorError::Lookup {
                input: input.to_string(),
                message: "generator is not an optimized-lookup transducer".to_string(),
            });
        }
    };
    let paths = paths.map_err(|e| GeneratorError::Lookup {
        input: input.to_string(),
        message: e.to_string(),
    })?;
    Ok(paths
        .into_iter()
        .map(|path| {
            path.second
                .iter()
                .filter(|sym| !FdOperation::is_diacritic(sym.as_str()))
                .map(|sym| sym.as_str())
                .collect::<String>()
        })
        .collect())
}

fn find_from(text: &str, cursor: usize, needle: &str) -> Option<usize> {
    text.get(cursor..)
        .and_then(|rest| rest.find(needle))
        .map(|pos| cursor + pos)
}
