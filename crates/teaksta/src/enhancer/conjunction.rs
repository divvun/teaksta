//! Use the TAG-B TAG-I sequences resulting from the CG3 analysis with
//! [`crate::pipeline::vislcg3`] to enhance spans corresponding to the tags
//! specified by the activity as conjunction tags.
//!
//! Authors: Niels Ott?, Adriane Boyd, Heli Uibo

use std::collections::HashMap;

use anyhow::{Result, anyhow};
use tracing::{debug, info};

use crate::enhancer::cg_span::{SpanTag, TOKEN_CLASS};
use crate::server::api::Mode;
use crate::types::{CgReading, CgToken, Document, Enhancement};
use crate::util::enhancer_utils;

// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer]
#[derive(Debug, Clone, Default)]
pub struct Vislcg3ConjunctionEnhancer {
    pub conjunction_tags: Vec<String>,
}

impl Vislcg3ConjunctionEnhancer {
    /// The class every hit of this topic carries: `teaksta-` and the name the
    /// activity registry serves the topic under, which is how the client tells
    /// a hit from a plain word.
    pub const SPAN_CLASS: &'static str = "teaksta-Conjunctions";
    pub const CHUNK_BEGIN_SUFFIX: &'static str = "-B";
    pub const CHUNK_INSIDE_SUFFIX: &'static str = "-I";

    pub fn new() -> Self {
        Self::default()
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.initialize-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.initialize-fn]
    pub fn initialize(&mut self, context: &HashMap<String, String>) -> Result<()> {
        debug!("Conjunction tags {:?}", self.conjunction_tags);
        let conjunction_tags = context
            .get("conjunctionTags")
            .ok_or_else(|| anyhow!("configuration parameter conjunctionTags is not set"))?;
        // Java's String.split(",") drops trailing empty fields, but leaves the
        // whole input as the single element when the separator never matches.
        let mut tags: Vec<String> = conjunction_tags.split(',').map(str::to_string).collect();
        if conjunction_tags.contains(',') {
            while tags.last().is_some_and(|t| t.is_empty()) {
                tags.pop();
            }
        }
        self.conjunction_tags = tags;
        Ok(())
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.process-fn+4]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.process-fn+4]
    pub fn process(&self, doc: &mut Document, mode: Mode) -> Result<()> {
        info!("Starting conjunction enhancement");

        // keep track of ids for each annotation class
        let mut class_counts: HashMap<String, i32> = HashMap::new();
        for con_t in &self.conjunction_tags {
            class_counts.insert(con_t.clone(), 0);
            info!("Tag: {}", con_t);
        }

        // iterating over the configured tags instead of the class-count key set
        // because it is important to control the order in which spans are
        // enhanced

        for con_t in &self.conjunction_tags {
            // go through tokens
            for token_index in 0..doc.cg_tokens.len() {
                if matches!(mode, Mode::Cloze | Mode::Mc) {
                    // more than one reading? don't mark up if the exercise type
                    // is mc or cloze
                    if !self.is_safe(&doc.cg_tokens[token_index]) {
                        continue;
                    }
                }

                let (begin, end, reading_count) = {
                    let cgt = &doc.cg_tokens[token_index];
                    (cgt.begin, cgt.end, cgt.readings.len())
                };

                // analyze reading(s)
                // Loop over all the readings. If there is one analysis that
                // matches the tag pattern then the token will be selected for
                // the exercise.
                for i in 0..reading_count {
                    let matches = self.contains_tag(&doc.cg_tokens[token_index].readings[i], con_t);

                    if matches {
                        // increment id
                        let new_id = class_counts[con_t] + 1;
                        let id = enhancer_utils::get_id(&format!("teaksta-span-{con_t}"), new_id);
                        let span_tag = SpanTag::new(
                            id,
                            &[TOKEN_CLASS, Self::SPAN_CLASS, &format!("teaksta-{con_t}")],
                        );
                        // make new enhancement
                        let e = Enhancement {
                            begin,
                            end,
                            enhance_start: span_tag.start_tag(),
                            enhance_end: span_tag.end_tag().to_string(),
                            relevant: true,
                        };
                        class_counts.insert(con_t.clone(), new_id);
                        doc.enhancements.push(e);
                        break;
                    }
                }
            }
        }

        info!("Finished conjunction enhancement");
        Ok(())
    }

    /// Determines whether the given token is safe, i.e. unambiguous
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.is-safe-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.is-safe-fn]
    fn is_safe(&self, t: &CgToken) -> bool {
        t.readings.len() == 1
    }

    /// Determines whether the given reading contains the given tag
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.contains-tag-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.contains-tag-fn]
    fn contains_tag(&self, cgr: &CgReading, tag: &str) -> bool {
        for rtag in cgr {
            if tag == rtag.as_str() {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{
        assert_process_ignores_token_without_tags, assert_process_keeps_existing_enhancements,
        assert_safe_only_single_reading, assert_splits_tags, cg_token as token, reading,
    };

    fn context(conjunction_tags: &str) -> HashMap<String, String> {
        HashMap::from([("conjunctionTags".to_string(), conjunction_tags.to_string())])
    }

    /// Apply one `conjunctionTags` value and report the tags the enhancer
    /// stored.
    fn configured(
        enhancer: &mut Vislcg3ConjunctionEnhancer,
        conjunction_tags: &str,
    ) -> Vec<String> {
        enhancer
            .initialize(&context(conjunction_tags))
            .expect("conjunctionTags is set");
        enhancer.conjunction_tags.clone()
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.initialize-fn/test]
    #[test]
    fn initialize_splits_conjunction_tags_without_trimming() {
        let mut enhancer = Vislcg3ConjunctionEnhancer::new();
        assert!(enhancer.conjunction_tags.is_empty());

        assert_splits_tags(
            &[
                ("CC,CS", &["CC", "CS"]),
                (" CC , CS ", &[" CC ", " CS "]),
                ("CC", &["CC"]),
            ],
            |conjunction_tags| configured(&mut enhancer, conjunction_tags),
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.initialize-fn/test]
    #[test]
    fn initialize_drops_trailing_empties_only_with_comma() {
        let mut enhancer = Vislcg3ConjunctionEnhancer::new();

        assert_splits_tags(
            &[
                ("CC,CS,,", &["CC", "CS"]),
                ("CC,,CS", &["CC", "", "CS"]),
                (",,", &[]),
                ("", &[""]),
            ],
            |conjunction_tags| configured(&mut enhancer, conjunction_tags),
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.initialize-fn/test]
    #[test]
    fn initialize_fails_absent_conjunction_tags_keeps_field() {
        let mut enhancer = Vislcg3ConjunctionEnhancer::new();
        enhancer.conjunction_tags = vec!["CC".to_string()];

        let err = enhancer
            .initialize(&HashMap::new())
            .expect_err("conjunctionTags is mandatory");
        assert!(err.to_string().contains("conjunctionTags"), "{err}");
        assert_eq!(enhancer.conjunction_tags, ["CC"]);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.is-safe-fn/test]
    #[test]
    fn is_safe_holds_only_for_exactly_one_reading() {
        let enhancer = Vislcg3ConjunctionEnhancer::new();

        assert_safe_only_single_reading(
            4,
            6,
            &["\"ja\"", "CC", "@CVP"],
            &[&["\"ja\"", "CC", "@CVP"], &["\"ja\"", "Pcle", "@ADVL>"]],
            |t| enhancer.is_safe(t),
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.contains-tag-fn/test]
    #[test]
    fn contains_tag_requires_an_equal_element() {
        let enhancer = Vislcg3ConjunctionEnhancer::new();
        let coordinator = reading(&["\"ja\"", "CC", "@CVP"]);

        assert!(enhancer.contains_tag(&coordinator, "CC"));
        assert!(!enhancer.contains_tag(&coordinator, "CS"));
        assert!(!enhancer.contains_tag(&reading(&["\"go\"", "CS", "@CVP"]), "CC"));
        assert!(enhancer.contains_tag(&reading(&["\"go\"", "CS", "@CVP"]), "CS"));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.contains-tag-fn/test]
    #[test]
    fn contains_tag_rejects_substrings_and_decorations() {
        let enhancer = Vislcg3ConjunctionEnhancer::new();
        let coordinator = reading(&["\"CC\"", "CC-decorated", "@CC", "CCx"]);

        assert!(!enhancer.contains_tag(&coordinator, "CC"));
        assert!(!enhancer.contains_tag(&coordinator, "C"));
        assert!(!enhancer.contains_tag(&reading(&["\"ja\"", "CC"]), "cc"));
        assert!(!enhancer.contains_tag(&reading(&[]), "CC"));
        assert!(!enhancer.contains_tag(&reading(&["\"ja\"", "CC"]), ""));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.process-fn+4/test]
    #[test]
    fn process_walks_tags_but_adds_nothing_without_tokens() {
        let enhancer = Vislcg3ConjunctionEnhancer {
            conjunction_tags: vec!["CC".to_string(), "CS".to_string()],
        };

        assert_process_keeps_existing_enhancements("Mun ja don.", |doc| {
            enhancer.process(doc, Mode::Colorize)
        });
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.process-fn+4/test]
    #[test]
    fn process_without_configured_tags_never_inspects_a_token() {
        let enhancer = Vislcg3ConjunctionEnhancer::new();

        assert_process_ignores_token_without_tags(
            "Mun ja don.",
            token(4, 6, &[&["\"ja\"", "CC", "@CVP"]]),
            |doc| enhancer.process(doc, Mode::Colorize),
        );
    }

    /// The coordinator reading carries the configured tag, so the only thing
    /// that can keep the token out of the output is the exercise: `mc` and
    /// `cloze` pass over it for its second reading, `colorize` and `click`
    /// take it.
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-conjunction-enhancer.vislcg3-conjunction-enhancer.process-fn+4/test]
    #[test]
    fn an_ambiguous_token_reaches_the_marking_exercises_only() {
        let enhancer = Vislcg3ConjunctionEnhancer {
            conjunction_tags: vec!["CC".to_string()],
        };

        for (mode, expected) in [
            (Mode::Mc, 0),
            (Mode::Cloze, 0),
            (Mode::Colorize, 1),
            (Mode::Click, 1),
        ] {
            let mut doc = Document::new("Mun ja don.", "sme");
            doc.cg_tokens.push(token(
                4,
                6,
                &[&["\"ja\"", "CC", "@CVP"], &["\"ja\"", "Pcle", "@ADVL>"]],
            ));

            enhancer
                .process(&mut doc, mode)
                .expect("a token the exercise never reaches is not a failure");

            assert_eq!(doc.enhancements.len(), expected, "{mode:?}");
        }
    }
}
