//! An enhancement class that puts a `<span>` around *all* tokens and gives
//! them the hit class as well when they belong to a given POS.
//!
//! Authors: Aleksandar Dimitrov, Adriane Boyd
//! Version: 0.1

use std::collections::HashMap;
use std::sync::LazyLock;

use anyhow::{Result, anyhow};
use regex::Regex;
use tracing::{Level, debug, enabled, trace};

use crate::enhancer::cg_span::{HIT_CLASS, SpanTag, TOKEN_CLASS};
use crate::types::{Document, Enhancement};
use crate::util::enhancer_utils;

/// `.*[^\p{P}].*` as a full match: the token has to carry at least one
/// character outside the Unicode punctuation category, and `.` does not match
/// a line terminator, so a token spanning a line break fails the test.
static NON_PUNCTUATION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:.*[^\p{P}].*)$").expect("non-punctuation pattern"));

// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer]
#[derive(Debug, Clone, Default)]
pub struct TokenEnhancer {
    pub tags: Vec<String>,
    pub use_lemma_filter: bool,
}

impl TokenEnhancer {
    pub fn new() -> Self {
        Self::default()
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.initialize-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.initialize-fn]
    pub fn initialize(&mut self, context: &HashMap<String, String>) -> Result<()> {
        let tags = context
            .get("Tags")
            .ok_or_else(|| anyhow!("NullPointerException: configuration parameter Tags"))?;
        // Java's String.split(",") drops trailing empty fields, but leaves the
        // whole input as the single element when the separator never matches.
        let mut split: Vec<String> = tags.split(',').map(str::to_string).collect();
        if tags.contains(',') {
            while split.last().is_some_and(|tag| tag.is_empty()) {
                split.pop();
            }
        }
        self.tags = split;

        let use_lemma_filter = context.get("UseLemmaFilter").ok_or_else(|| {
            anyhow!("NullPointerException: configuration parameter UseLemmaFilter")
        })?;
        self.use_lemma_filter = use_lemma_filter.parse::<bool>().map_err(|_| {
            anyhow!(
                "configuration parameter UseLemmaFilter is not a boolean: {}",
                use_lemma_filter
            )
        })?;

        Ok(())
    }

    /// Iterate over all tokens and put a span around them. If a token matches
    /// one of the given POS tags, then mark it up as a hit.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn+2]
    pub fn process(&self, cas: &mut Document) -> Result<()> {
        let mut id: i32 = 0;
        debug!("Starting enhancement");

        // the UIMA annotation index hands annotations out in ascending begin,
        // then descending end order
        let mut text_index: Vec<usize> = (0..cas.tokens.len()).collect();
        text_index.sort_by(|left, right| {
            cas.tokens[*left]
                .begin
                .cmp(&cas.tokens[*right].begin)
                .then(cas.tokens[*right].end.cmp(&cas.tokens[*left].end))
        });

        let mut enhancements: Vec<Enhancement> = Vec::new();

        for index in text_index {
            let t = &cas.tokens[index];
            let covered_text = cas.covered_text(t.begin, t.end);
            // enhance all non-punctuation tokens
            if NON_PUNCTUATION.is_match(covered_text) {
                let mut e = Enhancement {
                    begin: t.begin,
                    end: t.end,
                    ..Enhancement::default()
                };

                id += 1;
                let hit;

                match t.tag.as_deref() {
                    None => {
                        debug!("Encountered token with NULL tag");
                        hit = 0;
                    }
                    Some(tag) if self.tags.iter().any(|known| known.as_str() == tag) => {
                        if self.use_lemma_filter {
                            match t.lemma.as_deref() {
                                None => hit = 0,
                                Some("") => hit = 0,
                                Some(_) => hit = 1,
                            }
                        } else {
                            hit = 1;
                        }
                    }
                    Some(_) => {
                        hit = 0;
                    }
                }

                e.relevant = hit == 1;
                let classes: &[&str] = match e.relevant {
                    true => &[TOKEN_CLASS, HIT_CLASS],
                    false => &[TOKEN_CLASS],
                };
                let span_tag = SpanTag::new(enhancer_utils::get_id("WERTi-span", id), classes);
                e.enhance_start = span_tag.start_tag();
                e.enhance_end = span_tag.end_tag().to_string();

                if enabled!(Level::TRACE) {
                    trace!(
                        "Enhanced {} with tag {} with id {}",
                        covered_text,
                        // Java concatenates a null tag as the literal text null
                        t.tag.as_deref().unwrap_or("null"),
                        id
                    );
                }
                enhancements.push(e);
            }
        }

        // the annotations are indexed as they are built; nothing reads the
        // enhancement index while the loop runs
        cas.enhancements.extend(enhancements);

        debug!("Finished enhancement");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Token;

    fn context(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    }

    fn token(begin: usize, end: usize, tag: Option<&str>, lemma: Option<&str>) -> Token {
        Token {
            begin,
            end,
            tag: tag.map(str::to_string),
            lemma: lemma.map(str::to_string),
            ..Token::default()
        }
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.initialize-fn/test]
    #[test]
    fn tags_are_split_on_commas_without_trimming() {
        let mut enhancer = TokenEnhancer::new();

        enhancer
            .initialize(&context(&[
                ("Tags", "in, to ,"),
                ("UseLemmaFilter", "true"),
            ]))
            .unwrap();

        assert_eq!(enhancer.tags, vec!["in", " to "]);
        assert!(enhancer.use_lemma_filter);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.initialize-fn/test]
    #[test]
    fn a_separator_free_tag_string_stays_one_element() {
        let mut enhancer = TokenEnhancer::new();

        enhancer
            .initialize(&context(&[("Tags", ""), ("UseLemmaFilter", "false")]))
            .unwrap();

        assert_eq!(enhancer.tags, vec![""]);
        assert!(!enhancer.use_lemma_filter);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.initialize-fn/test]
    #[test]
    fn interior_empty_fields_survive_trailing_ones_dropped() {
        let mut enhancer = TokenEnhancer::new();

        enhancer
            .initialize(&context(&[("Tags", "a,,b,,"), ("UseLemmaFilter", "false")]))
            .unwrap();

        assert_eq!(enhancer.tags, vec!["a", "", "b"]);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.initialize-fn/test]
    #[test]
    fn missing_or_bad_config_parameters_fail_initialisation() {
        let mut enhancer = TokenEnhancer::new();

        let err = enhancer
            .initialize(&context(&[("UseLemmaFilter", "false")]))
            .unwrap_err();
        assert!(err.to_string().contains("Tags"), "{}", err);

        let err = enhancer
            .initialize(&context(&[("Tags", "in")]))
            .unwrap_err();
        assert!(err.to_string().contains("UseLemmaFilter"), "{}", err);

        let err = enhancer
            .initialize(&context(&[("Tags", "in"), ("UseLemmaFilter", "yes")]))
            .unwrap_err();
        assert!(err.to_string().contains("not a boolean"), "{}", err);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn+2/test]
    #[test]
    fn punctuation_tokens_are_skipped_and_consume_no_id() {
        let mut cas = Document::new(". Mun boran", "sme");
        cas.tokens.push(token(2, 5, Some("Pron"), Some("mun")));
        cas.tokens.push(token(0, 1, Some("CLB"), None));
        cas.tokens.push(token(6, 11, Some("V"), Some("borrat")));
        let enhancer = TokenEnhancer {
            tags: vec!["V".to_string()],
            use_lemma_filter: false,
        };

        enhancer.process(&mut cas).unwrap();

        assert_eq!(cas.enhancements.len(), 2);
        assert_eq!((cas.enhancements[0].begin, cas.enhancements[0].end), (2, 5));
        assert_eq!(
            cas.enhancements[0].enhance_start,
            "<span id=\"WERTi-span-1\" class=\"teaksta-token\">"
        );
        assert!(!cas.enhancements[0].relevant);
        assert_eq!(
            (cas.enhancements[1].begin, cas.enhancements[1].end),
            (6, 11)
        );
        assert_eq!(
            cas.enhancements[1].enhance_start,
            "<span id=\"WERTi-span-2\" class=\"teaksta-token teaksta-hit\">"
        );
        assert!(cas.enhancements[1].relevant);
        assert_eq!(cas.enhancements[1].enhance_end, "</span>");
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn+2/test]
    #[test]
    fn a_token_spanning_two_line_breaks_is_skipped() {
        let mut cas = Document::new("a\nb\nc", "sme");
        cas.tokens.push(token(0, 5, Some("N"), Some("a")));
        let enhancer = TokenEnhancer {
            tags: vec!["N".to_string()],
            use_lemma_filter: false,
        };

        enhancer.process(&mut cas).unwrap();

        assert!(cas.enhancements.is_empty());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn+2/test]
    #[test]
    fn a_single_line_break_still_yields_an_enhancement() {
        let mut cas = Document::new("a\nb", "sme");
        cas.tokens.push(token(0, 3, Some("N"), Some("a")));
        let enhancer = TokenEnhancer {
            tags: vec!["N".to_string()],
            use_lemma_filter: false,
        };

        enhancer.process(&mut cas).unwrap();

        assert_eq!(cas.enhancements.len(), 1);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn+2/test]
    #[test]
    fn a_token_made_only_of_punctuation_is_skipped() {
        let mut cas = Document::new("...", "sme");
        cas.tokens.push(token(0, 3, Some("CLB"), None));
        let enhancer = TokenEnhancer {
            tags: vec!["CLB".to_string()],
            use_lemma_filter: false,
        };

        enhancer.process(&mut cas).unwrap();

        assert!(cas.enhancements.is_empty());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn+2/test]
    #[test]
    fn a_null_tag_is_never_a_hit() {
        let mut cas = Document::new("beana", "sme");
        cas.tokens.push(token(0, 5, None, Some("beana")));
        let enhancer = TokenEnhancer {
            tags: vec!["N".to_string()],
            use_lemma_filter: false,
        };

        enhancer.process(&mut cas).unwrap();

        assert_eq!(cas.enhancements.len(), 1);
        assert!(!cas.enhancements[0].relevant);
        assert_eq!(
            cas.enhancements[0].enhance_start,
            "<span id=\"WERTi-span-1\" class=\"teaksta-token\">"
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn+2/test]
    #[test]
    fn the_lemma_filter_demotes_matches_without_a_lemma() {
        let mut cas = Document::new("aaa bbb ccc", "sme");
        cas.tokens.push(token(0, 3, Some("N"), None));
        cas.tokens.push(token(4, 7, Some("N"), Some("")));
        cas.tokens.push(token(8, 11, Some("N"), Some("ccc")));
        let enhancer = TokenEnhancer {
            tags: vec!["N".to_string()],
            use_lemma_filter: true,
        };

        enhancer.process(&mut cas).unwrap();

        let relevant: Vec<bool> = cas.enhancements.iter().map(|e| e.relevant).collect();
        assert_eq!(relevant, vec![false, false, true]);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn+2/test]
    #[test]
    fn existing_enhancements_are_kept_and_new_ones_appended() {
        let mut cas = Document::new("beana", "sme");
        cas.enhancements.push(Enhancement {
            begin: 0,
            end: 5,
            enhance_start: "<e>".to_string(),
            enhance_end: "</e>".to_string(),
            relevant: true,
        });
        cas.tokens.push(token(0, 5, Some("N"), Some("beana")));
        let enhancer = TokenEnhancer {
            tags: vec!["N".to_string()],
            use_lemma_filter: false,
        };

        enhancer.process(&mut cas).unwrap();

        assert_eq!(cas.enhancements.len(), 2);
        assert_eq!(cas.enhancements[0].enhance_start, "<e>");
        assert_eq!(
            cas.enhancements[1].enhance_start,
            "<span id=\"WERTi-span-1\" class=\"teaksta-token teaksta-hit\">"
        );
        assert_eq!(cas.tokens.len(), 1);
    }
}
