//! An enhancement class that puts WERTi-`<span>`s around *all* tokens and
//! optionally gives them the attribute 'wertiviewhit' when they belong to a
//! given POS.
//!
//! Authors: Aleksandar Dimitrov, Adriane Boyd
//! Version: 0.1

use std::collections::HashMap;
use std::sync::LazyLock;

use anyhow::{Result, anyhow};
use regex::Regex;
use tracing::{Level, debug, enabled, trace};

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
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.token-enhancer.token-enhancer.process-fn]
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

                let mut hitclass = "";
                if hit == 1 {
                    hitclass = "wertiviewhit";
                    e.relevant = true;
                } else {
                    e.relevant = false;
                }
                e.enhance_start = format!(
                    "<span id=\"{}\" class=\"wertiviewtoken {}\">",
                    enhancer_utils::get_id("WERTi-span", id),
                    hitclass
                );
                e.enhance_end = "</span>".to_string();

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
