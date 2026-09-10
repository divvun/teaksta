//! An enhancement class that puts a `<span>` around *all* tokens and gives
//! them the hit class as well when they belong to a given POS.
//!
//! This is what makes the click exercise an exercise: the topic's own
//! enhancer marks only its hits, so without a span around every other word
//! there is nothing for the learner to pick wrongly. The spans written here
//! are the decoys, and they are marked irrelevant so that only the click
//! exercise carries them.
//!
//! Authors: Aleksandar Dimitrov, Adriane Boyd
//! Version: 0.1

use std::collections::HashMap;
use std::sync::LazyLock;

use anyhow::{Result, anyhow};
use regex::Regex;
use tracing::{Level, debug, enabled, trace, warn};

use crate::enhancer::cg_span::{HIT_CLASS, SpanTag, TOKEN_CLASS};
use crate::types::{Document, Enhancement, covered_text};
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

    /// Every annotation the UIMA index over `Token.type` would hand out, in
    /// the order it would hand them out: ascending begin, then descending
    /// end.
    ///
    /// The index is polymorphic, so it holds the `CGToken`s as well as the
    /// plain `Token`s — `CGToken extends Token` in the type system. That
    /// matters because the CG annotator takes every token it consumed back
    /// out of the index and puts the CG token carrying its analysis in its
    /// place, so by the time a post-processor runs the plain tokens are
    /// gone and the CG tokens are all there is. A CG token carries no `tag`
    /// and no `lemma`: the CG analysis lives in its readings, and the two
    /// features the Java inherited from `Token` were only ever copied from
    /// the token it replaced.
    fn annotation_index(cas: &Document) -> Vec<Candidate<'_>> {
        let mut index: Vec<Candidate<'_>> = cas
            .tokens
            .iter()
            .map(|t| Candidate {
                begin: t.begin,
                end: t.end,
                tag: t.tag.as_deref(),
                lemma: t.lemma.as_deref(),
            })
            .chain(cas.cg_tokens.iter().map(|t| Candidate {
                begin: t.begin,
                end: t.end,
                tag: None,
                lemma: None,
            }))
            .collect();
        index.sort_by(|left, right| left.begin.cmp(&right.begin).then(right.end.cmp(&left.end)));
        index
    }

    /// Iterate over all tokens and put a span around them. If a token matches
    /// one of the given POS tags, then mark it up as a hit.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn+3]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn+3]
    pub fn process(&self, cas: &mut Document) -> Result<()> {
        let mut id: i32 = 0;
        debug!("Starting enhancement");

        let text_index = Self::annotation_index(cas);
        let mut enhancements: Vec<Enhancement> = Vec::new();

        for t in text_index {
            let covered_text = match covered_text(&cas.text, t.begin, t.end) {
                Ok(covered) => covered,
                // A token the document text cannot be read at covers nothing
                // to wrap, so there is no enhancement to make from it.
                Err(unreadable) => {
                    warn!("Skipping token: {}", unreadable);
                    continue;
                }
            };
            // enhance all non-punctuation tokens
            if NON_PUNCTUATION.is_match(covered_text) {
                let mut e = Enhancement {
                    begin: t.begin,
                    end: t.end,
                    ..Enhancement::default()
                };

                id += 1;
                let hit;

                match t.tag {
                    None => {
                        debug!("Encountered token with NULL tag");
                        hit = 0;
                    }
                    Some(tag) if self.tags.iter().any(|known| known.as_str() == tag) => {
                        if self.use_lemma_filter {
                            match t.lemma {
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
                let span_tag = SpanTag::new(enhancer_utils::get_id("teaksta-span", id), classes);
                e.enhance_start = span_tag.start_tag();
                e.enhance_end = span_tag.end_tag().to_string();

                if enabled!(Level::TRACE) {
                    trace!(
                        "Enhanced {} with tag {} with id {}",
                        covered_text,
                        // Java concatenates a null tag as the literal text null
                        t.tag.unwrap_or("null"),
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

/// One annotation of the `Token.type` index, read for what this enhancer
/// asks of it. Holding the two features by reference keeps the index a view
/// over the document rather than a copy of it.
struct Candidate<'a> {
    begin: usize,
    end: usize,
    tag: Option<&'a str>,
    lemma: Option<&'a str>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CgToken, Token};

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

    /// A CG token as the CG annotator leaves it: the analysis is in the
    /// readings, and it carries neither of the two features inherited from
    /// `Token`.
    fn cg_token(begin: usize, end: usize) -> CgToken {
        CgToken {
            begin,
            end,
            ..CgToken::default()
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

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn+3/test]
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
            "<span id=\"teaksta-span-1\" class=\"teaksta-token\">"
        );
        assert!(!cas.enhancements[0].relevant);
        assert_eq!(
            (cas.enhancements[1].begin, cas.enhancements[1].end),
            (6, 11)
        );
        assert_eq!(
            cas.enhancements[1].enhance_start,
            "<span id=\"teaksta-span-2\" class=\"teaksta-token teaksta-hit\">"
        );
        assert!(cas.enhancements[1].relevant);
        assert_eq!(cas.enhancements[1].enhance_end, "</span>");
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn+3/test]
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

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn+3/test]
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

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn+3/test]
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

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn+3/test]
    #[test]
    fn an_unreadable_token_span_is_skipped() {
        let mut cas = Document::new("Sámegiella", "sme");
        // Inside the two-byte `á`, past the end, and readable.
        cas.tokens.push(token(0, 2, Some("N"), None));
        cas.tokens.push(token(0, 99, Some("N"), None));
        cas.tokens.push(token(3, 10, Some("N"), None));
        let enhancer = TokenEnhancer {
            tags: vec!["N".to_string()],
            use_lemma_filter: false,
        };

        enhancer.process(&mut cas).unwrap();

        assert_eq!(cas.enhancements.len(), 1);
        assert_eq!(
            (cas.enhancements[0].begin, cas.enhancements[0].end),
            (3, 10)
        );
        // The skipped tokens consumed no id.
        assert_eq!(
            cas.enhancements[0].enhance_start,
            "<span id=\"teaksta-span-1\" class=\"teaksta-token teaksta-hit\">"
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn+3/test]
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
            "<span id=\"teaksta-span-1\" class=\"teaksta-token\">"
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn+3/test]
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

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn+3/test]
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
            "<span id=\"teaksta-span-1\" class=\"teaksta-token teaksta-hit\">"
        );
        assert_eq!(cas.tokens.len(), 1);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn+3/test]
    #[test]
    fn the_cg_tokens_left_behind_are_enhanced() {
        // what the CAS looks like once the CG annotator has swapped every
        // token it consumed for the CG token carrying its analysis
        let mut cas = Document::new("Mun oidnen viesu ikte.", "sme");
        cas.cg_tokens.push(cg_token(0, 3));
        cas.cg_tokens.push(cg_token(4, 10));
        cas.cg_tokens.push(cg_token(11, 16));
        cas.cg_tokens.push(cg_token(17, 21));
        cas.cg_tokens.push(cg_token(21, 22));
        let enhancer = TokenEnhancer {
            tags: vec!["N".to_string()],
            use_lemma_filter: false,
        };

        enhancer.process(&mut cas).unwrap();

        // the full stop is punctuation and takes no span with it
        let spans: Vec<(usize, usize)> =
            cas.enhancements.iter().map(|e| (e.begin, e.end)).collect();
        assert_eq!(spans, vec![(0, 3), (4, 10), (11, 16), (17, 21)]);
        assert!(cas.enhancements.iter().all(|e| !e.relevant));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn+3/test]
    #[test]
    fn a_cg_token_is_never_a_hit() {
        // a CG token carries no tag of its own, so the configured tag list
        // has nothing to match and every span it writes is a decoy
        let mut cas = Document::new("beana", "sme");
        cas.cg_tokens.push(cg_token(0, 5));
        let enhancer = TokenEnhancer {
            tags: vec!["N".to_string(), "beana".to_string()],
            use_lemma_filter: false,
        };

        enhancer.process(&mut cas).unwrap();

        assert_eq!(cas.enhancements.len(), 1);
        assert!(!cas.enhancements[0].relevant);
        assert_eq!(
            cas.enhancements[0].enhance_start,
            "<span id=\"teaksta-span-1\" class=\"teaksta-token\">"
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn+3/test]
    #[test]
    fn both_kinds_of_token_are_read_in_order() {
        let mut cas = Document::new("aaa bbb ccc", "sme");
        cas.cg_tokens.push(cg_token(8, 11));
        cas.tokens.push(token(0, 3, Some("N"), Some("aaa")));
        cas.cg_tokens.push(cg_token(4, 7));
        let enhancer = TokenEnhancer {
            tags: vec!["N".to_string()],
            use_lemma_filter: false,
        };

        enhancer.process(&mut cas).unwrap();

        let spans: Vec<(usize, usize)> =
            cas.enhancements.iter().map(|e| (e.begin, e.end)).collect();
        assert_eq!(spans, vec![(0, 3), (4, 7), (8, 11)]);
        // the plain token still carries the tag it was given
        let relevant: Vec<bool> = cas.enhancements.iter().map(|e| e.relevant).collect();
        assert_eq!(relevant, vec![true, false, false]);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn+3/test]
    #[test]
    fn a_span_naming_no_text_is_skipped() {
        // begin and end past the end of the text, and a cut through the
        // middle of a two-byte character
        let mut cas = Document::new("á beana", "sme");
        cas.cg_tokens.push(cg_token(0, 1));
        cas.cg_tokens.push(cg_token(3, 8));
        cas.cg_tokens.push(cg_token(3, 40));
        let enhancer = TokenEnhancer::default();

        enhancer.process(&mut cas).unwrap();

        assert_eq!(cas.enhancements.len(), 1);
        assert_eq!((cas.enhancements[0].begin, cas.enhancements[0].end), (3, 8));
        assert_eq!(
            cas.enhancements[0].enhance_start,
            "<span id=\"teaksta-span-1\" class=\"teaksta-token\">"
        );
    }
}
