//! What the plural-noun topic does that its siblings do not: the plural
//! reading pattern and the two generator inputs. The pass they all share is
//! exercised in `cg_enhancer_tests`.

use super::*;
use crate::types::CgToken;
use crate::util::cas_utils;

fn plural_reading() -> Vec<String> {
    ["\"beana\"", "N", "<sme>", "Pl", "Nom", "@SUBJ"]
        .iter()
        .map(|tag| tag.to_string())
        .collect()
}

fn singular_reading() -> Vec<String> {
    [
        "\"čáhci\"",
        "N",
        "<sme>",
        "Sem/Plc_Substnc_Wthr",
        "Sg",
        "Nom",
    ]
    .iter()
    .map(|tag| tag.to_string())
    .collect()
}

fn cg_token(begin: usize, end: usize, reading: Vec<String>) -> CgToken {
    CgToken {
        begin,
        end,
        readings: vec![reading],
    }
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.initialize-fn/test]
#[test]
fn initialize_splits_on_comma_without_trimming() {
    let mut enhancer = Vislcg3NounPlEnhancer::default();
    enhancer
        .initialize(Some("N Pl Nom, N Pl Acc, N Ess"))
        .expect("configured");

    assert_eq!(
        enhancer.n_pl_tags,
        Some(vec![
            "N Pl Nom".to_string(),
            " N Pl Acc".to_string(),
            " N Ess".to_string(),
        ])
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.initialize-fn/test]
#[test]
fn initialize_without_parameter_fails_and_leaves_field_unset() {
    let mut enhancer = Vislcg3NounPlEnhancer::default();
    let err = enhancer.initialize(None).unwrap_err();

    assert_eq!(
        err.to_string(),
        "NPlTags configuration parameter is not set"
    );
    assert!(enhancer.n_pl_tags.is_none());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.process-fn/test]
#[test]
fn process_spans_plural_nouns_and_numbers_repeated_readings() {
    let enhancer = Vislcg3NounPlEnhancer::default();
    let mut doc = Document::new("beanat beanat", "sme");
    doc.cg_tokens = vec![
        cg_token(0, 6, plural_reading()),
        cg_token(7, 13, plural_reading()),
    ];

    enhancer.process(&mut doc).expect("process");

    let first = concat!(
        "<span id=\"WERTi-span-beana-N-xsmey-Pl-Nom-@SUBJ-1\" ",
        "class=\"wertiviewtoken  wertiviewSubstantivePlural\"lemma=\"beana\">"
    );
    let second = concat!(
        "<span id=\"WERTi-span-beana-N-xsmey-Pl-Nom-@SUBJ-2\" ",
        "class=\"wertiviewtoken  wertiviewSubstantivePlural\"lemma=\"beana\">"
    );

    assert_eq!(doc.enhancements.len(), 2);
    assert!(doc.enhancements[0].relevant);
    assert_eq!(doc.enhancements[0].begin, 0);
    assert_eq!(doc.enhancements[0].end, 6);
    assert_eq!(doc.enhancements[0].enhance_start, first);
    assert_eq!(doc.enhancements[0].enhance_end, "</span>");
    assert_eq!(doc.enhancements[1].begin, 7);
    assert_eq!(doc.enhancements[1].end, 13);
    assert_eq!(doc.enhancements[1].enhance_start, second);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.process-fn/test]
#[test]
fn process_ignores_readings_without_a_plural_case_tag() {
    let enhancer = Vislcg3NounPlEnhancer::default();
    let mut doc = Document::new("čáhci", "sme");
    doc.cg_tokens = vec![cg_token(0, 6, singular_reading())];

    enhancer.process(&mut doc).expect("process");

    assert!(doc.enhancements.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.process-fn/test]
#[test]
fn process_returns_early_when_the_cas_was_cancelled() {
    let enhancer = Vislcg3NounPlEnhancer::default();
    let mut doc = Document::new("beanat", "sme");
    doc.cg_tokens = vec![cg_token(0, 6, plural_reading())];
    cas_utils::add_enh_id(&mut doc, -1);

    enhancer.process(&mut doc).expect("process");

    assert!(doc.enhancements.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.write-morphological-forms-fn/test]
#[test]
fn write_morphological_forms_expands_cases_and_appends_answer() {
    let enhancer = Vislcg3NounPlEnhancer::default();

    let block = enhancer
        .write_morphological_forms("beana+N+Pl+Nom+@SUBJ")
        .expect("block");

    assert_eq!(
        block,
        "beana+N+Pl+Nom\nbeana+N+Pl+Acc\nbeana+N+Pl+Gen\nbeana+N+Pl+Ill\n\
         beana+N+Pl+Loc\nbeana+N+Pl+Com\nbeana+N+Pl+Ess\nbeana+N+Pl+Nom\n"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.write-morphological-forms-fn/test]
#[test]
fn write_morphological_forms_keeps_first_matching_case_only() {
    let enhancer = Vislcg3NounPlEnhancer::default();

    let block = enhancer
        .write_morphological_forms("beana+N+Pl+Com+@ADVL")
        .expect("block");

    assert!(block.starts_with("beana+N+Pl+Nom\n"));
    assert!(block.ends_with("beana+N+Pl+Com\n"));
    assert_eq!(block.lines().count(), 8);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.write-morphological-forms-fn/test]
#[test]
fn write_morphological_forms_range_error_without_syntactic_tag() {
    let enhancer = Vislcg3NounPlEnhancer::default();

    let missing = enhancer
        .write_morphological_forms("beana+N+Pl+Nom")
        .unwrap_err();
    assert_eq!(missing.to_string(), "begin 0, end -2, length 14");

    let leading = enhancer.write_morphological_forms("@SUBJ").unwrap_err();
    assert_eq!(leading.to_string(), "begin 0, end -1, length 5");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.write-lemma-and-analyses-fn/test]
#[test]
fn write_lemma_and_analyses_drops_language_syntactic_tags() {
    let enhancer = Vislcg3NounPlEnhancer::default();

    let line = enhancer
        .write_lemma_and_analyses("beana+N+<sme>+Pl+Nom+@SUBJ")
        .expect("line");

    assert_eq!(line, "beana+N+Pl+Nom\n");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.write-lemma-and-analyses-fn/test]
#[test]
fn write_lemma_and_analyses_range_error_missing_delimiters() {
    let enhancer = Vislcg3NounPlEnhancer::default();

    let no_plus = enhancer.write_lemma_and_analyses("beana").unwrap_err();
    assert_eq!(no_plus.to_string(), "begin 0, end -1, length 5");

    let no_tag = enhancer
        .write_lemma_and_analyses("beana+N+<sme>+Pl+Nom")
        .unwrap_err();
    assert_eq!(no_tag.to_string(), "begin 0, end -2, length 8");
}
