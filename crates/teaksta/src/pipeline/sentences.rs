//! Sentence detection.
//!
//! `OpenNlpSentenceDetector` turns the token-masked document into plain-text
//! sentence boundaries; `HtmlSentenceAnnotator` splits those further wherever
//! a relevant text span opens a new block box in the page it came from.
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

// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator]
#[derive(Debug, Clone, Copy, Default)]
pub struct HtmlSentenceAnnotator;

impl HtmlSentenceAnnotator {
    pub fn new() -> Self {
        HtmlSentenceAnnotator
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator.process-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator.process-fn+2]
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

                // if a sentence boundary was not just added but this span
                // opens a block box of its own, insert a sentence boundary
                if current_sent_start != t.begin as i64 && t.block_start {
                    produced.push(SentenceAnnotation {
                        begin: current_sent_start as usize,
                        end: prev_rt_end,
                    });
                    current_sent_start = t.begin as i64;
                    last_added_sent_end = prev_rt_end as i64;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn token(begin: usize, end: usize) -> Token {
        Token {
            begin,
            end,
            ..Default::default()
        }
    }

    fn relevant(begin: usize, end: usize) -> RelevantText {
        RelevantText {
            begin,
            end,
            ..Default::default()
        }
    }

    fn plain(begin: usize, end: usize) -> PlainTextSentenceAnnotation {
        PlainTextSentenceAnnotation { begin, end }
    }

    /// `mun guolli`, with a relevant span per word and a single plain-text
    /// sentence covering both. `block` says whether the second word opened a
    /// block box of its own in the page the text came from.
    fn two_spans(block: bool) -> (Document, Vec<PlainTextSentenceAnnotation>) {
        let mut doc = Document::new("mun guolli", "sme");
        doc.relevant_texts.push(relevant(0, 3));
        doc.relevant_texts.push(RelevantText {
            block_start: block,
            ..relevant(4, 10)
        });

        (doc, vec![plain(0, 10)])
    }

    fn html_sentences(block: bool) -> Vec<(usize, usize)> {
        let (mut doc, sents) = two_spans(block);
        HtmlSentenceAnnotator::new()
            .process(&mut doc, &sents)
            .unwrap();
        doc.sentences.iter().map(|s| (s.begin, s.end)).collect()
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector.initialize-fn/test]
    #[test]
    fn initialize_registers_english_and_nothing_else() {
        OpenNlpSentenceDetector::new().initialize().unwrap();

        let registry = detectors();
        assert_eq!(registry.len(), 1);
        assert!(registry.contains_key("en"));
        assert!(!registry.contains_key("sme"));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector.process-fn/test]
    #[test]
    fn process_refuses_a_language_that_has_no_detector() {
        let mut doc = Document::new("Mun boran guoli.", "sme");
        doc.tokens.push(token(0, 3));

        let err = OpenNlpSentenceDetector::new()
            .process(&mut doc)
            .expect_err("no detector is ever registered for sme");

        assert!(
            err.to_string()
                .contains("analysis engine process exception")
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector.process-fn/test]
    #[test]
    fn process_masks_tokens_before_detector_lookup() {
        let mut doc = Document::new("Mun", "sme");
        doc.tokens.push(token(0, 99));

        let err = OpenNlpSentenceDetector::new()
            .process(&mut doc)
            .expect_err("the token span runs off the end of the document");

        assert!(err.to_string().contains("is not within the document"));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector.process-fn/test]
    #[test]
    fn process_over_english_yields_at_least_one_sentence() {
        let mut detector = OpenNlpSentenceDetector::new();
        detector.initialize().unwrap();

        let mut doc = Document::new("Mun boran guoli.", "en");
        doc.tokens.push(token(0, 3));
        doc.tokens.push(token(4, 9));
        doc.tokens.push(token(10, 16));

        match detector.process(&mut doc) {
            Ok(sentences) => {
                assert!(!sentences.is_empty());
                for s in &sentences {
                    assert!(s.begin <= s.end);
                    assert!(s.end <= doc.text.len());
                }
            }
            Err(e) => {
                let message = e.to_string();
                assert!(
                    message.contains("TEAKSTA_BUNDLE") || message.contains("sentences"),
                    "unexpected failure: {message}"
                );
            }
        }
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator.process-fn+2/test]
    #[test]
    fn html_process_keeps_sentence_whole_without_relevant_text() {
        let mut doc = Document::new("Mun boran guoli.", "sme");

        HtmlSentenceAnnotator::new()
            .process(&mut doc, &[plain(0, 16)])
            .unwrap();

        assert_eq!(doc.sentences.len(), 1);
        assert_eq!((doc.sentences[0].begin, doc.sentences[0].end), (0, 16));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator.process-fn+2/test]
    #[test]
    fn html_process_keeps_sentence_whole_within_one_span() {
        let mut doc = Document::new("Mun boran guoli.", "sme");
        doc.relevant_texts.push(relevant(0, 16));

        HtmlSentenceAnnotator::new()
            .process(&mut doc, &[plain(0, 16)])
            .unwrap();

        assert_eq!(doc.sentences.len(), 1);
        assert_eq!((doc.sentences[0].begin, doc.sentences[0].end), (0, 16));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator.process-fn+2/test]
    #[test]
    fn html_process_splits_sentence_at_a_block_start() {
        assert_eq!(html_sentences(true), vec![(0, 3), (4, 10)]);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator.process-fn+2/test]
    #[test]
    fn html_process_keeps_one_block_sentence_whole() {
        assert_eq!(html_sentences(false), vec![(0, 10)]);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator.process-fn+2/test]
    #[test]
    fn html_process_never_breaks_before_the_first_span() {
        let mut doc = Document::new("mun guolli", "sme");
        doc.relevant_texts.push(RelevantText {
            block_start: true,
            ..relevant(0, 3)
        });

        HtmlSentenceAnnotator::new()
            .process(&mut doc, &[plain(0, 10)])
            .unwrap();

        assert_eq!(doc.sentences.len(), 1);
        assert_eq!((doc.sentences[0].begin, doc.sentences[0].end), (0, 10));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator.process-fn+2/test]
    #[test]
    fn html_process_appends_one_annotation_per_sentence() {
        let mut doc = Document::new("Mun boran. Guolli lea buorre.", "sme");
        doc.sentences.push(SentenceAnnotation { begin: 0, end: 0 });

        HtmlSentenceAnnotator::new()
            .process(&mut doc, &[plain(11, 29), plain(0, 10)])
            .unwrap();

        let spans: Vec<(usize, usize)> = doc.sentences.iter().map(|s| (s.begin, s.end)).collect();
        assert_eq!(spans, vec![(0, 0), (0, 10), (11, 29)]);
    }
}
