//! Use the TAG-B TAG-I sequences resulting from the CG3 analysis with
//! [`crate::pipeline::vislcg3`] to enhance spans corresponding to the tags
//! specified by the activity as tags of negation forms of verbs.
//!
//! Authors: Niels Ott?, Adriane Boyd, Heli Uibo

use std::collections::HashMap;

use anyhow::{Result, anyhow};
use tracing::info;

use crate::enhancer::syntactic;
use crate::types::{CgReading, CgToken, Document};

// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer]
#[derive(Debug, Clone, Default)]
pub struct Vislcg3ObjectEnhancer {
    pub object_tags: Vec<String>,
}

impl Vislcg3ObjectEnhancer {
    pub const CHUNK_BEGIN_SUFFIX: &'static str = "-B";
    pub const CHUNK_INSIDE_SUFFIX: &'static str = "-I";

    pub fn new() -> Self {
        Self::default()
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.initialize-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.initialize-fn+2]
    pub fn initialize(&mut self, context: &HashMap<String, String>) -> Result<()> {
        info!("Object tags {:?}", self.object_tags);
        let obj_tags = context
            .get("ObjTags")
            .ok_or_else(|| anyhow!("configuration parameter ObjTags is not set"))?;
        // Java's String.split(",") drops trailing empty fields, but leaves the
        // whole input as the single element when the separator never matches.
        let mut tags: Vec<String> = obj_tags.split(',').map(str::to_string).collect();
        if obj_tags.contains(',') {
            while tags.last().is_some_and(|t| t.is_empty()) {
                tags.pop();
            }
        }
        self.object_tags = tags;
        Ok(())
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.process-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.process-fn+2]
    pub fn process(&self, doc: &mut Document) -> Result<()> {
        syntactic::run(
            doc,
            &syntactic::FunctionSpec {
                start_log: "Starting Object enhancement",
                finish_log: "Finished Object enhancement",
                span_class: "teaksta-Object",
                tags: &self.object_tags,
                is_safe: &|t| self.is_safe(t),
                contains_tag: &|cgr, tag| self.contains_tag(cgr, tag),
            },
        )
    }

    /// Determines whether the given token is safe, i.e. unambiguous
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.is-safe-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.is-safe-fn]
    fn is_safe(&self, t: &CgToken) -> bool {
        t.readings.len() == 1
    }

    /// Determines whether the given reading contains the given tag
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.contains-tag-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.contains-tag-fn]
    fn contains_tag(&self, cgr: &CgReading, tag: &str) -> bool {
        let mut reading_str = String::new();
        for rtag in cgr {
            reading_str = reading_str + rtag.as_str() + " ";
        }

        // Tag string contains the given tag sequence as a substring, plus it is
        // in the accusative case if it is the phrase nucleus.
        if reading_str.contains(tag) {
            info!("{:?} contains {}", cgr, tag);
            return true;
        }

        false
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.get-lemma-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.get-lemma-fn]
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
        assert_process_requires_enhancement_type, assert_safe_only_single_reading,
        assert_splits_tags, cg_token as token, reading,
    };

    fn context(obj_tags: &str) -> HashMap<String, String> {
        HashMap::from([("ObjTags".to_string(), obj_tags.to_string())])
    }

    /// Apply one `ObjTags` value and report the tags the enhancer stored.
    fn configured(enhancer: &mut Vislcg3ObjectEnhancer, obj_tags: &str) -> Vec<String> {
        enhancer
            .initialize(&context(obj_tags))
            .expect("ObjTags is set");
        enhancer.object_tags.clone()
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.initialize-fn+2/test]
    #[test]
    fn initialize_splits_obj_tags_on_commas_without_trimming() {
        let mut enhancer = Vislcg3ObjectEnhancer::new();
        assert!(enhancer.object_tags.is_empty());

        assert_splits_tags(
            &[
                ("OBJ", &["OBJ"]),
                (" OBJ ,@OBJ→", &[" OBJ ", "@OBJ→"]),
                ("OBJ,,SUBJ", &["OBJ", "", "SUBJ"]),
            ],
            |obj_tags| configured(&mut enhancer, obj_tags),
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.initialize-fn+2/test]
    #[test]
    fn initialize_drops_trailing_empties_only_with_comma() {
        let mut enhancer = Vislcg3ObjectEnhancer::new();

        assert_splits_tags(
            &[
                ("OBJ,,", &["OBJ"]),
                (",,", &[]),
                ("", &[""]),
                (",OBJ", &["", "OBJ"]),
            ],
            |obj_tags| configured(&mut enhancer, obj_tags),
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.initialize-fn+2/test]
    #[test]
    fn initialize_fails_absent_obj_tags_keeps_field() {
        let mut enhancer = Vislcg3ObjectEnhancer::new();
        enhancer.object_tags = vec!["OBJ".to_string()];

        let err = enhancer
            .initialize(&HashMap::new())
            .expect_err("ObjTags is mandatory");
        assert!(err.to_string().contains("ObjTags"), "{err}");
        assert_eq!(enhancer.object_tags, ["OBJ"]);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.is-safe-fn/test]
    #[test]
    fn is_safe_holds_only_for_exactly_one_reading() {
        let enhancer = Vislcg3ObjectEnhancer::new();

        assert_safe_only_single_reading(
            0,
            6,
            &["\"mánná\"", "N", "Acc", "@OBJ→"],
            &[&["N", "Sg", "Acc", "@OBJ→"], &["N", "Pl", "Nom", "@SUBJ→"]],
            |t| enhancer.is_safe(t),
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.contains-tag-fn/test]
    #[test]
    fn contains_tag_matches_substring_of_flattened_reading() {
        let enhancer = Vislcg3ObjectEnhancer::new();
        let accusative = reading(&["\"mánná\"", "N", "Sg", "Acc", "@OBJ→"]);

        assert!(enhancer.contains_tag(&accusative, "OBJ"));
        assert!(enhancer.contains_tag(&reading(&["V", "@-F<OBJ"]), "OBJ"));
        assert!(enhancer.contains_tag(&accusative, "Sg Acc"));
        assert!(enhancer.contains_tag(&accusative, "@OBJ→ "));

        assert!(!enhancer.contains_tag(&accusative, "obj"));
        assert!(!enhancer.contains_tag(&accusative, "Acc Sg"));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.contains-tag-fn/test]
    #[test]
    fn contains_tag_matches_lemma_embedding_tag_text() {
        let enhancer = Vislcg3ObjectEnhancer::new();
        let subject_with_telling_lemma = reading(&["\"OBJEKTA\"", "N", "Sg", "Nom", "@SUBJ→"]);

        assert!(enhancer.contains_tag(&subject_with_telling_lemma, "OBJ"));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.contains-tag-fn/test]
    #[test]
    fn contains_tag_flattens_empty_reading_to_empty_string() {
        let enhancer = Vislcg3ObjectEnhancer::new();
        let empty = reading(&[]);

        assert!(!enhancer.contains_tag(&empty, "OBJ"));
        assert!(enhancer.contains_tag(&empty, ""));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.get-lemma-fn/test]
    #[test]
    fn get_lemma_strips_quotes_keeps_last_quoted() {
        let enhancer = Vislcg3ObjectEnhancer::new();

        assert_eq!(
            enhancer.get_lemma(&reading(&["\"mánná\"", "N", "Sg", "Acc"])),
            "mánná"
        );
        assert_eq!(
            enhancer.get_lemma(&reading(&["\"first\"", "N", "\"second\""])),
            "second"
        );
        assert_eq!(enhancer.get_lemma(&reading(&["N", "Sg", "Acc"])), "");
        assert_eq!(enhancer.get_lemma(&reading(&[])), "");
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.get-lemma-fn/test]
    #[test]
    #[should_panic(expected = "starts at 1 but ends at 0")]
    fn get_lemma_panics_on_lone_double_quote() {
        let enhancer = Vislcg3ObjectEnhancer::new();
        let _ = enhancer.get_lemma(&reading(&["\""]));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.process-fn+2/test]
    #[test]
    fn process_walks_tags_but_adds_nothing_without_tokens() {
        let enhancer = Vislcg3ObjectEnhancer {
            object_tags: vec!["OBJ".to_string(), "SUBJ".to_string()],
            ..Default::default()
        };

        assert_process_keeps_existing_enhancements("Mun oainnán mánáid.", |doc| {
            enhancer.process(doc)
        });
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.process-fn+2/test]
    #[test]
    fn process_without_configured_tags_never_inspects_a_token() {
        let enhancer = Vislcg3ObjectEnhancer::new();

        assert_process_ignores_token_without_tags(
            "Mun oainnán mánáid.",
            token(12, 18, &[&["\"mánná\"", "N", "Sg", "Acc", "@OBJ→"]]),
            |doc| enhancer.process(doc),
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.process-fn+2/test]
    #[test]
    fn process_requires_enhancement_type_for_first_token() {
        let enhancer = Vislcg3ObjectEnhancer {
            object_tags: vec!["OBJ".to_string()],
            ..Default::default()
        };
        let mut doc = Document::new("Mun oainnán mánáid.", "sme");
        doc.cg_tokens
            .push(token(12, 18, &[&["\"mánná\"", "N", "Sg", "Acc", "@OBJ→"]]));

        assert_process_requires_enhancement_type(&mut doc, |doc| enhancer.process(doc));
    }
}
