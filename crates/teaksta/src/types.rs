//! Span-annotation document model replacing the UIMA CAS and the JCasGen
//! type system (WERTiTypeSystem.xml + vislcg3TypeSystem.xml). Every
//! annotation is a half-open byte span `[begin, end)` into `Document::text`.
//!
//! The document is serialisable so the page handler can cache an analysed
//! document between requests, which is what the UIMA XMI cache did.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

/// One morphological reading of a token, as produced by the analyser +
/// CG3 disambiguation. The first element is the lemma line content; the
/// remaining elements are the tag strings of the reading.
pub type CgReading = Vec<String>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Token {
    pub begin: usize,
    pub end: usize,
    pub tag: Option<String>,
    pub detailedtag: Option<String>,
    pub lemma: Option<String>,
    pub chunk: Option<String>,
    pub mltag: Option<String>,
}

/// Token bearing the full set of CG3 readings that survived disambiguation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CgToken {
    pub begin: usize,
    pub end: usize,
    pub readings: Vec<CgReading>,
}

/// A stretch of document text classified as relevant (or not) for
/// enhancement, with the HTML context it was found in.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelevantText {
    pub begin: usize,
    pub end: usize,
    pub relevant: bool,
    pub html_content_type: Option<String>,
    pub enclosing_tag: Option<String>,
    /// Whether a block-level element boundary sits before this stretch, so a
    /// sentence running across it is really two.
    pub block_start: bool,
}

/// Where one stretch of the analysed text sits in the page it was taken
/// from: a half-open range of [`Document::text`] paired with the DOM text
/// node that supplied it, addressed by its position in a document-order walk
/// of the page's text nodes.
///
/// The range covers a whole text node, so a document offset inside it is the
/// same distance into the node's own text.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextSegment {
    pub begin: usize,
    pub end: usize,
    pub node: usize,
    pub block_start: bool,
}

/// The page an analysis document was extracted from, with the map from its
/// text back to the DOM. Held as source rather than as a parsed tree because
/// the analysis document is cached between requests, and reparsing is
/// deterministic: the same source yields the same walk.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PageMap {
    pub html: String,
    pub segments: Vec<TextSegment>,
}

/// The HTML fragments to splice around a span in the final output.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Enhancement {
    pub begin: usize,
    pub end: usize,
    pub enhance_start: String,
    pub enhance_end: String,
    pub relevant: bool,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct SentenceAnnotation {
    pub begin: usize,
    pub end: usize,
}

/// Document-wide enhancement-id marker (`werti.uima.types.global.
/// EnhancementId`): a `DocumentAnnotation` subtype carrying one long-valued
/// `enhId` feature, used as a whole-document validity marker rather than a
/// span, so `begin` and `end` stay 0.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnhancementId {
    pub begin: usize,
    pub end: usize,
    pub enh_id: i64,
}

/// The key the shipped activity descriptors register their pipelines under,
/// and so what [`Document::language`] carries in a running deployment.
///
/// It reads as a language code and is not one: every topic registers its pre-
/// and postprocessor under `en` because no analysis engine was ever registered
/// under `sme`, and the pipelines behind that key are the North Sámi ones. It
/// sits beside the document rather than beside the handlers so that a test
/// builds one the pipelines would actually accept.
pub const PIPELINE_LANGUAGE: &str = "en";

/// The analysis document: the text under analysis plus one store per
/// annotation type. Pipeline stages consume and extend the stores.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Document {
    pub text: String,
    /// The registry key the pipelines processing this document are looked up
    /// under — [`PIPELINE_LANGUAGE`] in a running deployment — and not the
    /// language the text is written in, which nothing here records. Empty
    /// means the document was reset, which is what
    /// [`crate::util::cas_utils::has_been_reset`] reads it for.
    pub language: String,
    pub page: PageMap,
    pub tokens: Vec<Token>,
    pub cg_tokens: Vec<CgToken>,
    pub relevant_texts: Vec<RelevantText>,
    pub enhancements: Vec<Enhancement>,
    pub sentences: Vec<SentenceAnnotation>,
    pub enhancement_ids: Vec<EnhancementId>,
}

impl Document {
    pub fn new(text: impl Into<String>, language: impl Into<String>) -> Self {
        Document {
            text: text.into(),
            language: language.into(),
            ..Default::default()
        }
    }

    /// The first span in the document that is not a readable stretch of
    /// [`Document::text`], named by the store holding it, or `None` when
    /// every span is readable.
    ///
    /// A document assembled by the pipeline stages is readable by
    /// construction — every offset came from the text it indexes. One decoded
    /// from the analysis cache is a file, so its offsets are checked here
    /// before a stage indexes the text with them.
    pub fn invalid_span(&self) -> Option<InvalidSpan> {
        let mut spans = std::iter::empty()
            .chain(spans_of("tokens", &self.tokens, |t| (t.begin, t.end)))
            .chain(spans_of("cg_tokens", &self.cg_tokens, |t| (t.begin, t.end)))
            .chain(spans_of("relevant_texts", &self.relevant_texts, |t| {
                (t.begin, t.end)
            }))
            .chain(spans_of("enhancements", &self.enhancements, |e| {
                (e.begin, e.end)
            }))
            .chain(spans_of("sentences", &self.sentences, |s| (s.begin, s.end)))
            .chain(spans_of("enhancement_ids", &self.enhancement_ids, |i| {
                (i.begin, i.end)
            }))
            .chain(spans_of("page.segments", &self.page.segments, |s| {
                (s.begin, s.end)
            }));

        spans.find(|span| covered_text(&self.text, span.begin, span.end).is_err())
    }
}

/// The text a `[begin, end)` span covers, or the failure naming the span when
/// it runs past the end of the text, ends before it begins, or falls inside a
/// multibyte character. North Sámi carries multibyte characters throughout, so
/// an offset a byte off a character boundary is an ordinary consequence of a
/// span paired with the wrong text rather than a remote one.
///
/// Every stage that reads the text at an annotation's offsets comes through
/// here, so no offset the document carries can end a request by panicking.
pub fn covered_text(text: &str, begin: usize, end: usize) -> Result<&str> {
    text.get(begin..end)
        .ok_or_else(|| anyhow!("span {}..{} is not within the document text", begin, end))
}

/// A span that does not index the text it belongs to, named by its store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidSpan {
    pub store: &'static str,
    pub begin: usize,
    pub end: usize,
}

impl std::fmt::Display for InvalidSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} span {}..{} is not a stretch of the document text",
            self.store, self.begin, self.end
        )
    }
}

fn spans_of<T>(
    store: &'static str,
    items: &[T],
    span: impl Fn(&T) -> (usize, usize),
) -> impl Iterator<Item = InvalidSpan> {
    items.iter().map(move |item| {
        let (begin, end) = span(item);
        InvalidSpan { store, begin, end }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A word whose second character is two bytes wide, so an offset one byte
    /// past its start is inside a character rather than between two.
    const TEXT: &str = "Sámegiella";

    fn covered(begin: usize, end: usize) -> Option<String> {
        covered_text(TEXT, begin, end).ok().map(str::to_string)
    }

    #[test]
    fn a_mid_character_span_is_uncovered() {
        assert_eq!(covered(0, 1).as_deref(), Some("S"));
        assert_eq!(covered(0, TEXT.len()).as_deref(), Some(TEXT));
        assert_eq!(covered(4, 4).as_deref(), Some(""));
        // `á` occupies bytes 1 and 2.
        assert_eq!(covered(1, 3).as_deref(), Some("á"));
        assert_eq!(covered(0, 2), None);
        assert_eq!(covered(2, 4), None);
    }

    #[test]
    fn a_span_outside_the_text_is_uncovered() {
        assert_eq!(covered(0, TEXT.len() + 1), None);
        assert_eq!(covered(TEXT.len() + 1, TEXT.len() + 2), None);
        assert_eq!(covered(5, 3), None);
        assert_eq!(covered(usize::MAX, usize::MAX), None);
        assert_eq!(
            covered_text(TEXT, 0, 99).unwrap_err().to_string(),
            "span 0..99 is not within the document text"
        );
    }

    /// The store the document reports an unreadable span in, once `add` has
    /// put one more annotation beside a readable token.
    fn reported_store(add: impl FnOnce(&mut Document)) -> Option<&'static str> {
        let mut doc = Document::new(TEXT, PIPELINE_LANGUAGE);
        doc.tokens.push(Token {
            begin: 0,
            end: 1,
            ..Token::default()
        });
        add(&mut doc);
        doc.invalid_span().map(|span| span.store)
    }

    #[test]
    fn every_store_is_checked_for_unreadable_spans() {
        assert_eq!(reported_store(|_| {}), None);
        // Each span below is either inside the two-byte `á` or past the end.
        assert_eq!(
            reported_store(|doc| doc.tokens.push(Token {
                begin: 0,
                end: 2,
                ..Token::default()
            })),
            Some("tokens")
        );
        assert_eq!(
            reported_store(|doc| doc.cg_tokens.push(CgToken {
                begin: 0,
                end: 99,
                ..CgToken::default()
            })),
            Some("cg_tokens")
        );
        assert_eq!(
            reported_store(|doc| doc.relevant_texts.push(RelevantText {
                begin: 2,
                end: 4,
                ..RelevantText::default()
            })),
            Some("relevant_texts")
        );
        assert_eq!(
            reported_store(|doc| doc.enhancements.push(Enhancement {
                begin: 99,
                end: 100,
                ..Enhancement::default()
            })),
            Some("enhancements")
        );
        assert_eq!(
            reported_store(|doc| doc.sentences.push(SentenceAnnotation { begin: 0, end: 2 })),
            Some("sentences")
        );
        assert_eq!(
            reported_store(|doc| doc.enhancement_ids.push(EnhancementId {
                begin: 0,
                end: 2,
                enh_id: 1,
            })),
            Some("enhancement_ids")
        );
        assert_eq!(
            reported_store(|doc| doc.page.segments.push(TextSegment {
                begin: 0,
                end: 2,
                ..TextSegment::default()
            })),
            Some("page.segments")
        );
    }
}
