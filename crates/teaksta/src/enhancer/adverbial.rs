//! Use the TAG-B TAG-I sequences resulting from the CG3 analysis with
//! [`crate::pipeline::vislcg3`] to enhance spans corresponding to the tags
//! specified by the activity as tags of negation forms of verbs.
//!
//! Authors: Niels Ott?, Adriane Boyd, Heli Uibo

use std::collections::HashMap;

use anyhow::{Result, anyhow};
use tracing::{debug, trace};

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

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.initialize-fn+3]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.initialize-fn+3]
    pub fn new(context: &HashMap<String, String>) -> Result<Self> {
        let configured = context
            .get("AdvTags")
            .ok_or_else(|| anyhow!("configuration parameter AdvTags is not set"))?;
        debug!("Adverbial tags {:?}", configured);

        Ok(Vislcg3AdverbialEnhancer {
            adv_tags: syntactic::split_tags(configured),
        })
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.process-fn+4]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.process-fn+4]
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
                trace!("{:?} lemma: {}", cgr, lemma);
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

    /// Build an enhancer from one `AdvTags` value and report the tags it
    /// holds.
    fn configured(value: &str) -> Vec<String> {
        Vislcg3AdverbialEnhancer::new(&context(value))
            .expect("AdvTags is set")
            .adv_tags
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.initialize-fn+3/test]
    #[test]
    fn initialize_splits_adv_tags_on_commas_without_trimming() {
        assert_splits_tags(
            &[
                ("ADVL", &["ADVL"]),
                (" ADVL ,@<ADVL", &[" ADVL ", "@<ADVL"]),
                ("ADVL,,@ADVL>", &["ADVL", "", "@ADVL>"]),
            ],
            configured,
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.initialize-fn+3/test]
    #[test]
    fn initialize_drops_trailing_empties_only_with_comma() {
        assert_splits_tags(
            &[
                ("ADVL,,", &["ADVL"]),
                (",,", &[]),
                ("", &[""]),
                (",ADVL", &["", "ADVL"]),
            ],
            configured,
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.initialize-fn+3/test]
    #[test]
    fn no_enhancer_is_built_without_adv_tags() {
        let err = Vislcg3AdverbialEnhancer::new(&HashMap::new()).expect_err("AdvTags is mandatory");

        assert!(err.to_string().contains("AdvTags"), "{err}");
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.is-safe-fn/test]
    #[test]
    fn is_safe_holds_only_for_exactly_one_reading() {
        let enhancer = Vislcg3AdverbialEnhancer::default();

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
        let enhancer = Vislcg3AdverbialEnhancer::default();
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
        let enhancer = Vislcg3AdverbialEnhancer::default();
        let subject_with_telling_lemma = reading(&["\"ADVLijk\"", "N", "Sg", "Nom", "@SUBJ→"]);

        assert!(enhancer.contains_tag(&subject_with_telling_lemma, "ADVL"));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.contains-tag-fn/test]
    #[test]
    fn contains_tag_flattens_empty_reading_to_empty_string() {
        let enhancer = Vislcg3AdverbialEnhancer::default();
        let empty = reading(&[]);

        assert!(!enhancer.contains_tag(&empty, "ADVL"));
        assert!(enhancer.contains_tag(&empty, ""));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.get-lemma-fn/test]
    #[test]
    fn get_lemma_strips_quotes_keeps_last_quoted() {
        let enhancer = Vislcg3AdverbialEnhancer::default();

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
        let enhancer = Vislcg3AdverbialEnhancer::default();
        let _ = enhancer.get_lemma(&reading(&["\""]));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.process-fn+4/test]
    #[test]
    fn process_walks_tags_but_adds_nothing_without_tokens() {
        let enhancer = Vislcg3AdverbialEnhancer {
            adv_tags: vec!["ADVL".to_string(), "SUBJ".to_string()],
        };

        assert_process_keeps_existing_enhancements("Mun oidnen viesus ikte.", |doc| {
            enhancer.process(doc, Mode::Colorize)
        });
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.process-fn+4/test]
    #[test]
    fn process_without_configured_tags_never_inspects_a_token() {
        let enhancer = Vislcg3AdverbialEnhancer::default();

        assert_process_ignores_token_without_tags(
            "Mun oidnen viesus ikte.",
            token(11, 17, &[&["\"viessu\"", "N", "Sg", "Loc", "@ADVL>"]]),
            |doc| enhancer.process(doc, Mode::Colorize),
        );
    }
}
