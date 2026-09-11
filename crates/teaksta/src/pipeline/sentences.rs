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
use std::sync::LazyLock;
use tracing::{debug, error};

use crate::morpho::MorphoPipeline;
use crate::pipeline::mask_to_spans;
use crate::types::{Document, PIPELINE_LANGUAGE, SentenceAnnotation, Spanned, index_order};

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

impl Spanned for PlainTextSentenceAnnotation {
    fn begin(&self) -> usize {
        self.begin
    }
    fn end(&self) -> usize {
        self.end
    }
}

/// Ambiguous, non-strict subiterator: every annotation that begins within
/// `bound`, in index order, including those that extend past its end.
///
/// `ordered` is the whole store in index order, computed once by the caller:
/// begins ascend along it, so the window is the stretch between the first
/// entry beginning at or after the bound and the first beginning past its
/// end, which is found by bisection rather than by a scan.
fn subiterator<'a, T: Spanned, B: Spanned>(
    items: &'a [T],
    ordered: &'a [usize],
    bound: &B,
) -> impl Iterator<Item = &'a T> {
    let from = ordered.partition_point(|&at| items[at].begin() < bound.begin());
    let to = ordered.partition_point(|&at| items[at].begin() <= bound.end());

    ordered[from..to.max(from)]
        .iter()
        .map(move |&at| &items[at])
}

// `\s` is held to the ASCII whitespace set the source pattern matched.
static TRAILING_SPACE_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?-u:\s+)$").expect("trailing space pattern"));
static SENTENCE_BEGIN_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[\p{L}\p{N}\p{P}]").expect("sentence begin pattern"));

/// Wrapper for the sentence detector.
// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector]
// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector.initialize-fn+1]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector.initialize-fn+1]
#[derive(Debug, Clone, Copy, Default)]
pub struct OpenNlpSentenceDetector;

impl OpenNlpSentenceDetector {
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector.process-fn+1]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector.process-fn+1]
    pub fn process(&self, jcas: &mut Document) -> Result<Vec<PlainTextSentenceAnnotation>> {
        debug!("Starting sentence detection");

        // put tokens in their proper positions in an otherwise empty document,
        // so detection runs over the masked buffer rather than the real text
        let rtext = mask_to_spans(&jcas.text, &jcas.tokens)?;

        // Only the key the pipelines are registered under has a detector, so a
        // document naming anything else is refused here.
        if jcas.language != PIPELINE_LANGUAGE {
            error!("No tagger for language: {}", jcas.language);
            bail!("analysis engine process exception");
        }
        let detector = MorphoPipeline::shared();

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

        debug!("Finished sentence detection");
        Ok(sentences)
    }
}

// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator]
#[derive(Debug, Clone, Copy, Default)]
pub struct HtmlSentenceAnnotator;

impl HtmlSentenceAnnotator {
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator.process-fn+3]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator.process-fn+3]
    pub fn process(
        &self,
        jcas: &mut Document,
        sent_index: &[PlainTextSentenceAnnotation],
    ) -> Result<()> {
        debug!("Starting HTML sentence detection");

        let mut produced: Vec<SentenceAnnotation> = Vec::new();
        // The relevant texts are ordered once and windowed per sentence, not
        // ordered again for every one of them.
        let relevant_order = index_order(&jcas.relevant_texts);

        for position in index_order(sent_index) {
            let s = &sent_index[position];
            let rtit = subiterator(&jcas.relevant_texts, &relevant_order, s);

            // end of previous text span in the loop
            let mut prev_rt_end: usize = 0;
            // start of current new sentence under consideration
            //   (may span multiple text segments)
            let mut current_sent_start: Option<usize> = None;
            // end of last sentence added
            let mut last_added_sent_end: Option<usize> = None;
            // Carries no information beyond "the inner loop ran at least once".
            let mut last_s: Option<PlainTextSentenceAnnotation> = None;

            for t in rtit {
                // initialize in first loop
                let start = *current_sent_start.get_or_insert_with(|| {
                    prev_rt_end = s.begin;
                    t.begin
                });

                // if a sentence boundary was not just added but this span
                // opens a block box of its own, insert a sentence boundary
                if start != t.begin && t.block_start {
                    produced.push(SentenceAnnotation {
                        begin: start,
                        end: prev_rt_end,
                    });
                    current_sent_start = Some(t.begin);
                    last_added_sent_end = Some(prev_rt_end);
                }

                prev_rt_end = t.end;
                last_s = Some(*s);
            }

            match (last_added_sent_end, last_s, current_sent_start) {
                // if no sentences were added (because the whole sentence
                // corresponded to a single RelevantText span or all spans
                // were of the same type), add the whole original plain text
                // sentence as a sentence
                (None, _, _) => produced.push(SentenceAnnotation {
                    begin: s.begin,
                    end: s.end,
                }),
                // add the last sentence if needed
                (Some(added), Some(last_s), Some(start)) if added != last_s.end => {
                    produced.push(SentenceAnnotation {
                        begin: start,
                        end: last_s.end,
                    })
                }
                _ => {}
            }
        }

        jcas.sentences.extend(produced);

        debug!("Finished HTML sentence detection");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{RelevantText, Token};

    /// A key no detector is ever registered under. `sme` is the case that
    /// bites: the deployment processes North Sámi, and a document naming the
    /// language of its text rather than the key its pipelines are registered
    /// under finds nothing here.
    const UNREGISTERED_LANGUAGE: &str = "sme";

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
        let mut doc = Document::new("mun guolli", PIPELINE_LANGUAGE);
        doc.relevant_texts.push(relevant(0, 3));
        doc.relevant_texts.push(RelevantText {
            block_start: block,
            ..relevant(4, 10)
        });

        (doc, vec![plain(0, 10)])
    }

    fn html_sentences(block: bool) -> Vec<(usize, usize)> {
        let (mut doc, sents) = two_spans(block);
        HtmlSentenceAnnotator.process(&mut doc, &sents).unwrap();
        doc.sentences.iter().map(|s| (s.begin, s.end)).collect()
    }

    /// Whether the pass refuses a document naming this language before it ever
    /// reaches the models.
    fn refused(language: &str) -> bool {
        let mut doc = Document::new("Mun", language);
        doc.tokens.push(token(0, 3));

        OpenNlpSentenceDetector
            .process(&mut doc)
            .err()
            .is_some_and(|e| e.to_string().contains("analysis engine process exception"))
    }

    /// The registry the Java replaced wholesale on every initialisation held
    /// exactly one entry, so what it decided was that any language but the
    /// key the pipelines are registered under is refused — which needs
    /// neither a registry nor an initialisation step to say.
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector.initialize-fn+1/test]
    #[test]
    fn only_the_registered_key_reaches_a_detector() {
        assert!(refused(UNREGISTERED_LANGUAGE));
        assert!(refused("de"));
        assert!(refused(""));
        assert!(!refused(PIPELINE_LANGUAGE));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector.process-fn+1/test]
    #[test]
    fn process_refuses_a_language_that_has_no_detector() {
        let mut doc = Document::new("Mun boran guoli.", UNREGISTERED_LANGUAGE);
        doc.tokens.push(token(0, 3));

        let err = OpenNlpSentenceDetector
            .process(&mut doc)
            .expect_err("no detector is ever registered for sme");

        assert!(
            err.to_string()
                .contains("analysis engine process exception")
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector.process-fn+1/test]
    #[test]
    fn process_masks_tokens_before_detector_lookup() {
        // The masking failure wins over the lookup that would have failed too.
        let mut doc = Document::new("Mun", UNREGISTERED_LANGUAGE);
        doc.tokens.push(token(0, 99));

        let err = OpenNlpSentenceDetector
            .process(&mut doc)
            .expect_err("the token span runs off the end of the document");

        assert!(err.to_string().contains("is not within the document"));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.open-nlp-sentence-detector.open-nlp-sentence-detector.process-fn+1/test]
    #[test]
    fn process_over_the_registered_key_yields_sentences() {
        let detector = OpenNlpSentenceDetector;

        // North Sámi text under the key the pipelines are registered under,
        // which is the pair a running deployment hands the detector.
        let mut doc = Document::new("Mun boran guoli.", PIPELINE_LANGUAGE);
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

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator.process-fn+3/test]
    #[test]
    fn html_process_keeps_sentence_whole_without_relevant_text() {
        let mut doc = Document::new("Mun boran guoli.", PIPELINE_LANGUAGE);

        HtmlSentenceAnnotator
            .process(&mut doc, &[plain(0, 16)])
            .unwrap();

        assert_eq!(doc.sentences.len(), 1);
        assert_eq!((doc.sentences[0].begin, doc.sentences[0].end), (0, 16));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator.process-fn+3/test]
    #[test]
    fn html_process_keeps_sentence_whole_within_one_span() {
        let mut doc = Document::new("Mun boran guoli.", PIPELINE_LANGUAGE);
        doc.relevant_texts.push(relevant(0, 16));

        HtmlSentenceAnnotator
            .process(&mut doc, &[plain(0, 16)])
            .unwrap();

        assert_eq!(doc.sentences.len(), 1);
        assert_eq!((doc.sentences[0].begin, doc.sentences[0].end), (0, 16));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator.process-fn+3/test]
    #[test]
    fn html_process_splits_sentence_at_a_block_start() {
        assert_eq!(html_sentences(true), vec![(0, 3), (4, 10)]);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator.process-fn+3/test]
    #[test]
    fn html_process_keeps_one_block_sentence_whole() {
        assert_eq!(html_sentences(false), vec![(0, 10)]);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator.process-fn+3/test]
    #[test]
    fn html_process_never_breaks_before_the_first_span() {
        let mut doc = Document::new("mun guolli", PIPELINE_LANGUAGE);
        doc.relevant_texts.push(RelevantText {
            block_start: true,
            ..relevant(0, 3)
        });

        HtmlSentenceAnnotator
            .process(&mut doc, &[plain(0, 10)])
            .unwrap();

        assert_eq!(doc.sentences.len(), 1);
        assert_eq!((doc.sentences[0].begin, doc.sentences[0].end), (0, 10));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.html-sentence-annotator.html-sentence-annotator.process-fn+3/test]
    #[test]
    fn html_process_appends_one_annotation_per_sentence() {
        let mut doc = Document::new("Mun boran. Guolli lea buorre.", PIPELINE_LANGUAGE);
        doc.sentences.push(SentenceAnnotation { begin: 0, end: 0 });

        HtmlSentenceAnnotator
            .process(&mut doc, &[plain(11, 29), plain(0, 10)])
            .unwrap();

        let spans: Vec<(usize, usize)> = doc.sentences.iter().map(|s| (s.begin, s.end)).collect();
        assert_eq!(spans, vec![(0, 0), (0, 10), (11, 29)]);
    }
}
