//! Wrapper for the "Giellatekno tokenizer" (tokenisation that is specially
//! adapted to North Sámi).

use anyhow::{Result, anyhow, bail};
use regex::Regex;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, MutexGuard};
use tracing::{debug, info, trace};

use crate::morpho::MorphoPipeline;
use crate::types::{Document, RelevantText, Token};

/// Annotation index order: ascending `begin`, then descending `end`.
fn index_order(spans: &[RelevantText]) -> Vec<&RelevantText> {
    let mut ordered: Vec<&RelevantText> = spans.iter().collect();
    ordered.sort_by(|a, b| a.begin.cmp(&b.begin).then(b.end.cmp(&a.end)));
    ordered
}

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

/// Index of the first occurrence of `needle` at or after `from`, or `-1`.
/// Mirrors `String.indexOf(String, int)`: a negative `from` is clamped to
/// zero, and an empty needle answers `min(from, len)`.
fn index_of_from(haystack: &str, needle: &str, from: i64) -> i64 {
    let from = from.max(0) as usize;

    if from >= haystack.len() {
        if needle.is_empty() {
            return haystack.len() as i64;
        }
        if from > haystack.len() {
            return -1;
        }
    }

    if needle.is_empty() {
        return from as i64;
    }

    let hay = haystack.as_bytes();
    let ned = needle.as_bytes();

    if ned.len() > hay.len() {
        return -1;
    }

    // UTF-8 is self-synchronising, so a byte-wise scan can only ever land on
    // a character boundary and matches the character-index search.
    for i in from..=(hay.len() - ned.len()) {
        if &hay[i..i + ned.len()] == ned {
            return i as i64;
        }
    }

    -1
}

/// Byte index of the character immediately preceding `idx`, or `-1` when
/// `idx` is the start of the string.
fn prev_char_index(haystack: &str, idx: usize) -> i64 {
    match haystack[..idx].chars().next_back() {
        Some(c) => (idx - c.len_utf8()) as i64,
        None => -1,
    }
}

/// The document text with everything outside the relevant spans blanked to
/// spaces, at byte-for-byte identical offsets.
fn mask_to_relevant_text(text: &str, spans: &[RelevantText]) -> Result<String> {
    let mut rtext = vec![b' '; text.len()];

    for t in index_order(spans) {
        let covered = text.get(t.begin..t.end).ok_or_else(|| {
            anyhow!(
                "relevant text {}..{} is not within the document",
                t.begin,
                t.end
            )
        })?;
        rtext[t.begin..t.end].copy_from_slice(covered.as_bytes());
    }

    Ok(String::from_utf8(rtext)?)
}

/// A token is annotated only when it holds at least one character outside the
/// Unicode separator category, which drops whitespace-only tokens including
/// non-breaking spaces.
static NON_SEPARATOR_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:.*?[^\p{Z}].*)$").expect("non separator pattern"));

/// The deployment locations the Java concatenated the preprocessing command
/// from, kept only because that command reaches the log.
const TOOLS_DIR: &str = "/opt/smi/sme/bin/";
const ABBR_DIR: &str = "/opt/smi/sme/bin/";

static PREPROCESS_CMD: LazyLock<String> =
    LazyLock::new(|| format!("{}preprocess --abbr={}abbr.txt", TOOLS_DIR, ABBR_DIR));

/// Language code to tokeniser. Replaced wholesale on every initialisation,
/// so the last initialised instance owns the registry.
static TOKENIZERS: LazyLock<Mutex<HashMap<String, &'static MorphoPipeline>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn tokenizers() -> MutexGuard<'static, HashMap<String, &'static MorphoPipeline>> {
    TOKENIZERS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer]
#[derive(Debug, Clone, Copy, Default)]
pub struct GiellateknoTokenizer;

impl GiellateknoTokenizer {
    pub fn new() -> Self {
        GiellateknoTokenizer
    }

    /// The registry built here is never consulted by [`Self::process`], which
    /// tokenises through the morphological pipeline instead.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.initialize-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.initialize-fn]
    pub fn initialize(&mut self) -> Result<()> {
        let mut registry = HashMap::new();
        registry.insert("en".to_string(), MorphoPipeline::shared());
        *tokenizers() = registry;

        Ok(())
    }

    /// Tokenises the relevant portions of the document and maps the
    /// one-token-per-line result back onto offsets in the document. The
    /// stdout-consumer plumbing the external `preprocess` command needed is
    /// subsumed by the morphological pipeline seam.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.process-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.process-fn]
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
        info!("Starting token annotation");

        // put relevant text spans in their proper positions in an empty document
        let text_string = mask_to_relevant_text(&jcas.text, &jcas.relevant_texts)?;

        let file_path = "/tmp/konteakstaInput.txt";

        let _lang = jcas.language.clone();

        let tokenisation_pipeline = format!("/bin/cat \"{}\" | {}", file_path, *PREPROCESS_CMD);
        info!("Preprocessing command: {}", tokenisation_pipeline);

        // Every line of the tokeniser output carries a trailing newline, as
        // the stdout consumer appended one per line read.
        let tokenised_text: String = match MorphoPipeline::shared().tokenize(&text_string) {
            Ok(lines) => lines.iter().map(|line| format!("{line}\n")).collect(),
            Err(e) => {
                println!("{}", e);
                String::new()
            }
        };

        info!("tokenised_text={}", tokenised_text);

        let tokens = split_lines(&tokenised_text);

        let mut skew: i64 = 0;

        for token in tokens {
            // include all tokens that don't consist of whitespace, i.e., prevent
            // unicode non-breaking space from becoming a token
            info!("next token: {}", token);
            let mut token_start = index_of_from(&text_string, token, skew);
            info!("Token {}}} starts at {}", token, token_start);

            if token_start == -1 {
                // Handle the hyphenated words that are "repaired" by preprocess
                // and thus not found in the original text.
                let hyphen = index_of_from(&text_string, "-", skew);
                if hyphen != -1 {
                    let from = skew.max(0) as usize;
                    // the character immediately preceding the hyphen is dropped
                    let to = prev_char_index(&text_string, hyphen as usize);
                    let candidate = if to < 0 {
                        None
                    } else {
                        text_string.get(from..to as usize)
                    };
                    let syllable = candidate
                        .ok_or_else(|| anyhow!("string index out of range: {}..{}", from, to))?;

                    // search the part of the word preceding the hyphen instead
                    // of the whole word
                    token_start = index_of_from(&text_string, syllable, skew);
                    skew = token_start + token.len() as i64 + 1; // 1 = length of the hyphen
                } else {
                    // restarts the scan at the head of the document, so later
                    // tokens can match at earlier, wrong positions
                    skew = 0;
                    continue;
                }
            } else {
                skew = token_start + token.len() as i64; // This is the normal case!
            }

            if NON_SEPARATOR_PATTERN.is_match(token) {
                // The repair branch never re-checks the search, so a failed
                // lookup would otherwise annotate from a negative offset.
                if token_start < 0 {
                    bail!("token {:?} resolved to a negative begin offset", token);
                }

                let start = token_start as usize;
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
        let mut doc = Document::new(text, "sme");
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

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.initialize-fn/test]
    #[test]
    fn initialize_registers_english_and_process_never_consults_it() {
        GiellateknoTokenizer::new().initialize().unwrap();

        {
            let registry = tokenizers();
            assert_eq!(registry.len(), 1);
            assert!(registry.contains_key("en"));
            assert!(!registry.contains_key("de"));
        }

        let mut doc = document("mun boran", &[(0, 9)]);
        doc.language = "de".to_string();
        assert!(GiellateknoTokenizer::new().process(&mut doc).is_ok());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.process-fn/test]
    #[test]
    fn process_rejects_a_relevant_span_outside_the_document() {
        let mut doc = document("mun", &[(0, 99)]);

        let err = GiellateknoTokenizer::new()
            .process(&mut doc)
            .expect_err("the relevant span runs off the end of the document");

        assert!(err.to_string().contains("is not within the document"));
        assert!(doc.tokens.is_empty());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.process-fn/test]
    #[test]
    fn process_annotates_only_spans_slicing_document_text() {
        let mut doc = document("mun boran guoli.", &[(0, 16)]);

        GiellateknoTokenizer::new().process(&mut doc).unwrap();

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

        GiellateknoTokenizer::new().process(&mut doc).unwrap();

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
        GiellateknoTokenizer::new().process(&mut doc).unwrap();

        assert!(doc.tokens.is_empty());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.is-done-fn/test]
    #[test]
    fn process_completes_with_or_without_pipeline_output() {
        let mut doc = document("mun boran guoli.", &[(0, 16)]);

        assert!(GiellateknoTokenizer::new().process(&mut doc).is_ok());
    }

    #[test]
    fn masking_blanks_everything_outside_the_relevant_spans() {
        let masked = mask_to_relevant_text("Mun boran guoli.", &[relevant(4, 9)]).unwrap();

        assert_eq!(masked, "    boran       ");
        assert_eq!(masked.len(), "Mun boran guoli.".len());
    }

    #[test]
    fn masking_keeps_multibyte_offsets_intact() {
        let text = "áigi guolli";
        let masked = mask_to_relevant_text(text, &[relevant(0, 5)]).unwrap();

        assert_eq!(masked, "áigi       ");
        assert_eq!(masked.len(), text.len());
    }

    #[test]
    fn masking_rejects_a_span_that_splits_a_character() {
        let err = mask_to_relevant_text("áigi", &[relevant(0, 1)]).unwrap_err();

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
    fn searching_from_cursor_matches_java_index_of() {
        assert_eq!(index_of_from("guolli guolli", "guolli", 0), 0);
        assert_eq!(index_of_from("guolli guolli", "guolli", 1), 7);
        assert_eq!(index_of_from("guolli", "boran", 0), -1);
        assert_eq!(index_of_from("guolli", "guollit", 0), -1);
        assert_eq!(index_of_from("áigi", "igi", 0), 2);
        assert_eq!(index_of_from("guolli", "g", -5), 0);
        assert_eq!(index_of_from("guolli", "", 3), 3);
        assert_eq!(index_of_from("guolli", "", 99), 6);
        assert_eq!(index_of_from("guolli", "g", 99), -1);
    }

    #[test]
    fn character_before_index_found_by_its_width() {
        assert_eq!(prev_char_index("mun-boran", 3), 2);
        assert_eq!(prev_char_index("mun-boran", 0), -1);
        assert_eq!(prev_char_index("á-boran", 2), 0);
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
