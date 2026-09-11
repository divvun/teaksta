//! Span-annotation document model replacing the UIMA CAS and the JCasGen
//! type system (WERTiTypeSystem.xml + vislcg3TypeSystem.xml). Every
//! annotation is a half-open byte span `[begin, end)` into `Document::text`.
//!
//! The document is serialisable so the page handler can cache an analysed
//! document between requests, which is what the UIMA XMI cache did.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

/// One morphological reading of a token, as produced by the analyser + CG3
/// disambiguation: the whitespace-separated fields of a CG-3 reading line, in
/// the order the stream wrote them.
///
/// It is a flat list of fields and not a base form paired with its tags,
/// because a reading does not reliably have exactly one base form. A
/// subreading is one level further indented and qualifies the reading above
/// it, so its fields — its own quoted base form among them — are appended to
/// that reading, and a reading built that way carries several quoted fields
/// among its tags. Every topic that wants a base form therefore scans the
/// whole list and keeps the *last* quoted field rather than reading a
/// position, and every topic that matches tags flattens the list and matches
/// against the flattening. Splitting the first field off as a `lemma` would
/// name a field that is not always the base form the topics use.
pub type CgReading = Vec<String>;

/// How a topic reads a reading's tags as one string. Every enhancer matches
/// its tag patterns against a flattened reading rather than against the tags
/// themselves, and the two shapes below are the ones the Java classes built
/// inline — kept apart because the patterns are matched literally, so the
/// separator is part of what a pattern can see.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadingJoin {
    /// Each tag followed by one space, so the result ends in a space:
    /// `"gietta" N Sg Nom `.
    TrailingSpace,
    /// Each tag preceded by a `+`, so the result opens with one:
    /// `+"gietta"+N+Sg+Nom`.
    LeadingPlus,
}

/// Append a reading's tags to `out` as one string. The buffer belongs to the
/// caller so a loop over the readings of a token flattens into one
/// allocation; an empty reading appends nothing.
pub fn flatten_reading_into(out: &mut String, cgr: &CgReading, join: ReadingJoin) {
    for rtag in cgr {
        match join {
            ReadingJoin::TrailingSpace => {
                out.push_str(rtag);
                out.push(' ');
            }
            ReadingJoin::LeadingPlus => {
                out.push('+');
                out.push_str(rtag);
            }
        }
    }
}

/// [`flatten_reading_into`] onto a string of its own, for the callers that
/// read one reading and have no buffer to reuse.
pub fn flatten_reading(cgr: &CgReading, join: ReadingJoin) -> String {
    let mut out = String::new();
    flatten_reading_into(&mut out, cgr, join);
    out
}

/// The tags the constraint grammar marks a cohort with when what it covers
/// is punctuation rather than a word.
///
/// `CLB` is the clause boundary — the full stop, comma, colon, semicolon,
/// exclamation and question marks and the ellipsis all come back carrying it
/// — and `PUNCT` is the rest: the quotation marks, brackets and dashes, whose
/// `LEFT` and `RIGHT` qualifiers ride beside `PUNCT` rather than replacing
/// it. Both are read off the real stream rather than assumed.
pub const PUNCTUATION_TAGS: [&str; 2] = ["CLB", "PUNCT"];

/// Whether the analysis says this cohort covers punctuation rather than a
/// word, and so that nothing may be built on it: no topic hit, no decoy, no
/// slot in a question.
///
/// The test is over the tags of the cohort's readings, so a base form whose
/// own letters spell `CLB` is a word and not a boundary. It asks every
/// reading rather than the first, because a cohort the analysis calls
/// punctuation on any reading at all is not a word a learner is asked about
/// — and because that is what makes the answer independent of which reading
/// a topic would have picked.
///
/// This is a guard and not a filter: it is the last thing standing between a
/// mistake in the offsets layer and a full stop offered to a learner as a
/// word to click or a blank to fill, so every consumer of the CG tokens
/// applies it before it looks at a reading.
pub fn is_punctuation_cohort(token: &CgToken) -> bool {
    token.readings.iter().any(|reading| {
        reading
            .iter()
            .any(|tag| PUNCTUATION_TAGS.contains(&tag.as_str()))
    })
}

/// The exercise a request asks for.
// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.enhancement-type+2]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.enhancement-type+2]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Colorize,
    Click,
    Mc,
    Cloze,
}

impl Mode {
    pub const ALL: [Mode; 4] = [Mode::Colorize, Mode::Click, Mode::Mc, Mode::Cloze];

    pub fn parse(value: &str) -> Option<Mode> {
        Mode::ALL.into_iter().find(|mode| mode.name() == value)
    }

    pub fn name(self) -> &'static str {
        match self {
            Mode::Colorize => "colorize",
            Mode::Click => "click",
            Mode::Mc => "mc",
            Mode::Cloze => "cloze",
        }
    }
}

/// A half-open `[begin, end)` byte span into a document's text: what every
/// annotation store below is a list of. The pipeline stages order and window
/// their annotations through this one trait, so the index order is written
/// once rather than per store.
pub trait Spanned {
    fn begin(&self) -> usize;
    fn end(&self) -> usize;
}

macro_rules! spanned {
    ($($t:ty),* $(,)?) => {
        $(impl Spanned for $t {
            fn begin(&self) -> usize {
                self.begin
            }
            fn end(&self) -> usize {
                self.end
            }
        })*
    };
}

/// Positions of the annotations in index order: ascending `begin`, then
/// descending `end`. Positions rather than references, because a caller that
/// replaces annotations has to name exactly these entries in the store
/// afterwards; one that only reads them maps back over the slice.
pub fn index_order<T: Spanned>(items: &[T]) -> Vec<usize> {
    let mut ordered: Vec<usize> = (0..items.len()).collect();
    ordered.sort_by(|&a, &b| {
        items[a]
            .begin()
            .cmp(&items[b].begin())
            .then(items[b].end().cmp(&items[a].end()))
    });
    ordered
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Token {
    pub begin: usize,
    pub end: usize,
    pub tag: Option<String>,
    pub lemma: Option<String>,
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
    /// In document order, which is also ascending `begin`, and disjoint: each
    /// segment covers a whole text node and the next one starts past the end
    /// of this one. [`crate::util::html_utils::extract`] is the only thing
    /// that builds this list and builds it that way, and the encoding the
    /// analysis cache round-trips preserves the order. The renderer bisects
    /// on it rather than scanning it per enhancement, so a list that was not
    /// in that order would place fewer enhancements — the same outcome as a
    /// segment naming a node the page no longer has.
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

spanned!(
    Token,
    CgToken,
    RelevantText,
    TextSegment,
    Enhancement,
    SentenceAnnotation,
);

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
    /// language the text is written in, which nothing here records.
    pub language: String,
    pub page: PageMap,
    pub tokens: Vec<Token>,
    pub cg_tokens: Vec<CgToken>,
    pub relevant_texts: Vec<RelevantText>,
    pub enhancements: Vec<Enhancement>,
    pub sentences: Vec<SentenceAnnotation>,
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
            .chain(spans_of("tokens", &self.tokens))
            .chain(spans_of("cg_tokens", &self.cg_tokens))
            .chain(spans_of("relevant_texts", &self.relevant_texts))
            .chain(spans_of("enhancements", &self.enhancements))
            .chain(spans_of("sentences", &self.sentences))
            .chain(spans_of("page.segments", &self.page.segments));

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

fn spans_of<T: Spanned>(store: &'static str, items: &[T]) -> impl Iterator<Item = InvalidSpan> {
    items.iter().map(move |item| InvalidSpan {
        store,
        begin: item.begin(),
        end: item.end(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A word whose second character is two bytes wide, so an offset one byte
    /// past its start is inside a character rather than between two.
    const TEXT: &str = "Sámegiella";

    #[test]
    fn a_reading_flattens_both_ways_topics_read_it() {
        let reading: CgReading = ["\"gietta\"", "N", "Sg", "Nom"]
            .iter()
            .map(|tag| (*tag).to_string())
            .collect();

        assert_eq!(
            flatten_reading(&reading, ReadingJoin::TrailingSpace),
            "\"gietta\" N Sg Nom "
        );
        assert_eq!(
            flatten_reading(&reading, ReadingJoin::LeadingPlus),
            "+\"gietta\"+N+Sg+Nom"
        );

        let empty: CgReading = Vec::new();
        assert_eq!(flatten_reading(&empty, ReadingJoin::TrailingSpace), "");
        assert_eq!(flatten_reading(&empty, ReadingJoin::LeadingPlus), "");

        // the buffer a loop reuses keeps what is already in it
        let mut buffer = String::from("kept");
        flatten_reading_into(&mut buffer, &empty, ReadingJoin::LeadingPlus);
        assert_eq!(buffer, "kept");
        buffer.clear();
        flatten_reading_into(&mut buffer, &reading, ReadingJoin::LeadingPlus);
        assert_eq!(buffer, "+\"gietta\"+N+Sg+Nom");
    }

    /// A cohort carrying one reading built from `tags`.
    fn cohort(tags: &[&str]) -> CgToken {
        CgToken {
            begin: 0,
            end: 1,
            readings: vec![tags.iter().map(|tag| (*tag).to_string()).collect()],
        }
    }

    #[test]
    fn the_punctuation_guard_reads_tags_not_lemmas() {
        // what the analyser answers for the punctuation of a real page
        assert!(is_punctuation_cohort(&cohort(&["\".\"", "CLB"])));
        assert!(is_punctuation_cohort(&cohort(&["\",\"", "CLB"])));
        assert!(is_punctuation_cohort(&cohort(&["\"…\"", "CLB"])));
        assert!(is_punctuation_cohort(&cohort(&["\"«\"", "PUNCT", "LEFT"])));
        assert!(is_punctuation_cohort(&cohort(&["\")\"", "PUNCT", "RIGHT"])));
        assert!(is_punctuation_cohort(&cohort(&["\"–\"", "PUNCT"])));

        // a base form whose own letters spell a tag is a word
        assert!(!is_punctuation_cohort(&cohort(&["\"CLB\"", "N", "Sg"])));
        assert!(!is_punctuation_cohort(&cohort(&["\"vuoiPUNCTga\"", "N"])));
        assert!(!is_punctuation_cohort(&cohort(&["\"guovlu\"", "N", "Sg"])));
        assert!(!is_punctuation_cohort(&CgToken::default()));

        // any reading is enough, not only the one a topic would have chosen
        let noun_first = CgToken {
            begin: 0,
            end: 1,
            readings: vec![
                vec!["\"guovlu\"".to_string(), "N".to_string()],
                vec!["\".\"".to_string(), "CLB".to_string()],
            ],
        };
        assert!(is_punctuation_cohort(&noun_first));
    }

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
            reported_store(|doc| doc.page.segments.push(TextSegment {
                begin: 0,
                end: 2,
                ..TextSegment::default()
            })),
            Some("page.segments")
        );
    }
}
