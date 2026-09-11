//! What the singular-noun topic does that its siblings do not: the reading
//! patterns, the preposition-hint pass, and the two generator inputs. The
//! pass they all share is exercised in `cg_enhancer_tests`.

use super::*;
use crate::types::CgToken;
use crate::types::PIPELINE_LANGUAGE;

fn enhancer() -> Vislcg3NounEnhancer {
    Vislcg3NounEnhancer { n_tags: None }
}

fn cg_token(begin: usize, end: usize, readings: &[&[&str]]) -> CgToken {
    CgToken {
        begin,
        end,
        readings: readings
            .iter()
            .map(|reading| reading.iter().map(|tag| tag.to_string()).collect())
            .collect(),
    }
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.initialize-fn/test]
#[test]
fn n_tags_split_on_commas_keep_leading_spaces() {
    let mut enh = enhancer();

    enh.initialize(Some("Sg Nom, Sg Acc, Ess")).unwrap();

    assert_eq!(
        enh.n_tags.as_deref(),
        Some(
            &[
                "Sg Nom".to_string(),
                " Sg Acc".to_string(),
                " Ess".to_string()
            ][..]
        )
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.initialize-fn/test]
#[test]
fn a_missing_n_tags_parameter_fails_initialisation() {
    let mut enh = enhancer();

    let err = enh.initialize(None).unwrap_err();

    assert!(err.to_string().contains("NTags"), "{err}");
    assert!(enh.n_tags.is_none());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.process-fn+5/test]
#[test]
fn a_singular_noun_span_carries_its_lemma() {
    let enh = enhancer();
    let mut doc = Document::new("čáhci čáhci", PIPELINE_LANGUAGE);
    let reading: &[&str] = &["\"čáhci\"", "N", "<sme>", "Sem/Plc", "Sg", "Nom"];
    doc.cg_tokens.push(cg_token(0, 7, &[reading]));
    doc.cg_tokens.push(cg_token(8, 15, &[reading]));

    enh.process(&mut doc, Mode::Colorize).unwrap();

    assert_eq!(doc.enhancements.len(), 2);
    assert!(doc.enhancements[0].relevant);
    assert_eq!((doc.enhancements[0].begin, doc.enhancements[0].end), (0, 7));
    assert_eq!(doc.enhancements[0].enhance_end, "</span>");
    assert_eq!(
        doc.enhancements[0].enhance_start,
        "<span id=\"teaksta-span-čáhci-N-xsmey-Sem/Plc-Sg-Nom-1\" \
         class=\"teaksta-token teaksta-Substantive\" lemma=\"čáhci\">"
    );
    assert_eq!(
        doc.enhancements[1].enhance_start,
        "<span id=\"teaksta-span-čáhci-N-xsmey-Sem/Plc-Sg-Nom-2\" \
         class=\"teaksta-token teaksta-Substantive\" lemma=\"čáhci\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.process-fn+5/test]
#[test]
fn preposition_hint_is_linked_from_next_noun() {
    let enh = enhancer();
    let mut doc = Document::new("maŋŋel beana", PIPELINE_LANGUAGE);
    doc.cg_tokens.push(cg_token(0, 8, &[&["\"maŋŋel\"", "Pr"]]));
    doc.cg_tokens
        .push(cg_token(9, 14, &[&["\"beana\"", "N", "Sg", "Nom"]]));

    enh.process(&mut doc, Mode::Colorize).unwrap();

    assert_eq!(doc.enhancements.len(), 2);
    assert_eq!(
        doc.enhancements[0].enhance_start,
        "<span id=\"teaksta-span-maŋŋel-Pr-1\" class=\"teaksta-hinttag\">"
    );
    assert_eq!((doc.enhancements[0].begin, doc.enhancements[0].end), (0, 8));
    assert_eq!(
        doc.enhancements[1].enhance_start,
        "<span id=\"teaksta-span-beana-N-Sg-Nom-1\" \
         class=\"teaksta-token teaksta-Substantive\" lemma=\"beana\" \
         hintid=\"teaksta-span-maŋŋel-Pr-1\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.process-fn+5/test]
#[test]
fn a_bare_number_tag_without_case_is_skipped() {
    let enh = enhancer();
    let mut doc = Document::new("ruoktu ruovttut", PIPELINE_LANGUAGE);
    doc.cg_tokens
        .push(cg_token(0, 6, &[&["\"ruoktu\"", "N", "Sg"]]));
    doc.cg_tokens
        .push(cg_token(7, 15, &[&["\"ruoktu\"", "N", "Pl"]]));

    enh.process(&mut doc, Mode::Colorize).unwrap();

    assert!(doc.enhancements.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.process-fn+5/test]
#[test]
fn every_named_number_and_case_pair_qualifies() {
    let enh = enhancer();
    let mut doc = Document::new("beana", PIPELINE_LANGUAGE);
    for number in ["Sg", "Pl"] {
        for case in ["Nom", "Acc", "Gen", "Ill", "Loc", "Com"] {
            doc.cg_tokens
                .push(cg_token(0, 5, &[&["\"beana\"", "N", number, case]]));
        }
    }
    doc.cg_tokens
        .push(cg_token(0, 5, &[&["\"beana\"", "N", "Ess"]]));

    enh.process(&mut doc, Mode::Colorize).unwrap();

    assert_eq!(doc.enhancements.len(), 13);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.process-fn+5/test]
#[test]
fn one_excluded_reading_disqualifies_a_token() {
    let enh = enhancer();
    let mut doc = Document::new("mun", PIPELINE_LANGUAGE);
    doc.cg_tokens.push(cg_token(
        0,
        3,
        &[
            &["\"mun\"", "N", "Sg", "Nom"],
            &["\"mun\"", "Pron", "Pers", "Sg1", "Nom"],
        ],
    ));

    enh.process(&mut doc, Mode::Colorize).unwrap();

    assert!(doc.enhancements.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.process-fn+5/test]
#[test]
fn an_adjective_before_pred_stays_eligible() {
    let enh = enhancer();
    let mut doc = Document::new("stuoris beana", PIPELINE_LANGUAGE);
    doc.cg_tokens.push(cg_token(
        0,
        13,
        &[
            &["\"stuoris\"", "A", "Pred"],
            &["\"beana\"", "N", "Sg", "Nom"],
        ],
    ));

    enh.process(&mut doc, Mode::Colorize).unwrap();

    // `A+` is excluded only when no `Pred` follows it on the same line.
    assert_eq!(doc.enhancements.len(), 1);

    let mut without_pred = Document::new("stuoris beana", PIPELINE_LANGUAGE);
    without_pred.cg_tokens.push(cg_token(
        0,
        13,
        &[
            &["\"stuoris\"", "A", "Sg", "Nom"],
            &["\"beana\"", "N", "Sg", "Nom"],
        ],
    ));

    enh.process(&mut without_pred, Mode::Colorize).unwrap();

    assert!(without_pred.enhancements.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-morphological-forms-fn+2/test]
#[test]
fn four_distractor_analyses_precede_the_correct_one() {
    let enh = enhancer();

    let block = enh.write_morphological_forms("beana+N+Sg+Nom").unwrap();

    assert_eq!(
        block,
        "beana+N+Sg+Acc\nbeana+N+Sg+Ill\nbeana+N+Sg+Loc\nbeana+N+Sg+Com\nbeana+N+Sg+Nom\n"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-morphological-forms-fn+2/test]
#[test]
fn syntactic_tag_dropped_from_correct_answer_line() {
    let enh = enhancer();

    let block = enh
        .write_morphological_forms("beana+N+Sg+Nom+@SUBJ>")
        .unwrap();

    assert!(block.ends_with("beana+N+Sg+Nom\n"), "{block}");
    assert!(!block.contains('@'), "{block}");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-morphological-forms-fn+2/test]
#[test]
fn plural_readings_draw_from_the_plural_distractor_table() {
    let enh = enhancer();

    let block = enh.write_morphological_forms("beana+N+Pl+Com").unwrap();

    assert_eq!(
        block,
        "beana+N+Pl+Nom\nbeana+N+Pl+Ill\nbeana+N+Ess\nbeana+N+Sg+Acc\nbeana+N+Pl+Com\n"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-morphological-forms-fn+2/test]
#[test]
fn generator_rejected_tags_are_stripped() {
    let enh = enhancer();

    let block = enh
        .write_morphological_forms("beana+N+Allegro+Sg+Nom")
        .unwrap();

    assert_eq!(
        block,
        "beana+N+Sg+Acc\nbeana+N+Sg+Ill\nbeana+N+Sg+Loc\nbeana+N+Sg+Com\nbeana+N+Sg+Nom\n"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-morphological-forms-fn+2/test]
#[test]
fn an_essive_reading_sweeps_its_own_table() {
    let enh = enhancer();

    let block = enh.write_morphological_forms("beana+N+Ess").unwrap();

    assert_eq!(
        block,
        "beana+N+Pl+Nom\nbeana+N+Sg+Ill\nbeana+N+Sg+Loc\nbeana+N+Pl+Acc\nbeana+N+Ess\n"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-morphological-forms-fn+2/test]
#[test]
fn a_bare_case_marker_without_number_fails() {
    let enh = enhancer();

    let err = enh
        .write_morphological_forms("guolli+N+Loc+Sg+Nom")
        .unwrap_err();

    assert!(err.to_string().contains("end -1"), "{err}");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-lemma-and-analyses-fn+2/test]
#[test]
fn a_singular_genitive_gains_its_plural_counterpart() {
    let enh = enhancer();

    let block = enh
        .write_lemma_and_analyses("beana+N+<sme>+Sem/Ani+Sg+Gen+@SUBJ>")
        .unwrap();

    assert_eq!(block, "beana+N+Sem/Ani+Sg+Gen\nbeana+N+Sem/Ani+Pl+Gen\n");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-lemma-and-analyses-fn+2/test]
#[test]
fn a_plural_locative_gains_its_singular_counterpart() {
    let enh = enhancer();

    let block = enh
        .write_lemma_and_analyses("beana+N+<sme>+Pl+Loc")
        .unwrap();

    assert_eq!(block, "beana+N+Pl+Loc\nbeana+N+Sg+Loc\n");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-lemma-and-analyses-fn+2/test]
#[test]
fn nominative_readings_yield_a_single_line() {
    let enh = enhancer();

    let block = enh
        .write_lemma_and_analyses("beana+N+<sme>+Sg+Nom")
        .unwrap();

    assert_eq!(block, "beana+N+Sg+Nom\n");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-lemma-and-analyses-fn+2/test]
#[test]
fn a_reading_without_a_plus_fails() {
    let enh = enhancer();

    let err = enh.write_lemma_and_analyses("beana").unwrap_err();

    assert!(err.to_string().contains("end -1"), "{err}");
}
