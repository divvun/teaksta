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

// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer]
#[derive(Debug, Clone, Default)]
pub struct Vislcg3AdverbialEnhancer {
    pub adv_tags: Vec<String>,
}

impl Vislcg3AdverbialEnhancer {
    /// The class every hit of this topic carries: `teaksta-` and the name the
    /// activity registry serves the topic under, which is how the client tells
    /// a hit from a plain word.
    pub const SPAN_CLASS: &'static str = "teaksta-Adverbial";
    pub const CHUNK_BEGIN_SUFFIX: &'static str = "-B";
    pub const CHUNK_INSIDE_SUFFIX: &'static str = "-I";

    pub fn new() -> Self {
        Self::default()
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.initialize-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.initialize-fn+2]
    pub fn initialize(&mut self, context: &HashMap<String, String>) -> Result<()> {
        info!("Adverbial tags {:?}", self.adv_tags);
        let adv_tags = context
            .get("AdvTags")
            .ok_or_else(|| anyhow!("configuration parameter AdvTags is not set"))?;
        // Java's String.split(",") drops trailing empty fields, but leaves the
        // whole input as the single element when the separator never matches.
        let mut tags: Vec<String> = adv_tags.split(',').map(str::to_string).collect();
        if adv_tags.contains(',') {
            while tags.last().is_some_and(|t| t.is_empty()) {
                tags.pop();
            }
        }
        self.adv_tags = tags;
        Ok(())
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.process-fn+3]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.process-fn+3]
    pub fn process(&self, doc: &mut Document, mode: Mode) -> Result<()> {
        syntactic::run(
            doc,
            &syntactic::FunctionSpec::plain(
                "Starting Adverbial enhancement",
                "Finished adv enhancement",
                Self::SPAN_CLASS,
                &self.adv_tags,
                &|t| self.is_safe(t),
                &|cgr, tag| self.contains_tag(cgr, tag),
            ),
            mode,
        )
    }

    /// Determines whether the given token is safe, i.e. unambiguous
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.is-safe-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.is-safe-fn]
    fn is_safe(&self, t: &CgToken) -> bool {
        t.readings.len() == 1
    }

    /// Determines whether the given reading contains the given tag
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.contains-tag-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.contains-tag-fn]
    fn contains_tag(&self, cgr: &CgReading, tag: &str) -> bool {
        let reading_str = flatten_reading(cgr, ReadingJoin::TrailingSpace);

        // Tag string contains the given tag sequence as a substring. Only noun
        // phrases as adverbials.
        if reading_str.contains(tag) {
            return true;
        }

        false
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.get-lemma-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.get-lemma-fn]
    // Retained with no call sites: every invocation in the class is commented
    // out, so the method is unreachable by design.
    #[allow(dead_code)]
    fn get_lemma(&self, cgr: &CgReading) -> String {
        let mut lemma = String::new();
        // Obtain the lemma from the CG reading.
        for rtag in cgr {
            if rtag.starts_with('"') {
                lemma = rtag[1..rtag.len() - 1].to_string();
                info!("{:?} lemma: {}", cgr, lemma);
            }
        }
        // The lemma needs no conversion to UTF-8: the whole CG input and output
        // is already converted.
        lemma
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{
        assert_process_ignores_token_without_tags, assert_process_keeps_existing_enhancements,
        assert_safe_only_single_reading, assert_splits_tags, cg_token as token, reading,
    };

    fn context(adv_tags: &str) -> HashMap<String, String> {
        HashMap::from([("AdvTags".to_string(), adv_tags.to_string())])
    }

    /// Apply one `AdvTags` value and report the tags the enhancer stored.
    fn configured(enhancer: &mut Vislcg3AdverbialEnhancer, adv_tags: &str) -> Vec<String> {
        enhancer
            .initialize(&context(adv_tags))
            .expect("AdvTags is set");
        enhancer.adv_tags.clone()
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.initialize-fn+2/test]
    #[test]
    fn initialize_splits_adv_tags_on_commas_without_trimming() {
        let mut enhancer = Vislcg3AdverbialEnhancer::new();
        assert!(enhancer.adv_tags.is_empty());

        assert_splits_tags(
            &[
                ("ADVL", &["ADVL"]),
                (" ADVL ,@<ADVL", &[" ADVL ", "@<ADVL"]),
                ("ADVL,,@ADVL>", &["ADVL", "", "@ADVL>"]),
            ],
            |adv_tags| configured(&mut enhancer, adv_tags),
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.initialize-fn+2/test]
    #[test]
    fn initialize_drops_trailing_empties_only_with_comma() {
        let mut enhancer = Vislcg3AdverbialEnhancer::new();

        assert_splits_tags(
            &[
                ("ADVL,,", &["ADVL"]),
                (",,", &[]),
                ("", &[""]),
                (",ADVL", &["", "ADVL"]),
            ],
            |adv_tags| configured(&mut enhancer, adv_tags),
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.initialize-fn+2/test]
    #[test]
    fn initialize_fails_absent_adv_tags_keeps_paths() {
        let mut enhancer = Vislcg3AdverbialEnhancer::new();
        enhancer.adv_tags = vec!["ADVL".to_string()];

        let err = enhancer
            .initialize(&HashMap::new())
            .expect_err("AdvTags is mandatory");
        assert!(err.to_string().contains("AdvTags"), "{err}");
        assert_eq!(enhancer.adv_tags, ["ADVL"]);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.is-safe-fn/test]
    #[test]
    fn is_safe_holds_only_for_exactly_one_reading() {
        let enhancer = Vislcg3AdverbialEnhancer::new();

        assert_safe_only_single_reading(
            0,
            5,
            &["\"ikte\"", "Adv", "@ADVL>"],
            &[&["Adv", "@ADVL>"], &["N", "Sg", "Nom", "@SUBJ→"]],
            |t| enhancer.is_safe(t),
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.contains-tag-fn/test]
    #[test]
    fn contains_tag_matches_substring_of_flattened_reading() {
        let enhancer = Vislcg3AdverbialEnhancer::new();
        let locative = reading(&["\"viessu\"", "N", "Sg", "Loc", "@ADVL>"]);

        assert!(enhancer.contains_tag(&locative, "ADVL"));
        assert!(enhancer.contains_tag(&reading(&["Adv", "@<ADVL"]), "ADVL"));
        assert!(enhancer.contains_tag(&locative, "Sg Loc"));
        assert!(enhancer.contains_tag(&locative, "@ADVL> "));

        assert!(!enhancer.contains_tag(&locative, "advl"));
        assert!(!enhancer.contains_tag(&locative, "Loc Sg"));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.contains-tag-fn/test]
    #[test]
    fn contains_tag_matches_lemma_embedding_tag_text() {
        let enhancer = Vislcg3AdverbialEnhancer::new();
        let subject_with_telling_lemma = reading(&["\"ADVLijk\"", "N", "Sg", "Nom", "@SUBJ→"]);

        assert!(enhancer.contains_tag(&subject_with_telling_lemma, "ADVL"));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.contains-tag-fn/test]
    #[test]
    fn contains_tag_flattens_empty_reading_to_empty_string() {
        let enhancer = Vislcg3AdverbialEnhancer::new();
        let empty = reading(&[]);

        assert!(!enhancer.contains_tag(&empty, "ADVL"));
        assert!(enhancer.contains_tag(&empty, ""));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.get-lemma-fn/test]
    #[test]
    fn get_lemma_strips_quotes_keeps_last_quoted() {
        let enhancer = Vislcg3AdverbialEnhancer::new();

        assert_eq!(
            enhancer.get_lemma(&reading(&["\"viessu\"", "N", "Sg", "Loc"])),
            "viessu"
        );
        assert_eq!(
            enhancer.get_lemma(&reading(&["\"first\"", "Adv", "\"second\""])),
            "second"
        );
        assert_eq!(enhancer.get_lemma(&reading(&["Adv", "@ADVL>"])), "");
        assert_eq!(enhancer.get_lemma(&reading(&[])), "");
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.get-lemma-fn/test]
    #[test]
    #[should_panic(expected = "starts at 1 but ends at 0")]
    fn get_lemma_panics_on_lone_double_quote() {
        let enhancer = Vislcg3AdverbialEnhancer::new();
        let _ = enhancer.get_lemma(&reading(&["\""]));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.process-fn+3/test]
    #[test]
    fn process_walks_tags_but_adds_nothing_without_tokens() {
        let enhancer = Vislcg3AdverbialEnhancer {
            adv_tags: vec!["ADVL".to_string(), "SUBJ".to_string()],
            ..Default::default()
        };

        assert_process_keeps_existing_enhancements("Mun oidnen viesus ikte.", |doc| {
            enhancer.process(doc, Mode::Colorize)
        });
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.process-fn+3/test]
    #[test]
    fn process_without_configured_tags_never_inspects_a_token() {
        let enhancer = Vislcg3AdverbialEnhancer::new();

        assert_process_ignores_token_without_tags(
            "Mun oidnen viesus ikte.",
            token(11, 17, &[&["\"viessu\"", "N", "Sg", "Loc", "@ADVL>"]]),
            |doc| enhancer.process(doc, Mode::Colorize),
        );
    }
}
