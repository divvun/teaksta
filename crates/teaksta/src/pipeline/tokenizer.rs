//! Wrapper for the "Giellatekno tokenizer" (tokenisation that is specially
//! adapted to North Sámi).

use anyhow::{Context as _, Result, anyhow};
use regex::Regex;
use std::sync::LazyLock;
use tracing::{debug, trace};

use crate::morpho::MorphoPipeline;
use crate::pipeline::mask_to_spans;
use crate::types::{Document, Token};

/// Splits on `'\n'` the way `String.split` does: an input without a single
/// separator yields the whole input, and trailing empty segments are dropped.
fn split_lines(input: &str) -> Vec<&str> {
    if !input.contains('\n') {
        return vec![input];
    }

    let mut parts: Vec<&str> = input.split('\n').collect();
    while parts.last().is_some_and(|part| part.is_empty()) {
        parts.pop();
    }
    parts
}

/// Index of the first occurrence of `needle` at or after the byte offset
/// `from`, or `None`. An empty needle answers `min(from, len)`.
fn index_of_from(haystack: &str, needle: &str, from: usize) -> Option<usize> {
    if from >= haystack.len() {
        return needle.is_empty().then_some(haystack.len());
    }
    if needle.is_empty() {
        return Some(from);
    }

    let hay = haystack.as_bytes();
    let ned = needle.as_bytes();
    if ned.len() > hay.len() {
        return None;
    }

    // UTF-8 is self-synchronising, so a byte-wise scan can only ever match at
    // a character boundary and agrees with a character-index search. The scan
    // is written out rather than deferred to `str::find` because the cursor
    // the repair branch advances need not land on one.
    (from..=(hay.len() - ned.len())).find(|&i| &hay[i..i + ned.len()] == ned)
}

/// Byte index of the character immediately preceding `idx`, or `None` when
/// `idx` is the start of the string.
fn prev_char_index(haystack: &str, idx: usize) -> Option<usize> {
    haystack[..idx]
        .chars()
        .next_back()
        .map(|c| idx - c.len_utf8())
}

/// A token is annotated only when it holds at least one character outside the
/// Unicode separator category, which drops whitespace-only tokens including
/// non-breaking spaces.
static NON_SEPARATOR_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:.*?[^\p{Z}].*)$").expect("non separator pattern"));

// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer]
// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.initialize-fn+1]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.initialize-fn+1]
#[derive(Debug, Clone, Copy, Default)]
pub struct GiellateknoTokenizer;

impl GiellateknoTokenizer {
    /// Tokenises the relevant portions of the document and maps the
    /// one-token-per-line result back onto offsets in the document. The
    /// stdout-consumer plumbing the external `preprocess` command needed is
    /// subsumed by the morphological pipeline seam.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.process-fn+3]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.process-fn+3]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.ext-command-consume2-string-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.ext-command-consume2-string-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.run-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.run-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.is-done-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.is-done-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.get-buffer-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.get-buffer-fn]
    pub fn process(&self, jcas: &mut Document) -> Result<()> {
        debug!("Starting token annotation");

        // put relevant text spans in their proper positions in an empty document
        let text_string = mask_to_spans(&jcas.text, &jcas.relevant_texts)?;

        // Every line of the tokeniser output carries a trailing newline, as
        // the stdout consumer appended one per line read. A tokenisation that
        // failed is the request's failure: the alternative is an empty token
        // list, which reaches the learner as a page with no exercise on it
        // and no indication that anything went wrong.
        let tokenised_text: String = MorphoPipeline::shared()
            .tokenize(&text_string)
            .context("tokenising the relevant text")?
            .iter()
            .map(|line| format!("{line}\n"))
            .collect();

        trace!("tokenised_text={}", tokenised_text);

        let tokens = split_lines(&tokenised_text);

        let mut skew: usize = 0;

        for token in tokens {
            // include all tokens that don't consist of whitespace, i.e., prevent
            // unicode non-breaking space from becoming a token
            let start = match index_of_from(&text_string, token, skew) {
                Some(start) => {
                    skew = start + token.len(); // This is the normal case!
                    start
                }
                // Handle the hyphenated words that are "repaired" by preprocess
                // and thus not found in the original text.
                None => {
                    let Some(hyphen) = index_of_from(&text_string, "-", skew) else {
                        // restarts the scan at the head of the document, so
                        // later tokens can match at earlier, wrong positions
                        skew = 0;
                        continue;
                    };

                    // the character immediately preceding the hyphen is dropped
                    let syllable = prev_char_index(&text_string, hyphen)
                        .and_then(|to| text_string.get(skew..to))
                        .ok_or_else(|| {
                            anyhow!("string index out of range: {}..{}", skew, hyphen)
                        })?;

                    // search the part of the word preceding the hyphen instead
                    // of the whole word. The syllable is the stretch of the
                    // text at `skew`, so the search answers `skew` itself.
                    let start = index_of_from(&text_string, syllable, skew)
                        .ok_or_else(|| anyhow!("syllable {syllable:?} is not at {skew}"))?;
                    skew = start + token.len() + 1; // 1 = length of the hyphen
                    start
                }
            };
            trace!("Token {} starts at {}", token, start);

            if NON_SEPARATOR_PATTERN.is_match(token) {
                let mut t = Token {
                    begin: start,
                    end: start + token.len(),
                    ..Default::default()
                };

                let covered = jcas
                    .text
                    .get(t.begin..t.end)
                    .ok_or_else(|| anyhow!("token span {}..{} is out of range", t.begin, t.end))?
                    .to_string();

                let tlen = covered.chars().count();
                let first = covered.chars().next();
                let last = covered.chars().last();

                // check for leading or trailing unicode quotes or possessives
                // that the tokenisation doesn't separate from the adjacent words
                if tlen > 1 && matches!(first, Some('\u{2018}' | '\u{201C}')) {
                    let quote_len = first.map(char::len_utf8).unwrap_or(0);
                    t.begin = start + quote_len;

                    jcas.tokens.push(Token {
                        begin: start,
                        end: start + quote_len,
                        ..Default::default()
                    });
                } else if tlen > 1 && matches!(last, Some('\u{2019}' | '\u{201D}')) {
                    let quote_len = last.map(char::len_utf8).unwrap_or(0);
                    t.end = start + token.len() - quote_len;

                    jcas.tokens.push(Token {
                        begin: start + token.len() - quote_len,
                        end: start + token.len(),
                        ..Default::default()
                    });
                } else if tlen > 2 && ends_with_possessive(&covered) {
                    let s_len = 's'.len_utf8();
                    let possessive_len = '\u{2019}'.len_utf8() + s_len;
                    t.end = start + token.len() - possessive_len;

                    jcas.tokens.push(Token {
                        begin: start + token.len() - possessive_len,
                        end: start + token.len() - s_len,
                        ..Default::default()
                    });

                    jcas.tokens.push(Token {
                        begin: start + token.len() - s_len,
                        end: start + token.len(),
                        ..Default::default()
                    });
                }

                let (begin, end) = (t.begin, t.end);
                jcas.tokens.push(t);
                trace!(
                    "Token: {} {} {}",
                    begin,
                    jcas.text.get(begin..end).unwrap_or(""),
                    end
                );
            }
        }

        debug!("Finished token annotation");
        Ok(())
    }
}

/// Whether the covered text ends in the possessive `’s`.
fn ends_with_possessive(covered: &str) -> bool {
    let mut chars = covered.chars().rev();
    matches!((chars.next(), chars.next()), (Some('s'), Some('\u{2019}')))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PIPELINE_LANGUAGE, RelevantText};

    fn relevant(begin: usize, end: usize) -> RelevantText {
        RelevantText {
            begin,
            end,
            ..Default::default()
        }
    }

    /// The document text handed to the tokeniser, with the given spans marked
    /// relevant.
    fn document(text: &str, spans: &[(usize, usize)]) -> Document {
        let mut doc = Document::new(text, PIPELINE_LANGUAGE);
        for (begin, end) in spans {
            doc.relevant_texts.push(relevant(*begin, *end));
        }
        doc
    }

    /// The buffer the stdout consumer accumulates: one trailing newline per
    /// line read.
    fn drained(lines: &[&str]) -> String {
        lines.iter().map(|line| format!("{line}\n")).collect()
    }

    /// `process` over a document, for the tests that are about what the pass
    /// does with the tokeniser's output rather than about the tokeniser. With
    /// the models in place the call succeeds; without them it names the step
    /// that failed, which is the only other outcome the pass has.
    fn processed(doc: &mut Document) -> bool {
        match GiellateknoTokenizer.process(doc) {
            Ok(()) => true,
            Err(e) => {
                let message = format!("{e:#}");
                assert!(
                    message.contains("tokenising the relevant text"),
                    "unexpected error: {message}"
                );
                false
            }
        }
    }

    /// The registry the Java built here keyed a tokeniser by language and was
    /// never read back, so the stage has no initialisation step at all and
    /// the document's language reaches nothing the pass does.
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.initialize-fn+1/test]
    #[test]
    fn the_pass_needs_no_setup_and_no_language() {
        let mut registered = document("mun boran", &[(0, 9)]);
        let mut foreign = document("mun boran", &[(0, 9)]);
        foreign.language = "de".to_string();

        assert_eq!(processed(&mut registered), processed(&mut foreign));
        assert_eq!(registered.tokens.len(), foreign.tokens.len());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.process-fn+3/test]
    #[test]
    fn process_rejects_a_relevant_span_outside_the_document() {
        let mut doc = document("mun", &[(0, 99)]);

        let err = GiellateknoTokenizer
            .process(&mut doc)
            .expect_err("the relevant span runs off the end of the document");

        assert!(err.to_string().contains("is not within the document"));
        assert!(doc.tokens.is_empty());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.process-fn+3/test]
    #[test]
    fn process_annotates_only_spans_slicing_document_text() {
        let mut doc = document("mun boran guoli.", &[(0, 16)]);

        processed(&mut doc);

        for t in &doc.tokens {
            assert!(t.begin <= t.end);
            let covered = doc
                .text
                .get(t.begin..t.end)
                .expect("token spans a valid slice of the document");
            assert!(!covered.is_empty());
            assert!(!covered.contains('\n'));
        }
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.ext-command-consume2-string-fn/test]
    #[test]
    fn process_accumulates_nothing_without_relevant_text() {
        let mut doc = document("mun boran guoli.", &[]);

        processed(&mut doc);

        assert!(doc.tokens.is_empty());
        assert_eq!(doc.text, "mun boran guoli.");
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.run-fn/test]
    #[test]
    fn drained_buffer_ends_lines_with_newlines_and_resplits() {
        let buffer = drained(&["mun", "boran", "guoli", "."]);

        assert_eq!(buffer, "mun\nboran\nguoli\n.\n");
        assert!(buffer.ends_with('\n'));
        assert_eq!(split_lines(&buffer), vec!["mun", "boran", "guoli", "."]);
        assert_eq!(
            split_lines(&drained(&["mun", "", "boran"])),
            ["mun", "", "boran"]
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.get-buffer-fn/test]
    #[test]
    fn empty_buffer_yields_one_never_annotated_token() {
        assert_eq!(drained(&[]), "");
        assert_eq!(split_lines(""), vec![""]);
        assert!(!NON_SEPARATOR_PATTERN.is_match(""));

        let mut doc = document("", &[]);
        processed(&mut doc);

        assert!(doc.tokens.is_empty());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.is-done-fn/test]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.process-fn+3/test]
    #[test]
    fn a_failed_tokenisation_is_reported_not_swallowed() {
        let mut doc = document("mun boran guoli.", &[(0, 16)]);

        match GiellateknoTokenizer.process(&mut doc) {
            // with the models in place the relevant text is tokenised
            Ok(()) => assert!(!doc.tokens.is_empty()),
            // and without them the pass reports the failure, rather than
            // completing over an empty token list and handing the request an
            // exercise with no exercises in it
            Err(e) => {
                let message = format!("{e:#}");
                assert!(
                    message.contains("tokenising the relevant text"),
                    "unexpected error: {message}"
                );
                assert!(doc.tokens.is_empty());
            }
        }
    }

    #[test]
    fn masking_blanks_everything_outside_the_relevant_spans() {
        let masked = mask_to_spans("Mun boran guoli.", &[relevant(4, 9)]).unwrap();

        assert_eq!(masked, "    boran       ");
        assert_eq!(masked.len(), "Mun boran guoli.".len());
    }

    #[test]
    fn masking_keeps_multibyte_offsets_intact() {
        let text = "áigi guolli";
        let masked = mask_to_spans(text, &[relevant(0, 5)]).unwrap();

        assert_eq!(masked, "áigi       ");
        assert_eq!(masked.len(), text.len());
    }

    #[test]
    fn masking_rejects_a_span_that_splits_a_character() {
        let err = mask_to_spans("áigi", &[relevant(0, 1)]).unwrap_err();

        assert!(err.to_string().contains("is not within the document"));
    }

    #[test]
    fn splitting_lines_drops_only_trailing_empty_segments() {
        assert_eq!(split_lines("mun"), vec!["mun"]);
        assert_eq!(split_lines("mun\nboran\n\n"), vec!["mun", "boran"]);
        assert_eq!(split_lines("\n\nmun"), vec!["", "", "mun"]);
        assert!(split_lines("\n\n").is_empty());
    }

    #[test]
    fn searching_from_cursor_finds_the_next_occurrence() {
        assert_eq!(index_of_from("guolli guolli", "guolli", 0), Some(0));
        assert_eq!(index_of_from("guolli guolli", "guolli", 1), Some(7));
        assert_eq!(index_of_from("guolli", "boran", 0), None);
        assert_eq!(index_of_from("guolli", "guollit", 0), None);
        assert_eq!(index_of_from("áigi", "igi", 0), Some(2));
        assert_eq!(index_of_from("guolli", "", 3), Some(3));
        assert_eq!(index_of_from("guolli", "", 99), Some(6));
        assert_eq!(index_of_from("guolli", "g", 99), None);
        // A cursor the repair branch left inside a character still searches
        // rather than panicking, because the scan is byte-wise.
        assert_eq!(index_of_from("áigi", "igi", 1), Some(2));
    }

    #[test]
    fn character_before_index_found_by_its_width() {
        assert_eq!(prev_char_index("mun-boran", 3), Some(2));
        assert_eq!(prev_char_index("mun-boran", 0), None);
        assert_eq!(prev_char_index("á-boran", 2), Some(0));
    }

    #[test]
    fn only_tokens_holding_non_separator_chars_annotated() {
        assert!(NON_SEPARATOR_PATTERN.is_match("guolli"));
        assert!(NON_SEPARATOR_PATTERN.is_match(" guolli "));
        assert!(NON_SEPARATOR_PATTERN.is_match("."));
        assert!(NON_SEPARATOR_PATTERN.is_match("\t"));
        assert!(!NON_SEPARATOR_PATTERN.is_match(" "));
        assert!(!NON_SEPARATOR_PATTERN.is_match("\u{00a0}"));
        assert!(!NON_SEPARATOR_PATTERN.is_match(""));
    }

    #[test]
    fn possessive_wants_right_single_quote_before_s() {
        assert!(ends_with_possessive("boy\u{2019}s"));
        assert!(!ends_with_possessive("boy's"));
        assert!(!ends_with_possessive("boy\u{2019}S"));
        assert!(!ends_with_possessive("guolli"));
        assert!(!ends_with_possessive("s"));
    }
}
