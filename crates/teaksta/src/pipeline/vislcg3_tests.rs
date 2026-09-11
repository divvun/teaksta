use super::*;
use crate::types::{PIPELINE_LANGUAGE, is_punctuation_cohort};

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
    let annotator = Vislcg3Annotator::default();
    let source = Token {
        begin: 12,
        end: 18,
        tag: Some("N".to_string()),
        lemma: Some("guolli".to_string()),
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

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.to-cg3-input-fn+1/test]
#[test]
fn to_cg3_input_writes_token_per_line() {
    let annotator = Vislcg3Annotator::default();
    let text = "Mun boran";
    let tokens = [token(0, 3), token(4, 9)];

    let out = annotator.to_cg3_input(text, &tokens, &[]).unwrap();

    assert_eq!(cg3_input_text(&out), "Mun\nboran\n");
    // every line names the token it was taken from
    assert_eq!(
        out.iter().map(|line| line.token).collect::<Vec<_>>(),
        vec![Some(0), Some(1)]
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.to-cg3-input-fn+1/test]
#[test]
fn to_cg3_input_injects_period_at_unpunctuated_end() {
    let annotator = Vislcg3Annotator::default();
    let text = "Oahppa";
    let tokens = [token(0, 6)];
    let sentences = [SentenceAnnotation { begin: 0, end: 6 }];

    let out = annotator.to_cg3_input(text, &tokens, &sentences).unwrap();

    assert_eq!(cg3_input_text(&out), "Oahppa\n.\n");
    // the injected boundary stands for no text in the document, so it names
    // no token and nothing the analyser answers it with can be placed
    assert_eq!(
        out.iter().map(|line| line.token).collect::<Vec<_>>(),
        vec![Some(0), None]
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.to-cg3-input-fn+1/test]
#[test]
fn to_cg3_input_suppresses_period_after_punctuation_token() {
    let annotator = Vislcg3Annotator::default();
    let text = "Mii!?";
    let tokens = [token(0, 3), token(3, 5)];
    let sentences = [SentenceAnnotation { begin: 0, end: 5 }];

    let out = annotator.to_cg3_input(text, &tokens, &sentences).unwrap();

    assert_eq!(cg3_input_text(&out), "Mii\n!?\n");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.to-cg3-input-fn+1/test]
#[test]
fn to_cg3_input_injects_period_after_dotted_word() {
    let annotator = Vislcg3Annotator::default();
    let text = "sh.";
    let tokens = [token(0, 3)];
    let sentences = [SentenceAnnotation { begin: 0, end: 3 }];

    let out = annotator.to_cg3_input(text, &tokens, &sentences).unwrap();

    assert_eq!(cg3_input_text(&out), "sh.\n.\n");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.to-cg3-input-fn+1/test]
#[test]
fn to_cg3_input_reports_span_outside_text() {
    let annotator = Vislcg3Annotator::default();
    let tokens = [token(0, 42)];

    let err = annotator.to_cg3_input("Mun", &tokens, &[]).unwrap_err();

    assert!(
        err.to_string()
            .contains("span 0..42 is not within the document text"),
        "unexpected error: {err}"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+5/test]
#[test]
fn parse_cg_output_builds_cohort_and_keeps_surface() {
    let annotator = Vislcg3Annotator::default();
    let cg =
        "\"<Mun>\"\n\t\"mun\" Pron Pers Sg1 Nom\n\n\"<boran>\"\n\t\"borrat\" V IV Ind Prs Sg1\n";

    let tokens = annotator.parse_cg_output(cg);

    assert_eq!(tokens.len(), 2);
    // the wordform is the analyser's own statement about which text the
    // readings under it are about, so it is kept
    assert_eq!(tokens[0].form, "Mun");
    assert_eq!(tokens[1].form, "boran");
    assert_eq!(
        tokens[0].readings,
        vec![reading(&["\"mun\"", "Pron", "Pers", "Sg1", "Nom"])]
    );
    assert_eq!(
        tokens[1].readings,
        vec![reading(&["\"borrat\"", "V", "IV", "Ind", "Prs", "Sg1"])]
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+5/test]
#[test]
fn parse_cg_output_drops_indent_keeps_later_readings() {
    let annotator = Vislcg3Annotator::default();
    let cg = "\"<lea>\"\n\t\"leat\" V IV Ind Prs Sg3\n\t\"leat\" V IV Imprt Sg2\n";

    let tokens = annotator.parse_cg_output(cg);

    assert_eq!(tokens.len(), 1);
    assert_eq!(
        tokens[0].readings,
        vec![
            reading(&["\"leat\"", "V", "IV", "Ind", "Prs", "Sg3"]),
            reading(&["\"leat\"", "V", "IV", "Imprt", "Sg2"]),
        ]
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+5/test]
#[test]
fn parse_cg_output_skips_cohorts_without_a_reading() {
    let annotator = Vislcg3Annotator::default();

    // two adjacent headers, and a header with nothing after it
    assert!(
        annotator
            .parse_cg_output("\"<Mun>\"\n\n\n\"<boran>\"\n")
            .is_empty()
    );

    // the readingless cohorts go and the rest of the stream survives
    let cg = "\"<Mun>\"\n\"<boran>\"\n\t\"borrat\" V IV Ind Prs Sg1\n\"<guoli>\"\n";
    let tokens = annotator.parse_cg_output(cg);

    assert_eq!(tokens.len(), 1);
    assert_eq!(
        tokens[0].readings,
        vec![reading(&["\"borrat\"", "V", "IV", "Ind", "Prs", "Sg1"])]
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+5/test]
#[test]
fn parse_cg_output_discards_readings_before_first_cohort() {
    let annotator = Vislcg3Annotator::default();
    let cg = "\n\t\"orphan\" N Sg Nom\n\"<Mun>\"\n\t\"mun\" Pron\n";

    let tokens = annotator.parse_cg_output(cg);

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].readings, vec![reading(&["\"mun\"", "Pron"])]);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+5/test]
#[test]
fn parse_cg_output_without_cohort_header_yields_nothing() {
    let annotator = Vislcg3Annotator::default();

    assert!(annotator.parse_cg_output("").is_empty());
    assert!(annotator.parse_cg_output("\t\"mun\" Pron\n").is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+5/test]
#[test]
fn parse_cg_output_skips_escaped_blank_markers() {
    let annotator = Vislcg3Annotator::default();
    let cg = "\"<Mun>\"\n\t\"mun\" Pron Pers Sg1 Nom @SUBJ>\n:\\n\n\"<.>\"\n\t\".\" CLB\n:\\n\n";

    let tokens = annotator.parse_cg_output(cg);

    assert_eq!(tokens.len(), 2);
    assert_eq!(
        tokens[0].readings,
        vec![reading(&[
            "\"mun\"", "Pron", "Pers", "Sg1", "Nom", "@SUBJ>"
        ])]
    );
    assert_eq!(tokens[1].readings, vec![reading(&["\".\"", "CLB"])]);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+5/test]
#[test]
fn parse_cg_output_drops_weight_and_tracking_tags() {
    let annotator = Vislcg3Annotator::default();
    let cg = concat!(
        "\"<viesu>\"\n",
        "\t\"viessu\" N Sem/Build Sg Acc <W:0.0> <firstCohortOfParagraph> ",
        "<LastCohortOfParagraph> @<OBJ\n",
        "\t\"viessu\" N <W:1.5> <firstCohort> <LastCohort> Sg Gen\n"
    );

    let tokens = annotator.parse_cg_output(cg);

    assert_eq!(
        tokens[0].readings,
        vec![
            reading(&["\"viessu\"", "N", "Sem/Build", "Sg", "Acc", "@<OBJ"]),
            reading(&["\"viessu\"", "N", "Sg", "Gen"]),
        ]
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+5/test]
#[test]
fn parse_cg_output_keeps_semantic_angle_bracket_tags() {
    let annotator = Vislcg3Annotator::default();
    let cg = "\"<viesu>\"\n\t\"viessu\" N <sme> Sg Acc <W:0.0>\n";

    let tokens = annotator.parse_cg_output(cg);

    assert_eq!(
        tokens[0].readings,
        vec![reading(&["\"viessu\"", "N", "<sme>", "Sg", "Acc"])]
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+5/test]
#[test]
fn parse_cg_output_folds_subreading_into_parent() {
    let annotator = Vislcg3Annotator::default();
    let cg = "\"<girjeráju>\"\n\t\"rádju\" N Sg Gen\n\t\t\"girji\" N Cmp/SgNom Cmp\n\t\"rádju\" N Sg Acc\n";

    let tokens = annotator.parse_cg_output(cg);

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

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn+5/test]
#[test]
fn parse_cg_output_tolerates_whitespace_only_lines() {
    let annotator = Vislcg3Annotator::default();
    let cg = "\"<Mun>\"\n \t\n\t\"mun\" Pron Pers Sg1 Nom\n   \n\"<boran>\"\n\t\"borrat\" V\n";

    let tokens = annotator.parse_cg_output(cg);

    assert_eq!(tokens.len(), 2);
    assert_eq!(
        tokens[0].readings,
        vec![reading(&["\"mun\"", "Pron", "Pers", "Sg1", "Nom"])]
    );
    assert_eq!(tokens[1].readings, vec![reading(&["\"borrat\"", "V"])]);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.process-fn+4/test]
#[test]
fn the_wordform_is_read_off_the_cohort_header() {
    assert_eq!(cohort_form("\"<Mun>\""), "Mun");
    assert_eq!(
        cohort_form("\"<Finnmárkku duottar>\""),
        "Finnmárkku duottar"
    );
    // a form written with the wrapper's own characters in it
    assert_eq!(cohort_form("\"<a\\\"b>\""), "a\"b");
    assert_eq!(cohort_form("\"<>\"a>\""), ">\"a");
    // a header the wrapper cannot be found in names nothing
    assert_eq!(cohort_form("\"<Mun"), "");
    assert_eq!(cohort_form("\t\"mun\" Pron"), "");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.process-fn+4/test]
#[test]
fn the_logged_reading_is_the_flattened_tag_sequence() {
    let readings = vec![reading(&["\"guolli\"", "N", "Sg", "Nom"])];

    assert_eq!(first_reading(&readings), "\"guolli\" N Sg Nom ");
    assert_eq!(first_reading(&[]), "");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.process-fn+4/test]
#[test]
fn process_propagates_token_span_outside_text() {
    let annotator = Vislcg3Annotator::default();
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

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.process-fn+4/test]
#[test]
fn process_of_a_document_without_tokens_indexes_nothing() {
    let annotator = Vislcg3Annotator::default();
    let mut jcas = Document::new("guolli", PIPELINE_LANGUAGE);

    let _ = annotator.process(&mut jcas);

    assert!(jcas.tokens.is_empty());
    assert!(jcas.cg_tokens.is_empty());
}

/// The document the mapping tests run over, with one token per whitespace-
/// separated word of `text` and the whole of it one sentence.
fn tokenised(text: &str) -> Document {
    let mut doc = Document::new(text, PIPELINE_LANGUAGE);
    let mut at = 0usize;
    for word in text.split(' ') {
        if !word.is_empty() {
            doc.tokens.push(token(at, at + word.len()));
        }
        at += word.len() + 1;
    }
    doc.sentences.push(SentenceAnnotation {
        begin: 0,
        end: text.len(),
    });
    doc
}

/// The base form of a CG token's first reading, stripped of its quotes.
fn lemma(cg: &CgToken) -> String {
    cg.readings
        .first()
        .and_then(|reading| reading.first())
        .map(|field| field.trim_matches('"').to_string())
        .unwrap_or_default()
}

/// One cohort of a synthetic stream: the wordform header and one reading.
fn cohort(form: &str, reading: &str) -> String {
    format!("\"<{form}>\"\n\t{reading}\n:\\n\n")
}

/// The stream the analyser answers for `Skánddat várreviđji nohká guovllu.`:
/// a dynamic compound mid-sentence, and — the cohort count the tokeniser did
/// not ask for — the multiword `Finnmárkku duottar` split back into its two
/// constituents by `mwesplit`.
fn split_stream() -> String {
    [
        cohort("Skánddat", "\"Skánda\" N Prop Sem/Plc Pl Nom @HNOUN"),
        cohort(
            "várreviđji",
            "\"viđji\" N Sem/Dummytag Sg Nom <cohort-with-dynamic-compound> @SUBJ>",
        ),
        cohort("Finnmárkku", "\"Finnmárku\" N Prop Sem/Plc Sg Gen @>N"),
        cohort("duottar", "\"duottar\" N Sem/Plc Sg Nom @SUBJ>"),
        cohort("nohká", "\"nohkat\" V IV Ind Prs Sg3 @+FMAINV"),
        cohort("guovllu", "\"guovlu\" N Sem/Plc Sg Acc @<OBJ"),
        cohort(".", "\".\" CLB"),
    ]
    .concat()
}

/// The one surface word the tokeniser answered with for the multiword, so
/// the document has one token where the stream has two cohorts.
const SPLIT_TEXT: &str = "Skánddat várreviđji Finnmárkku duottar nohká guovllu .";

/// [`SPLIT_TEXT`] tokenised the way the tokeniser answers it: the multiword
/// `Finnmárkku duottar` is one token, as it is an entry of the tokeniser's
/// own multiword list, where the analyser answers it with two cohorts.
fn split_document() -> Document {
    let mut doc = tokenised(SPLIT_TEXT);
    let multiword = SPLIT_TEXT.find("Finnmárkku").expect("the multiword");
    doc.tokens.retain(|t| t.begin != multiword);
    let joined = doc
        .tokens
        .iter_mut()
        .find(|t| SPLIT_TEXT[t.begin..t.end] == *"duottar")
        .expect("the second constituent");
    joined.begin = multiword;
    doc
}

/// Every CG token as the pair a learner sees: the base form the analysis
/// gives it, and the text the span it was placed at actually covers.
fn analysed(doc: &Document) -> Vec<(String, String)> {
    doc.cg_tokens
        .iter()
        .map(|cg| (lemma(cg), doc.text[cg.begin..cg.end].to_string()))
        .collect()
}

/// The same as strings, so the expectations below read as the text does.
fn pairs(expected: &[(&str, &str)]) -> Vec<(String, String)> {
    expected
        .iter()
        .map(|(lemma, covered)| (lemma.to_string(), covered.to_string()))
        .collect()
}

/// The bug this alignment exists to stop: the analyser answering one surface
/// word with several cohorts used to shift every cohort after it onto the
/// word before, until a noun's span landed on a full stop.
// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.process-fn+4/test]
#[test]
fn a_split_group_stays_on_its_surface_word() {
    let mut doc = split_document();

    Vislcg3Annotator::default()
        .map_cg_output(&mut doc, &split_stream())
        .expect("the stream maps");

    assert_eq!(
        analysed(&doc),
        pairs(&[
            ("Skánda", "Skánddat"),
            // the dynamic compound is one cohort and keeps the whole word
            ("viđji", "várreviđji"),
            // the split multiword is two, each on its own constituent and
            // both inside the one token the tokeniser answered with
            ("Finnmárku", "Finnmárkku"),
            ("duottar", "duottar"),
            // and every word after the split still carries its own analysis
            ("nohkat", "nohká"),
            ("guovlu", "guovllu"),
            (".", "."),
        ])
    );
}

/// Nothing the analysis calls punctuation is ever covered by a reading that
/// is not its own — the property the skew broke, stated over the same stream.
// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.process-fn+4/test]
#[test]
fn no_word_reading_lands_on_punctuation() {
    let mut doc = split_document();

    Vislcg3Annotator::default()
        .map_cg_output(&mut doc, &split_stream())
        .expect("the stream maps");

    for cg in &doc.cg_tokens {
        let covered = &doc.text[cg.begin..cg.end];
        assert_eq!(
            covered.chars().any(char::is_alphabetic),
            !is_punctuation_cohort(cg),
            "{covered:?} carries {:?}",
            cg.readings
        );
    }
}

/// A cohort the stream carries that no input line accounts for — the boundary
/// period the pass injects after a heading, and a cohort for a word that is
/// not in the document at all — is dropped where it stands rather than
/// shifting the lines after it.
// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.process-fn+4/test]
#[test]
fn a_cohort_no_line_accounts_for_shifts_nothing() {
    let mut doc = tokenised("Mun boran guoli");
    let stream = [
        cohort("Mun", "\"mun\" Pron Pers Sg1 Nom @SUBJ>"),
        // the sentence boundary the pass injected after the heading
        cohort(".", "\".\" CLB"),
        cohort("boran", "\"borrat\" V TV Ind Prs Sg1 @+FMAINV"),
        // and a cohort for a word no line ever carried
        cohort("beana", "\"beana\" N Sem/Ani Sg Nom @SUBJ>"),
        cohort("guoli", "\"guolli\" N Sem/Ani Sg Acc @<OBJ"),
    ]
    .concat();

    Vislcg3Annotator::default()
        .map_cg_output(&mut doc, &stream)
        .expect("the stream maps");

    assert_eq!(
        analysed(&doc),
        pairs(&[("mun", "Mun"), ("borrat", "boran"), ("guolli", "guoli")])
    );
    assert!(doc.tokens.is_empty(), "every token was replaced");
}

/// A line the stream never answers loses its analysis and nothing else: the
/// words around it keep theirs, and the token itself stays in the store so
/// the learner still reads the word.
// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.process-fn+4/test]
#[test]
fn an_unanswered_line_keeps_its_token() {
    let mut doc = tokenised("Mun boran guoli");
    let stream = [
        cohort("Mun", "\"mun\" Pron Pers Sg1 Nom @SUBJ>"),
        cohort("guoli", "\"guolli\" N Sem/Ani Sg Acc @<OBJ"),
    ]
    .concat();

    Vislcg3Annotator::default()
        .map_cg_output(&mut doc, &stream)
        .expect("the stream maps");

    assert_eq!(
        analysed(&doc),
        pairs(&[("mun", "Mun"), ("guolli", "guoli")])
    );
    let kept: Vec<&str> = doc
        .tokens
        .iter()
        .map(|t| &doc.text[t.begin..t.end])
        .collect();
    assert_eq!(kept, vec!["boran"]);
}

/// The window the alignment may spend getting back in step is bounded, so a
/// stream that has gone its own way costs the lines their analyses rather
/// than handing them somebody else's.
// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.process-fn+4/test]
#[test]
fn resynchronising_never_reaches_past_the_window() {
    let mut doc = tokenised("Mun guoli");
    let mut stream = String::from(&cohort("Mun", "\"mun\" Pron Pers Sg1 Nom @SUBJ>"));
    for i in 0..RESYNC_WINDOW {
        stream.push_str(&cohort(&format!("sátni{i}"), "\"sátni\" N Sg Nom"));
    }
    stream.push_str(&cohort("guoli", "\"guolli\" N Sem/Ani Sg Acc @<OBJ"));

    Vislcg3Annotator::default()
        .map_cg_output(&mut doc, &stream)
        .expect("the stream maps");

    // `guoli` sits one past the window, so it is left unanalysed rather than
    // matched against one of the cohorts standing in front of it
    assert_eq!(analysed(&doc), pairs(&[("mun", "Mun")]));
}

/// The multiword's constituents are placed by their own wordforms, so each
/// span is the stretch of the document that constituent is written at.
// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.process-fn+4/test]
#[test]
fn a_split_group_is_placed_inside_its_token() {
    let mut doc = split_document();
    let multiword = doc
        .tokens
        .iter()
        .find(|t| doc.text[t.begin..t.end].contains(' '))
        .cloned()
        .expect("the joined multiword token");

    Vislcg3Annotator::default()
        .map_cg_output(&mut doc, &split_stream())
        .expect("the stream maps");

    let inside: Vec<(usize, usize)> = doc
        .cg_tokens
        .iter()
        .filter(|cg| cg.begin >= multiword.begin && cg.end <= multiword.end)
        .map(|cg| (cg.begin, cg.end))
        .collect();
    assert_eq!(inside.len(), 2, "{inside:?}");
    assert_eq!(inside[0].0, multiword.begin);
    assert_eq!(inside[1].1, multiword.end);
    // and they do not overlap, so no character is claimed twice
    assert!(inside[0].1 <= inside[1].0);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.run-fst-cg-fn+2/test]
#[test]
fn run_fst_cg_terminates_lines_or_reports_failure() {
    let annotator = Vislcg3Annotator::default();

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
