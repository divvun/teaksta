use super::*;
use crate::types::PIPELINE_LANGUAGE;

fn token(begin: usize, end: usize) -> Token {
    Token {
        begin,
        end,
        ..Default::default()
    }
}

fn reading(tags: &[&str]) -> CgReading {
    tags.iter().map(|tag| tag.to_string()).collect()
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.copy-fn/test]
#[test]
fn copy_transfers_offsets_and_leaves_the_readings_alone() {
    let annotator = Vislcg3Annotator::new();
    let source = Token {
        begin: 12,
        end: 18,
        tag: Some("N".to_string()),
        lemma: Some("guolli".to_string()),
        ..Default::default()
    };
    let mut target = CgToken {
        begin: 0,
        end: 0,
        readings: vec![reading(&["\"guolli\"", "N", "Sg", "Nom"])],
    };

    annotator.copy(&source, &mut target);

    assert_eq!(target.begin, 12);
    assert_eq!(target.end, 18);
    assert_eq!(
        target.readings,
        vec![reading(&["\"guolli\"", "N", "Sg", "Nom"])]
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.to-cg3-input-fn/test]
#[test]
fn to_cg3_input_writes_token_per_line() {
    let annotator = Vislcg3Annotator::new();
    let text = "Mun boran";
    let tokens = [token(0, 3), token(4, 9)];

    let out = annotator.to_cg3_input(text, &tokens, &[]).unwrap();

    assert_eq!(out, "Mun\nboran\n");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.to-cg3-input-fn/test]
#[test]
fn to_cg3_input_injects_period_at_unpunctuated_end() {
    let annotator = Vislcg3Annotator::new();
    let text = "Oahppa";
    let tokens = [token(0, 6)];
    let sentences = [SentenceAnnotation { begin: 0, end: 6 }];

    let out = annotator.to_cg3_input(text, &tokens, &sentences).unwrap();

    assert_eq!(out, "Oahppa\n.\n");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.to-cg3-input-fn/test]
#[test]
fn to_cg3_input_suppresses_period_after_punctuation_token() {
    let annotator = Vislcg3Annotator::new();
    let text = "Mii!?";
    let tokens = [token(0, 3), token(3, 5)];
    let sentences = [SentenceAnnotation { begin: 0, end: 5 }];

    let out = annotator.to_cg3_input(text, &tokens, &sentences).unwrap();

    assert_eq!(out, "Mii\n!?\n");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.to-cg3-input-fn/test]
#[test]
fn to_cg3_input_injects_period_after_dotted_word() {
    let annotator = Vislcg3Annotator::new();
    let text = "sh.";
    let tokens = [token(0, 3)];
    let sentences = [SentenceAnnotation { begin: 0, end: 3 }];

    let out = annotator.to_cg3_input(text, &tokens, &sentences).unwrap();

    assert_eq!(out, "sh.\n.\n");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.to-cg3-input-fn/test]
#[test]
fn to_cg3_input_reports_span_outside_text() {
    let annotator = Vislcg3Annotator::new();
    let tokens = [token(0, 42)];

    let err = annotator.to_cg3_input("Mun", &tokens, &[]).unwrap_err();

    assert!(
        err.to_string()
            .contains("span 0..42 is not within the document text"),
        "unexpected error: {err}"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+3/test]
#[test]
fn parse_cg_output_builds_token_and_drops_surface() {
    let annotator = Vislcg3Annotator::new();
    let cg =
        "\"<Mun>\"\n\t\"mun\" Pron Pers Sg1 Nom\n\n\"<boran>\"\n\t\"borrat\" V IV Ind Prs Sg1\n";

    let tokens = annotator.parse_cg_output(cg).unwrap();

    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].begin, 0);
    assert_eq!(tokens[0].end, 0);
    assert_eq!(
        tokens[0].readings,
        vec![reading(&["\"mun\"", "Pron", "Pers", "Sg1", "Nom"])]
    );
    assert_eq!(
        tokens[1].readings,
        vec![reading(&["\"borrat\"", "V", "IV", "Ind", "Prs", "Sg1"])]
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+3/test]
#[test]
fn parse_cg_output_drops_indent_keeps_later_readings() {
    let annotator = Vislcg3Annotator::new();
    let cg = "\"<lea>\"\n\t\"leat\" V IV Ind Prs Sg3\n\t\"leat\" V IV Imprt Sg2\n";

    let tokens = annotator.parse_cg_output(cg).unwrap();

    assert_eq!(tokens.len(), 1);
    assert_eq!(
        tokens[0].readings,
        vec![
            reading(&["\"leat\"", "V", "IV", "Ind", "Prs", "Sg3"]),
            reading(&["\"leat\"", "V", "IV", "Imprt", "Sg2"]),
        ]
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+3/test]
#[test]
fn parse_cg_output_collapses_blanks_leaving_bare_cohorts() {
    let annotator = Vislcg3Annotator::new();
    let cg = "\"<Mun>\"\n\n\n\"<boran>\"\n";

    let tokens = annotator.parse_cg_output(cg).unwrap();

    assert_eq!(tokens.len(), 2);
    assert!(tokens[0].readings.is_empty());
    assert!(tokens[1].readings.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+3/test]
#[test]
fn parse_cg_output_discards_readings_before_first_cohort() {
    let annotator = Vislcg3Annotator::new();
    let cg = "\n\t\"orphan\" N Sg Nom\n\"<Mun>\"\n\t\"mun\" Pron\n";

    let tokens = annotator.parse_cg_output(cg).unwrap();

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].readings, vec![reading(&["\"mun\"", "Pron"])]);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+3/test]
#[test]
fn parse_cg_output_without_cohort_header_yields_nothing() {
    let annotator = Vislcg3Annotator::new();

    assert!(annotator.parse_cg_output("").unwrap().is_empty());
    assert!(
        annotator
            .parse_cg_output("\t\"mun\" Pron\n")
            .unwrap()
            .is_empty()
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+3/test]
#[test]
fn parse_cg_output_skips_escaped_blank_markers() {
    let annotator = Vislcg3Annotator::new();
    let cg = "\"<Mun>\"\n\t\"mun\" Pron Pers Sg1 Nom @SUBJ>\n:\\n\n\"<.>\"\n\t\".\" CLB\n:\\n\n";

    let tokens = annotator.parse_cg_output(cg).unwrap();

    assert_eq!(tokens.len(), 2);
    assert_eq!(
        tokens[0].readings,
        vec![reading(&[
            "\"mun\"", "Pron", "Pers", "Sg1", "Nom", "@SUBJ>"
        ])]
    );
    assert_eq!(tokens[1].readings, vec![reading(&["\".\"", "CLB"])]);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+3/test]
#[test]
fn parse_cg_output_drops_weight_and_tracking_tags() {
    let annotator = Vislcg3Annotator::new();
    let cg = concat!(
        "\"<viesu>\"\n",
        "\t\"viessu\" N Sem/Build Sg Acc <W:0.0> <firstCohortOfParagraph> ",
        "<LastCohortOfParagraph> @<OBJ\n",
        "\t\"viessu\" N <W:1.5> <firstCohort> <LastCohort> Sg Gen\n"
    );

    let tokens = annotator.parse_cg_output(cg).unwrap();

    assert_eq!(
        tokens[0].readings,
        vec![
            reading(&["\"viessu\"", "N", "Sem/Build", "Sg", "Acc", "@<OBJ"]),
            reading(&["\"viessu\"", "N", "Sg", "Gen"]),
        ]
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+3/test]
#[test]
fn parse_cg_output_keeps_semantic_angle_bracket_tags() {
    let annotator = Vislcg3Annotator::new();
    let cg = "\"<viesu>\"\n\t\"viessu\" N <sme> Sg Acc <W:0.0>\n";

    let tokens = annotator.parse_cg_output(cg).unwrap();

    assert_eq!(
        tokens[0].readings,
        vec![reading(&["\"viessu\"", "N", "<sme>", "Sg", "Acc"])]
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+3/test]
#[test]
fn parse_cg_output_folds_subreading_into_parent() {
    let annotator = Vislcg3Annotator::new();
    let cg = "\"<girjeráju>\"\n\t\"rádju\" N Sg Gen\n\t\t\"girji\" N Cmp/SgNom Cmp\n\t\"rádju\" N Sg Acc\n";

    let tokens = annotator.parse_cg_output(cg).unwrap();

    assert_eq!(
        tokens[0].readings,
        vec![
            reading(&[
                "\"rádju\"",
                "N",
                "Sg",
                "Gen",
                "\"girji\"",
                "N",
                "Cmp/SgNom",
                "Cmp"
            ]),
            reading(&["\"rádju\"", "N", "Sg", "Acc"]),
        ]
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+3/test]
#[test]
fn parse_cg_output_reports_index_error_on_whitespace() {
    let annotator = Vislcg3Annotator::new();
    let cg = "\"<Mun>\"\n \t\n";

    let err = annotator.parse_cg_output(cg).unwrap_err();

    assert_eq!(err.to_string(), "Index -1 out of bounds for length 0");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.process-fn/test]
#[test]
fn process_propagates_token_span_outside_text() {
    let annotator = Vislcg3Annotator::new();
    let mut jcas = Document::new("guolli", PIPELINE_LANGUAGE);
    jcas.tokens.push(token(0, 99));

    let err = annotator.process(&mut jcas).unwrap_err();

    assert!(
        err.to_string()
            .contains("span 0..99 is not within the document text"),
        "unexpected error: {err}"
    );
    assert_eq!(jcas.tokens.len(), 1);
    assert!(jcas.cg_tokens.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.process-fn/test]
#[test]
fn process_of_a_document_without_tokens_indexes_nothing() {
    let annotator = Vislcg3Annotator::new();
    let mut jcas = Document::new("guolli", PIPELINE_LANGUAGE);

    let _ = annotator.process(&mut jcas);

    assert!(jcas.tokens.is_empty());
    assert!(jcas.cg_tokens.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.run-fst-cg-fn+2/test]
#[test]
fn run_fst_cg_terminates_lines_or_reports_failure() {
    let annotator = Vislcg3Annotator::new();

    match annotator.run_fst_cg("Mun\nboran\n") {
        Ok(cg3output) => {
            assert!(cg3output.is_empty() || cg3output.ends_with('\n'));
            assert!(!cg3output.contains("\r\n"));
        }
        Err(e) => {
            let message = format!("{e:#}");
            assert!(
                message.contains("analyze") || message.contains(crate::morpho::BUNDLE_ENV),
                "unexpected error: {message}"
            );
        }
    }
}
