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
//!
//! Input reaches a pipeline in ordered chunks rather than as one document.
//! divvun-runtime wires each pipeline stage to the next through a 16-event
//! `tokio::sync::broadcast` channel, and a stage that fans one input out
//! into a batch — the sentence splitter emits one value per sentence — sends
//! that whole batch before the consumer on this seam's single-threaded
//! runtime is scheduled to read any of it. A document with more sentences
//! than the buffer holds therefore lost the overflow, and the run failed
//! with `channel lagged by N`. Chunking is confined to this module: every
//! method still takes a whole document and answers for the whole document,
//! with the per-chunk outputs concatenated in input order.

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

/// How many sentence boundaries one chunk of pipeline input carries. The
/// widest burst a stage can answer one chunk with is one value per sentence
/// in it, so this is what keeps a burst inside the 16-event channels the
/// stages are wired with — with room left over for the stages that answer
/// more than one value per sentence.
const CHUNK_BOUNDARY_BUDGET: usize = 6;

/// The byte ceiling a chunk is cut at when it reaches no sentence boundary
/// before this much input. It is generous because it is not what bounds a
/// burst: a single oversized value is forwarded and answered without
/// trouble, and only a burst of many values overflows a channel. Text is
/// therefore never cut at this ceiling mid-sentence — a pathological
/// sentence longer than it goes through whole rather than mid-word — while
/// the one-token-per-line stream, where every line is already a whole
/// token, is cut at the nearest line boundary.
const CHUNK_BYTE_BUDGET: usize = 8192;

/// The byte offsets raw text may be cut at: one past a newline, or one past
/// sentence-final punctuation that whitespace or the end of the text
/// follows. Each boundary then runs on past the whitespace trailing it, so
/// a chunk closes over the space between two sentences rather than opening
/// with it, and no boundary can fall inside a word. The end of the text is
/// not listed — the remainder after the last cut is the last chunk.
fn text_boundaries(text: &str) -> Vec<usize> {
    let mut boundaries = Vec::new();
    let mut chars = text.char_indices().peekable();

    while let Some((at, c)) = chars.next() {
        let after = at + c.len_utf8();
        let follows_sentence = matches!(c, '.' | '!' | '?')
            && chars.peek().is_none_or(|&(_, next)| next.is_whitespace());
        if c != '\n' && !follows_sentence {
            continue;
        }

        let mut end = after;
        while let Some(&(at, c)) = chars.peek() {
            if !c.is_whitespace() {
                break;
            }
            end = at + c.len_utf8();
            chars.next();
        }
        if end < text.len() {
            boundaries.push(end);
        }
    }

    boundaries
}

/// Raw text as ordered chunks whose concatenation is the input again. A
/// chunk is closed at the first boundary past either budget, and a chunk
/// carrying nothing but whitespace is never closed on its own — a masked
/// document is mostly spaces, and a chunk of them asks the pipeline a
/// question about nothing.
fn text_chunks(text: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut start = 0usize;
    let mut boundaries = 0usize;

    for end in text_boundaries(text) {
        boundaries += 1;
        let pending = &text[start..end];
        if boundaries < CHUNK_BOUNDARY_BUDGET && pending.len() < CHUNK_BYTE_BUDGET {
            continue;
        }
        if pending.trim().is_empty() {
            continue;
        }
        chunks.push(pending.to_string());
        start = end;
        boundaries = 0;
    }

    if start < text.len() || chunks.is_empty() {
        chunks.push(text[start..].to_string());
    }
    chunks
}

/// Whether a line of the one-token-per-line stream is nothing but
/// sentence-final punctuation — the stream's own sentence boundary, and the
/// token the vislcg3 stage injects to close a heading.
fn is_sentence_final_line(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty() && trimmed.chars().all(|c| matches!(c, '.' | '!' | '?'))
}

/// The one-token-per-line stream as ordered chunks, each rejoined with
/// newlines, together carrying every line in order. Cuts land after a
/// sentence-final line: constraint-grammar disambiguation windows are
/// sentence-local, so a seam there leaves every window whole. A stretch
/// that reaches the byte ceiling without one is cut at that line's end
/// instead, which is still never inside a token.
fn token_line_chunks(tokens: &[String]) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut start = 0usize;
    let mut enders = 0usize;
    let mut bytes = 0usize;

    for (at, token) in tokens.iter().enumerate() {
        bytes += token.len() + 1;
        let ender = is_sentence_final_line(token);
        if ender {
            enders += 1;
        }
        if !(ender && enders >= CHUNK_BOUNDARY_BUDGET) && bytes < CHUNK_BYTE_BUDGET {
            continue;
        }
        chunks.push(tokens[start..=at].join("\n"));
        start = at + 1;
        enders = 0;
        bytes = 0;
    }

    if start < tokens.len() || chunks.is_empty() {
        chunks.push(tokens[start..].join("\n"));
    }
    chunks
}

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

    /// Runs the named bundle pipeline over `chunks`, one `forward` per chunk
    /// in input order, and returns every value they streamed in that same
    /// order (batch producers like sentence splitting emit one value per
    /// element). The whole run holds the pipeline's slot for its length, so
    /// a request's chunks reach the handle as one uninterrupted sequence and
    /// no second caller's chunk lands between two of them. Handles are
    /// created lazily per pipeline name and reused; the bundle is reopened
    /// per handle, which keeps creation simple at the cost of a slower first
    /// call per pipeline.
    fn run(&self, pipeline: &'static str, chunks: Vec<String>) -> Result<Vec<PipelineValue>> {
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
            let mut values = Vec::new();
            for chunk in chunks {
                // Each chunk's stream is drained to its end before the next
                // chunk is forwarded. The handle carries one input channel
                // and one output channel, so output a forward left unread
                // would be delivered to whichever forward reads next.
                let mut stream = handle.forward(PipelineValue::String(chunk)).await;
                while let Some(item) = stream.next().await {
                    values.push(item.map_err(|e| anyhow!("pipeline {pipeline:?} failed: {e}"))?);
                }
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

    /// Runs a pipeline that answers one value per chunk and renders those
    /// values, in input order, as one string. The values are line streams,
    /// so a value that does not end its last line is given the terminator
    /// the next value's first line would otherwise be glued onto; one chunk
    /// is one value and nothing is inserted at all.
    fn run_to_string(&self, pipeline: &'static str, chunks: Vec<String>) -> Result<String> {
        let expected = chunks.len();
        let values = self.run(pipeline, chunks)?;
        if values.len() != expected {
            bail!(
                "pipeline {pipeline:?} returned {} values for {expected} chunk(s) of input, \
                 where one value per chunk was expected",
                values.len()
            );
        }
        let mut out = String::new();
        for value in values {
            let rendered = match value {
                PipelineValue::String(s) => s,
                PipelineValue::Json(j) => serde_json::to_string(&j)?,
                other => {
                    bail!("pipeline {pipeline:?} returned an unsupported value kind: {other:?}")
                }
            };
            if !out.is_empty() && !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(&rendered);
        }
        Ok(out)
    }

    /// North Sámi tokenisation. Replaces `cat <file> | preprocess
    /// --abbr=abbr.txt`: takes raw text, returns one surface token per
    /// element exactly as the preprocess output had one token per line.
    /// The bundle's `tokenize` pipeline emits a CG cohort stream; the
    /// surface forms are lifted from the `"<form>"` cohort lines. The text
    /// is fed in chunks cut at sentence boundaries, and the cohort streams
    /// they answer are read as the one stream their concatenation is.
    pub fn tokenize(&self, text: &str) -> Result<Vec<String>> {
        let out = self.run_to_string("tokenize", text_chunks(text))?;
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
    /// matching the legacy one-token-per-line input contract. The lines are
    /// fed in chunks cut after sentence-final ones, which is where a
    /// disambiguation window closes, and the answered cohort streams
    /// concatenate into the one stream the reader parses.
    pub fn analyze_disambiguate(&self, tokens: &[String]) -> Result<String> {
        self.run_to_string("analyze", token_line_chunks(tokens))
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
    /// words sequentially in the input. The text is fed in chunks cut at
    /// sentence boundaries, so the sentences come back in document order
    /// and the cursor walking them across the text only ever moves forward.
    pub fn sentence_spans(&self, text: &str) -> Result<Vec<(usize, usize)>> {
        let values = self.run("sentences", text_chunks(text))?;
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

#[cfg(test)]
mod tests {
    use super::*;

    /// `count` sentences of five words each, separated by single spaces.
    fn sentences(count: usize) -> String {
        (0..count)
            .map(|i| format!("Mun oidnen viesu ikte nummir {i}. "))
            .collect::<String>()
            .trim_end()
            .to_string()
    }

    fn lines(tokens: &[&str]) -> Vec<String> {
        tokens.iter().map(|t| t.to_string()).collect()
    }

    /// Nothing smaller than one budget is chunked at all, so the seam keeps
    /// asking the pipelines exactly the question it asked before chunking
    /// existed.
    #[test]
    fn a_small_text_is_one_chunk_of_itself() {
        for text in [
            "",
            " ",
            "Mun oidnen viesu ikte.",
            "Mun oidnen viesu ikte. Dat lei buorre.",
            "Mun oidnen viesu ikte.\nDat lei buorre.\n",
            "                ",
        ] {
            assert_eq!(text_chunks(text), vec![text.to_string()], "{text:?}");
        }
    }

    #[test]
    fn a_small_token_stream_is_one_chunk() {
        for tokens in [
            vec![],
            lines(&["viessu"]),
            lines(&[
                "Mun", "oidnen", "viesu", "ikte", ".", "Dat", "lei", "buorre", ".",
            ]),
        ] {
            assert_eq!(
                token_line_chunks(&tokens),
                vec![tokens.join("\n")],
                "{tokens:?}"
            );
        }
    }

    /// Chunking loses nothing and adds nothing: the chunks of a text are a
    /// partition of it, in order.
    #[test]
    fn text_chunks_concatenate_to_the_text() {
        for count in [1, 6, 7, 13, 60, 200] {
            let text = sentences(count);
            let chunks = text_chunks(&text);
            assert_eq!(chunks.concat(), text, "{count} sentences");
            assert!(chunks.iter().all(|chunk| !chunk.is_empty()));
        }
    }

    /// The same for the token stream, where a chunk is a run of whole lines
    /// and the chunks together carry every line once, in order.
    #[test]
    fn token_chunks_carry_every_line_in_order() {
        let tokens: Vec<String> = (0..200)
            .flat_map(|i| [format!("sátni{i}"), ".".to_string()])
            .collect();
        let chunks = token_line_chunks(&tokens);

        let carried: Vec<&str> = chunks.iter().flat_map(|chunk| chunk.lines()).collect();
        assert_eq!(
            carried,
            tokens.iter().map(String::as_str).collect::<Vec<_>>()
        );
        assert!(chunks.len() > 1, "{} chunks", chunks.len());
    }

    /// A chunk is closed at a sentence boundary, never in the middle of one,
    /// and carries no more sentences than the budget allows.
    #[test]
    fn a_text_chunk_ends_at_a_sentence_boundary() {
        let text = sentences(13);
        let chunks = text_chunks(&text);

        assert_eq!(chunks.len(), 3, "{chunks:#?}");
        for chunk in &chunks {
            assert!(chunk.trim_end().ends_with('.'), "{chunk:?}");
            assert!(
                chunk.matches('.').count() <= CHUNK_BOUNDARY_BUDGET,
                "{chunk:?}"
            );
        }
        // The cut takes the space after the full stop with it, so the next
        // chunk opens on the sentence rather than on the space.
        assert!(chunks[1].starts_with("Mun"), "{:?}", chunks[1]);
    }

    /// Paragraph breaks are boundaries too, and the newline closes the chunk
    /// it ends rather than opening the next one.
    #[test]
    fn a_paragraph_break_is_a_boundary() {
        let text = "a\nb\nc\nd\ne\nf\ng\n";
        let chunks = text_chunks(text);

        assert_eq!(
            chunks,
            vec!["a\nb\nc\nd\ne\nf\n".to_string(), "g\n".to_string()]
        );
        assert_eq!(chunks.concat(), text);
    }

    /// Sentence-final punctuation inside a word — a decimal point, an
    /// abbreviating full stop with no space after it — is not a boundary, so
    /// no cut can fall inside a token.
    #[test]
    fn punctuation_inside_a_word_is_no_boundary() {
        assert!(text_boundaries("3.5 ja 4.5").is_empty());
        assert!(text_boundaries("a.b.c.d.e.f.g.h").is_empty());
        assert_eq!(text_boundaries("Mun. Dat"), vec![5]);
    }

    /// One sentence longer than the byte ceiling goes through as a single
    /// oversized chunk rather than being cut inside a word: a lone large
    /// value is forwarded without trouble, and only a burst of many values
    /// overflows a channel.
    #[test]
    fn one_giant_sentence_is_never_cut_mid_word() {
        let giant = format!("{}.", "sátni ".repeat(4000));
        assert!(giant.len() > CHUNK_BYTE_BUDGET);

        assert_eq!(text_chunks(&giant), vec![giant.clone()]);

        // and it closes its chunk once it ends, rather than dragging the
        // rest of the document along with it.
        let text = format!("{giant} Dat lei buorre. Mun oidnen viesu.");
        let chunks = text_chunks(&text);
        assert_eq!(chunks.len(), 2, "{}", chunks.len());
        assert_eq!(chunks[0], format!("{giant} "));
        assert_eq!(chunks.concat(), text);
    }

    /// A masked document is mostly spaces, and a run of them is never a
    /// chunk of its own — it joins the text that follows it.
    #[test]
    fn a_whitespace_run_is_never_its_own_chunk() {
        let text = format!(
            "{}\nMun oidnen viesu ikte.",
            " ".repeat(CHUNK_BYTE_BUDGET * 2)
        );
        let chunks = text_chunks(&text);

        assert_eq!(chunks, vec![text.clone()]);
        assert!(chunks.iter().all(|chunk| !chunk.trim().is_empty()));
    }

    /// The token stream is cut after a line that is sentence-final
    /// punctuation, which is where a disambiguation window closes.
    #[test]
    fn a_token_chunk_ends_at_a_sentence_line() {
        let mut tokens = Vec::new();
        for _ in 0..8 {
            tokens.extend(lines(&["Mun", "oidnen", "viesu", "."]));
        }
        let chunks = token_line_chunks(&tokens);

        assert_eq!(chunks.len(), 2, "{chunks:#?}");
        assert!(chunks[0].ends_with("\n."), "{:?}", chunks[0]);
        assert_eq!(chunks[0].lines().filter(|l| *l == ".").count(), 6);
        assert_eq!(chunks[1], "Mun\noidnen\nviesu\n.\nMun\noidnen\nviesu\n.");
    }

    #[test]
    fn only_punctuation_only_lines_end_a_sentence() {
        assert!(is_sentence_final_line("."));
        assert!(is_sentence_final_line("!"));
        assert!(is_sentence_final_line("?!"));
        assert!(is_sentence_final_line("..."));
        assert!(!is_sentence_final_line(""));
        assert!(!is_sentence_final_line(" "));
        assert!(!is_sentence_final_line(","));
        assert!(!is_sentence_final_line("ikte."));
    }

    /// A stream of lines that never ends a sentence is still cut, at a line
    /// boundary, once it reaches the byte ceiling — the one place a whole
    /// line is what stands in for a whole sentence.
    #[test]
    fn token_lines_without_enders_cut_on_a_line() {
        let tokens: Vec<String> = (0..4000).map(|i| format!("sátni{i}")).collect();
        let chunks = token_line_chunks(&tokens);

        assert!(chunks.len() > 1, "{} chunks", chunks.len());
        for chunk in &chunks {
            assert!(
                !chunk.starts_with('\n') && !chunk.ends_with('\n'),
                "{chunk:?}"
            );
        }
        let carried: Vec<&str> = chunks.iter().flat_map(|chunk| chunk.lines()).collect();
        assert_eq!(
            carried,
            tokens.iter().map(String::as_str).collect::<Vec<_>>()
        );
    }
}
