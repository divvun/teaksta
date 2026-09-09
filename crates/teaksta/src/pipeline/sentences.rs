//! Sentence detection.
//!
//! `OpenNlpSentenceDetector` turns the token-masked document into plain-text
//! sentence boundaries; `HtmlSentenceAnnotator` splits those further wherever
//! block-level HTML markup sits between two relevant text spans.
//!
//! Both depend on the [`Token`] annotations from
//! [`crate::pipeline::tokenizer::GiellateknoTokenizer`].

use anyhow::{Result, anyhow, bail};
use regex::Regex;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, MutexGuard};
use tracing::{error, info};

use crate::morpho::MorphoPipeline;
use crate::types::{Document, RelevantText, SentenceAnnotation, Token};

/// Plain-text sentence boundary: produced by [`OpenNlpSentenceDetector`],
/// consumed by [`HtmlSentenceAnnotator`]. Kept apart from
/// [`SentenceAnnotation`], which is the enhanced-output sentence type, and
/// threaded between the two stages rather than held on [`Document`], which
/// has no store for it.
#[derive(Debug, Clone, Copy, Default)]
pub struct PlainTextSentenceAnnotation {
    pub begin: usize,
    pub end: usize,
}

trait Spanned {
    fn begin(&self) -> usize;
    fn end(&self) -> usize;
}

impl Spanned for Token {
    fn begin(&self) -> usize {
        self.begin
    }
    fn end(&self) -> usize {
        self.end
    }
}

impl Spanned for RelevantText {
    fn begin(&self) -> usize {
        self.begin
    }
    fn end(&self) -> usize {
        self.end
    }
}

impl Spanned for PlainTextSentenceAnnotation {
    fn begin(&self) -> usize {
        self.begin
    }
    fn end(&self) -> usize {
        self.end
    }
}

/// Annotation index order: ascending `begin`, then descending `end`.
fn index_order<T: Spanned>(items: &[T]) -> Vec<&T> {
    let mut ordered: Vec<&T> = items.iter().collect();
    ordered.sort_by(|a, b| a.begin().cmp(&b.begin()).then(b.end().cmp(&a.end())));
    ordered
}

/// Ambiguous, non-strict subiterator: every annotation that begins within
/// `bound`, in index order, including those that extend past its end.
fn subiterator<'a, T: Spanned, B: Spanned>(items: &'a [T], bound: &B) -> Vec<&'a T> {
    index_order(items)
        .into_iter()
        .filter(|t| t.begin() >= bound.begin() && t.begin() <= bound.end())
        .collect()
}

/// The document text with everything outside the given spans blanked to
/// spaces, at byte-for-byte identical offsets.
fn mask_to_spans<T: Spanned>(text: &str, spans: &[T]) -> Result<String> {
    let mut rtext = vec![b' '; text.len()];

    for t in index_order(spans) {
        let covered = text
            .get(t.begin()..t.end())
            .ok_or_else(|| anyhow!("span {}..{} is not within the document", t.begin(), t.end()))?;
        rtext[t.begin()..t.end()].copy_from_slice(covered.as_bytes());
    }

    Ok(String::from_utf8(rtext)?)
}

// `\s` is held to the ASCII whitespace set the source pattern matched.
static TRAILING_SPACE_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?-u:\s+)$").expect("trailing space pattern"));
static SENTENCE_BEGIN_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[\p{L}\p{N}\p{P}]").expect("sentence begin pattern"));

/// Language code to sentence detector. Replaced wholesale on every
/// initialisation, so the last initialised instance owns the registry.
static DETECTORS: LazyLock<Mutex<HashMap<String, &'static MorphoPipeline>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn detectors() -> MutexGuard<'static, HashMap<String, &'static MorphoPipeline>> {
    DETECTORS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Wrapper for the sentence detector.
// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector]
#[derive(Debug, Clone, Copy, Default)]
pub struct OpenNlpSentenceDetector;

impl OpenNlpSentenceDetector {
    pub fn new() -> Self {
        OpenNlpSentenceDetector
    }

    /// Only `"en"` is ever registered, even though the deployment processes
    /// North Sámi, so [`Self::process`] fails for every other language.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector.initialize-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector.initialize-fn]
    pub fn initialize(&mut self) -> Result<()> {
        let mut registry = HashMap::new();
        registry.insert("en".to_string(), MorphoPipeline::shared());
        *detectors() = registry;

        Ok(())
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector.process-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector.process-fn]
    pub fn process(&self, jcas: &mut Document) -> Result<Vec<PlainTextSentenceAnnotation>> {
        info!("Starting sentence detection");

        // put tokens in their proper positions in an otherwise empty document,
        // so detection runs over the masked buffer rather than the real text
        let rtext = mask_to_spans(&jcas.text, &jcas.tokens)?;

        let lang = jcas.language.clone();
        let detector = match detectors().get(&lang) {
            Some(detector) => *detector,
            None => {
                error!("No tagger for language: {}", lang);
                bail!("analysis engine process exception");
            }
        };

        // sentence end positions within the masked buffer
        let offsets: Vec<usize> = detector
            .sentence_spans(&rtext)?
            .into_iter()
            .map(|(_, end)| end)
            .collect();

        let mut sentences: Vec<PlainTextSentenceAnnotation> = Vec::new();

        // iterate one past the end of offsets.len() and use the end of
        // rtext instead of offsets[i] in the last iteration
        for i in 0..=offsets.len() {
            let current_offset = if i == offsets.len() {
                rtext.len()
            } else {
                offsets[i]
            };
            let previous_offset = if i > 0 { offsets[i - 1] } else { 0 };

            let sentence_str = rtext.get(previous_offset..current_offset).ok_or_else(|| {
                anyhow!("sentence slice {previous_offset}..{current_offset} is out of range")
            })?;
            let sentence_begin = SENTENCE_BEGIN_PATTERN
                .find(sentence_str)
                .map(|m| m.start())
                .unwrap_or(0);
            let sentence_end = TRAILING_SPACE_PATTERN
                .find(sentence_str)
                .map(|m| m.start())
                .unwrap_or(sentence_str.len());
            let start = previous_offset + sentence_begin;
            let end = previous_offset + sentence_end;
            sentences.push(PlainTextSentenceAnnotation { begin: start, end });
        }

        info!("Finished sentence detection");
        Ok(sentences)
    }
}

// HTML tags that typically indicate sentence breaks, but not necessarily
// a shift in content type. The `<h[1..6]` alternative is a class over the
// literal characters `1`, `.` and `6`, and `</h[1-6]` has no closing `>`.
static HTML_BREAK_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)^.*(<li|</li>|<ul|</ul>|<ol|</ol>|<h[1..6]|</h[1-6]).*$")
        .expect("html break pattern")
});

// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator]
#[derive(Debug, Clone, Copy, Default)]
pub struct HtmlSentenceAnnotator;

impl HtmlSentenceAnnotator {
    pub fn new() -> Self {
        HtmlSentenceAnnotator
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator.process-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator.process-fn]
    pub fn process(
        &self,
        jcas: &mut Document,
        sent_index: &[PlainTextSentenceAnnotation],
    ) -> Result<()> {
        info!("Starting HTML sentence detection");

        let mut produced: Vec<SentenceAnnotation> = Vec::new();

        for s in index_order(sent_index) {
            let rtit = subiterator(&jcas.relevant_texts, s);

            // end of previous text span in the loop
            let mut prev_rt_end: usize = 0;
            // start of current new sentence under consideration
            //   (may span multiple text segments)
            let mut current_sent_start: i64 = -1;
            // end of last sentence added
            let mut last_added_sent_end: i64 = -1;
            // Carries no information beyond "the inner loop ran at least once".
            let mut last_s: Option<PlainTextSentenceAnnotation> = None;

            for t in rtit {
                // initialize in first loop
                if current_sent_start == -1 {
                    current_sent_start = t.begin as i64;
                    prev_rt_end = s.begin;
                }

                // if a sentence boundary was not just added but any of the
                // HTML tags in the pattern appear between the previous rt span
                // and the current one, insert a sentence boundary
                if current_sent_start != t.begin as i64 {
                    // Taken with no ordering check: out-of-order spans fail here.
                    let gap = jcas.text.get(prev_rt_end..t.begin).ok_or_else(|| {
                        anyhow!("gap slice {}..{} is out of range", prev_rt_end, t.begin)
                    })?;
                    if HTML_BREAK_PATTERN.is_match(&gap.to_lowercase()) {
                        produced.push(SentenceAnnotation {
                            begin: current_sent_start as usize,
                            end: prev_rt_end,
                        });
                        current_sent_start = t.begin as i64;
                        last_added_sent_end = prev_rt_end as i64;
                    }
                }

                prev_rt_end = t.end;
                last_s = Some(*s);
            }

            // if no sentences were added (because the whole sentence
            // corresponded to a single RelevantText span or all spans were
            // of the same type), add the whole original plain text sentence
            // as a sentence
            if last_added_sent_end == -1 {
                produced.push(SentenceAnnotation {
                    begin: s.begin,
                    end: s.end,
                });
            // add the last sentence if needed
            } else if let Some(last_s) = last_s {
                if last_added_sent_end != last_s.end as i64 {
                    produced.push(SentenceAnnotation {
                        begin: current_sent_start as usize,
                        end: last_s.end,
                    });
                }
            }
        }

        jcas.sentences.extend(produced);

        info!("Finished HTML sentence detection");
        Ok(())
    }
}
