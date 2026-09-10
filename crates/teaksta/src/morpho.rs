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
use std::sync::{Mutex, OnceLock};

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

pub struct MorphoPipeline {
    runtime: tokio::runtime::Runtime,
    handles: Mutex<HashMap<&'static str, PipelineHandle>>,
    generator: OnceLock<Result<Mutex<AnyTransducer>, String>>,
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
        let mut handles = self
            .handles
            .lock()
            .map_err(|_| anyhow!("morpho pipeline lock poisoned"))?;
        self.runtime.block_on(async {
            if !handles.contains_key(pipeline) {
                let bundle = Bundle::from_bundle_named(&bundle_path, pipeline)
                    .await
                    .with_context(|| {
                        format!("loading pipeline {pipeline:?} from bundle {bundle_path:?}")
                    })?;
                let handle = bundle
                    .create(serde_json::json!({}))
                    .await
                    .with_context(|| format!("creating pipeline {pipeline:?}"))?;
                handles.insert(pipeline, handle);
            }
            let handle = handles
                .get_mut(pipeline)
                .expect("pipeline handle inserted above");
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
    /// exactly the echo the legacy readers key on.
    pub fn generate(&self, input: &str) -> Result<String> {
        let transducer = self.generator()?;
        let mut out = String::new();
        for line in input.lines() {
            if line.is_empty() {
                out.push('\n');
                continue;
            }
            let surfaces = lookup_surfaces(transducer, line);
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

    fn generator(&self) -> Result<&Mutex<AnyTransducer>> {
        let loaded = self.generator.get_or_init(|| {
            let path = std::env::var(GENERATOR_ENV).map_err(|_| {
                format!("{GENERATOR_ENV} is not set; point it at generator-gt-norm.hfstol")
            })?;
            let bytes = std::fs::read(&path)
                .map_err(|e| format!("reading generator transducer {path}: {e}"))?;
            let input = IStream::new_owned(std::io::Cursor::new(bytes));
            let mut stream = HfstInputStream::new_istream(input)
                .map_err(|e| format!("opening generator transducer {path}: {e}"))?;
            let transducer = stream
                .read()
                .map_err(|e| format!("reading generator transducer {path}: {e}"))?;
            match &transducer {
                AnyTransducer::OlW(_) | AnyTransducer::OlU(_) => {}
                _ => {
                    return Err(format!(
                        "generator {path} is not an optimized-lookup transducer"
                    ));
                }
            }
            Ok(Mutex::new(transducer))
        });
        match loaded {
            Ok(t) => Ok(t),
            Err(e) => Err(anyhow!("{e}")),
        }
    }
}

/// Flag-diacritic-aware lookup returning the surface string of each result
/// path (non-diacritic symbols only).
fn lookup_surfaces(transducer: &Mutex<AnyTransducer>, input: &str) -> Vec<String> {
    let mut guard = match transducer.lock() {
        Ok(guard) => guard,
        Err(_) => return Vec::new(),
    };
    let paths = match &mut *guard {
        AnyTransducer::OlW(t) => t.lookup_fd_string(input, -1, 10.0),
        AnyTransducer::OlU(t) => t.lookup_fd_string(input, -1, 10.0),
        _ => return Vec::new(),
    };
    let Ok(paths) = paths else {
        return Vec::new();
    };
    paths
        .into_iter()
        .map(|path| {
            path.second
                .iter()
                .filter(|sym| !FdOperation::is_diacritic(sym.as_str()))
                .map(|sym| sym.as_str())
                .collect::<String>()
        })
        .collect()
}

fn find_from(text: &str, cursor: usize, needle: &str) -> Option<usize> {
    text.get(cursor..)
        .and_then(|rest| rest.find(needle))
        .map(|pos| cursor + pos)
}
