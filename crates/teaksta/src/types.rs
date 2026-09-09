//! Span-annotation document model replacing the UIMA CAS and the JCasGen
//! type system (WERTiTypeSystem.xml + vislcg3TypeSystem.xml). Every
//! annotation is a half-open byte span `[begin, end)` into `Document::text`.

/// One morphological reading of a token, as produced by the analyser +
/// CG3 disambiguation. The first element is the lemma line content; the
/// remaining elements are the tag strings of the reading.
pub type CgReading = Vec<String>;

#[derive(Debug, Clone, Default)]
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
#[derive(Debug, Clone, Default)]
pub struct CgToken {
    pub begin: usize,
    pub end: usize,
    pub readings: Vec<CgReading>,
}

/// A stretch of document text classified as relevant (or not) for
/// enhancement, with the HTML context it was found in.
#[derive(Debug, Clone, Default)]
pub struct RelevantText {
    pub begin: usize,
    pub end: usize,
    pub relevant: bool,
    pub html_content_type: Option<String>,
    pub enclosing_tag: Option<String>,
}

/// One `<e>`-protocol XML tag occurrence in the document text.
#[derive(Debug, Clone, Default)]
pub struct EnhanceXml {
    pub begin: usize,
    pub end: usize,
    pub tag_name: String,
    pub closing: bool,
    pub irrelevant: bool,
}

/// The HTML fragments to splice around a span in the final output.
#[derive(Debug, Clone, Default)]
pub struct Enhancement {
    pub begin: usize,
    pub end: usize,
    pub enhance_start: String,
    pub enhance_end: String,
    pub relevant: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SentenceAnnotation {
    pub begin: usize,
    pub end: usize,
}

/// Document-wide enhancement-id marker (`werti.uima.types.global.
/// EnhancementId`): a `DocumentAnnotation` subtype carrying one long-valued
/// `enhId` feature, used as a whole-document validity marker rather than a
/// span, so `begin` and `end` stay 0.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EnhancementId {
    pub begin: usize,
    pub end: usize,
    pub enh_id: i64,
}

/// The analysis document: the text under analysis plus one store per
/// annotation type. Pipeline stages consume and extend the stores.
#[derive(Debug, Clone, Default)]
pub struct Document {
    pub text: String,
    pub language: String,
    pub tokens: Vec<Token>,
    pub cg_tokens: Vec<CgToken>,
    pub relevant_texts: Vec<RelevantText>,
    pub enhance_xml: Vec<EnhanceXml>,
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
