//! Span-annotation document model replacing the UIMA CAS and the JCasGen
//! type system (WERTiTypeSystem.xml + vislcg3TypeSystem.xml). Every
//! annotation is a half-open byte span `[begin, end)` into `Document::text`.
//!
//! The document is serialisable so the page handler can cache an analysed
//! document between requests, which is what the UIMA XMI cache did.

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

    /// The text covered by a `[begin, end)` span. Panics when the span is
    /// not on a char boundary or out of range, matching the strictness of
    /// the UIMA covered-text accessor.
    pub fn covered_text(&self, begin: usize, end: usize) -> &str {
        &self.text[begin..end]
    }
}
