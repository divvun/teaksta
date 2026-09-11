//! Use the TAG-B TAG-I sequences resulting from the CG3 analysis with
//! [`crate::pipeline::vislcg3`] to enhance spans corresponding to the tags
//! specified by the activity as tags of negation forms of verbs.
//!
//! Authors: Niels Ott?, Adriane Boyd, Heli Uibo

use std::collections::HashMap;

use anyhow::{Result, anyhow};
use tracing::info;

use crate::enhancer::syntactic;
use crate::server::api::Mode;
use crate::types::{CgReading, CgToken, Document, ReadingJoin, flatten_reading};

// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer]
#[derive(Debug, Clone, Default)]
pub struct Vislcg3SubjectEnhancer {
    pub subject_tags: Vec<String>,
}

impl Vislcg3SubjectEnhancer {
    /// The class every hit of this topic carries: `teaksta-` and the name the
    /// activity registry serves the topic under, which is how the client tells
    /// a hit from a plain word.
    pub const SPAN_CLASS: &'static str = "teaksta-Subject";
    pub const CHUNK_BEGIN_SUFFIX: &'static str = "-B";
    pub const CHUNK_INSIDE_SUFFIX: &'static str = "-I";

    pub fn new() -> Self {
        Self::default()
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer.initialize-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer.initialize-fn+2]
    pub fn initialize(&mut self, context: &HashMap<String, String>) -> Result<()> {
        info!("Subject tags {:?}", self.subject_tags);
        let subj_tags = context
            .get("SubjTags")
            .ok_or_else(|| anyhow!("configuration parameter SubjTags is not set"))?;
        // Java's String.split(",") drops trailing empty fields, but leaves the
        // whole input as the single element when the separator never matches.
        let mut tags: Vec<String> = subj_tags.split(',').map(str::to_string).collect();
        if subj_tags.contains(',') {
            while tags.last().is_some_and(|t| t.is_empty()) {
                tags.pop();
            }
        }
        self.subject_tags = tags;
        Ok(())
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer.process-fn+3]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer.process-fn+3]
    pub fn process(&self, doc: &mut Document, mode: Mode) -> Result<()> {
        syntactic::run(
            doc,
            &syntactic::FunctionSpec::plain(
                "Starting Subject enhancement",
                "Finished subject enhancement",
                Self::SPAN_CLASS,
                &self.subject_tags,
                &|t| self.is_safe(t),
                &|cgr, tag| self.contains_tag(cgr, tag),
            ),
            mode,
        )
    }

    /// Determines whether the given token is safe, i.e. unambiguous
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer.is-safe-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer.is-safe-fn]
    fn is_safe(&self, t: &CgToken) -> bool {
        t.readings.len() == 1
    }

    /// Determines whether the given reading contains the given tag
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer.contains-tag-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer.contains-tag-fn]
    fn contains_tag(&self, cgr: &CgReading, tag: &str) -> bool {
        let reading_str = flatten_reading(cgr, ReadingJoin::TrailingSpace);

        // Tag string contains the given tag sequence as a substring, plus it
        // should be in the nominative case.
        if reading_str.contains(tag) {
            info!("{:?} contains {}", cgr, tag);
            return true;
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

    fn context(subj_tags: &str) -> HashMap<String, String> {
        HashMap::from([("SubjTags".to_string(), subj_tags.to_string())])
    }

    /// Apply one `SubjTags` value and report the tags the enhancer stored.
    fn configured(enhancer: &mut Vislcg3SubjectEnhancer, subj_tags: &str) -> Vec<String> {
        enhancer
            .initialize(&context(subj_tags))
            .expect("SubjTags is set");
        enhancer.subject_tags.clone()
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer.initialize-fn+2/test]
    #[test]
    fn initialize_splits_subj_tags_on_commas_without_trimming() {
        let mut enhancer = Vislcg3SubjectEnhancer::new();
        assert!(enhancer.subject_tags.is_empty());

        assert_splits_tags(
            &[
                ("SUBJ", &["SUBJ"]),
                (" SUBJ ,@<SUBJ", &[" SUBJ ", "@<SUBJ"]),
                ("SUBJ,,@SUBJ→", &["SUBJ", "", "@SUBJ→"]),
            ],
            |subj_tags| configured(&mut enhancer, subj_tags),
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer.initialize-fn+2/test]
    #[test]
    fn initialize_drops_trailing_empties_only_with_comma() {
        let mut enhancer = Vislcg3SubjectEnhancer::new();

        assert_splits_tags(
            &[
                ("SUBJ,,", &["SUBJ"]),
                (",,", &[]),
                ("", &[""]),
                (",SUBJ", &["", "SUBJ"]),
            ],
            |subj_tags| configured(&mut enhancer, subj_tags),
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer.initialize-fn+2/test]
    #[test]
    fn initialize_fails_absent_subj_tags_keeps_field() {
        let mut enhancer = Vislcg3SubjectEnhancer::new();
        enhancer.subject_tags = vec!["SUBJ".to_string()];

        let err = enhancer
            .initialize(&HashMap::new())
            .expect_err("SubjTags is mandatory");
        assert!(err.to_string().contains("SubjTags"), "{err}");
        assert_eq!(enhancer.subject_tags, ["SUBJ"]);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer.is-safe-fn/test]
    #[test]
    fn is_safe_holds_only_for_exactly_one_reading() {
        let enhancer = Vislcg3SubjectEnhancer::new();

        assert_safe_only_single_reading(
            0,
            3,
            &["\"mun\"", "Pron", "Sg1", "Nom", "@SUBJ→"],
            &[
                &["Pron", "Sg1", "Nom", "@SUBJ→"],
                &["N", "Sg", "Acc", "@OBJ→"],
            ],
            |t| enhancer.is_safe(t),
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer.contains-tag-fn/test]
    #[test]
    fn contains_tag_matches_substring_of_flattened_reading() {
        let enhancer = Vislcg3SubjectEnhancer::new();
        let nominative = reading(&["\"mun\"", "Pron", "Pers", "Sg1", "Nom", "@SUBJ→"]);

        assert!(enhancer.contains_tag(&nominative, "SUBJ"));
        assert!(enhancer.contains_tag(&reading(&["N", "@<SUBJ"]), "SUBJ"));
        assert!(enhancer.contains_tag(&nominative, "Sg1 Nom"));
        assert!(enhancer.contains_tag(&nominative, "@SUBJ→ "));

        assert!(!enhancer.contains_tag(&nominative, "subj"));
        assert!(!enhancer.contains_tag(&nominative, "Nom Sg1"));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer.contains-tag-fn/test]
    #[test]
    fn contains_tag_ignores_case_despite_nominative_comment() {
        let enhancer = Vislcg3SubjectEnhancer::new();
        let accusative = reading(&["\"mánná\"", "N", "Sg", "Acc", "@SUBJ→"]);

        assert!(enhancer.contains_tag(&accusative, "SUBJ"));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer.contains-tag-fn/test]
    #[test]
    fn contains_tag_matches_lemma_embedding_tag_text() {
        let enhancer = Vislcg3SubjectEnhancer::new();
        let object_with_telling_lemma = reading(&["\"SUBJEAKTA\"", "N", "Sg", "Acc", "@OBJ→"]);

        assert!(enhancer.contains_tag(&object_with_telling_lemma, "SUBJ"));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer.contains-tag-fn/test]
    #[test]
    fn contains_tag_flattens_empty_reading_to_empty_string() {
        let enhancer = Vislcg3SubjectEnhancer::new();
        let empty = reading(&[]);

        assert!(!enhancer.contains_tag(&empty, "SUBJ"));
        assert!(enhancer.contains_tag(&empty, ""));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer.process-fn+3/test]
    #[test]
    fn process_walks_tags_but_adds_nothing_without_tokens() {
        let enhancer = Vislcg3SubjectEnhancer {
            subject_tags: vec!["SUBJ".to_string(), "OBJ".to_string()],
            ..Default::default()
        };

        assert_process_keeps_existing_enhancements("Mun oainnán mánáid.", |doc| {
            enhancer.process(doc, Mode::Colorize)
        });
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer.process-fn+3/test]
    #[test]
    fn process_without_configured_tags_never_inspects_a_token() {
        let enhancer = Vislcg3SubjectEnhancer::new();

        assert_process_ignores_token_without_tags(
            "Mun oainnán mánáid.",
            token(0, 3, &[&["\"mun\"", "Pron", "Sg1", "Nom", "@SUBJ→"]]),
            |doc| enhancer.process(doc, Mode::Colorize),
        );
    }
}
