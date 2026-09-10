use super::*;

use std::cell::Cell;
use std::io::{self, BufReader, Cursor, Read};
use std::rc::Rc;

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

/// Hands out `data` in one read, then fails every read after it, counting
/// how often it was polled.
struct ReadThenFail {
    data: Vec<u8>,
    reads: Rc<Cell<usize>>,
}

impl ReadThenFail {
    fn new(data: &str) -> (Self, Rc<Cell<usize>>) {
        let reads = Rc::new(Cell::new(0));
        (
            ReadThenFail {
                data: data.as_bytes().to_vec(),
                reads: Rc::clone(&reads),
            },
            reads,
        )
    }
}

impl Read for ReadThenFail {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reads.set(self.reads.get() + 1);
        if self.data.is_empty() {
            return Err(io::Error::other("external command went away"));
        }
        let taken = self.data.len().min(buf.len());
        buf[..taken].copy_from_slice(&self.data[..taken]);
        self.data.drain(..taken);
        Ok(taken)
    }
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
    let mut jcas = Document::new("guolli", "sme");
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
    let mut jcas = Document::new("guolli", "sme");

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

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger.ext-command-consume2-logger-fn/test]
#[test]
fn logger_consumer_keeps_prefix_and_reads_nothing() {
    let mut consumer = ExtCommandConsume2Logger::new(
        Cursor::new(b"VislCG3 warning\n".to_vec()),
        "VislCG STDERR: ".to_string(),
    );

    assert_eq!(consumer.msg_prefix, "VislCG STDERR: ");
    let mut untouched = String::new();
    consumer.reader.read_to_string(&mut untouched).unwrap();
    assert_eq!(untouched, "VislCG3 warning\n");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger.run-fn/test]
#[test]
fn logger_consumer_drains_the_reader_to_stream_end() {
    let mut consumer = ExtCommandConsume2Logger::new(
        Cursor::new(b"first\nsecond\nthird".to_vec()),
        "prefix".to_string(),
    );

    consumer.run();

    let mut left = String::new();
    consumer.reader.read_to_string(&mut left).unwrap();
    assert_eq!(left, "");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger.run-fn/test]
#[test]
fn logger_consumer_swallows_a_read_error_and_stops() {
    let (source, reads) = ReadThenFail::new("first\nsecond\n");
    let mut consumer = ExtCommandConsume2Logger::new(BufReader::new(source), "prefix".to_string());

    consumer.run();

    assert_eq!(reads.get(), 2);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.ext-command-consume2-string-fn/test]
#[test]
fn string_consumer_starts_unfinished_with_an_empty_buffer() {
    let mut consumer = ExtCommandConsume2String::new(Cursor::new(b"output\n".to_vec()));

    assert!(!consumer.finished);
    assert_eq!(consumer.buffer, "");
    let mut untouched = String::new();
    consumer.reader.read_to_string(&mut untouched).unwrap();
    assert_eq!(untouched, "output\n");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.run-fn/test]
#[test]
fn string_consumer_terminates_every_line_it_collects() {
    let mut consumer =
        ExtCommandConsume2String::new(Cursor::new(b"\"<Mun>\"\n\t\"mun\" Pron".to_vec()));

    consumer.run();

    assert_eq!(consumer.buffer, "\"<Mun>\"\n\t\"mun\" Pron\n");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.run-fn/test]
#[test]
fn string_consumer_normalises_crlf_and_handles_empty_stream() {
    let mut crlf = ExtCommandConsume2String::new(Cursor::new(b"one\r\ntwo\r\n".to_vec()));
    crlf.run();
    assert_eq!(crlf.buffer, "one\ntwo\n");

    let mut empty = ExtCommandConsume2String::new(Cursor::new(Vec::new()));
    empty.run();
    assert_eq!(empty.buffer, "");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.run-fn/test]
#[test]
fn string_consumer_keeps_partial_read_and_finishes() {
    let (source, reads) = ReadThenFail::new("kept\n");
    let mut consumer = ExtCommandConsume2String::new(BufReader::new(source));

    consumer.run();

    assert_eq!(consumer.buffer, "kept\n");
    assert!(consumer.finished);
    assert_eq!(reads.get(), 2);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.is-done-fn/test]
#[test]
fn is_done_reports_the_drain_stopped_either_way() {
    let mut clean = ExtCommandConsume2String::new(Cursor::new(b"line\n".to_vec()));
    assert!(!clean.is_done());
    clean.run();
    assert!(clean.is_done());

    let (source, _reads) = ReadThenFail::new("line\n");
    let mut aborted = ExtCommandConsume2String::new(BufReader::new(source));
    aborted.run();
    assert!(aborted.is_done());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.get-buffer-fn/test]
#[test]
fn get_buffer_withholds_text_until_drain_stops() {
    let mut consumer = ExtCommandConsume2String::new(Cursor::new(b"one\ntwo\n".to_vec()));

    assert_eq!(consumer.get_buffer(), None);

    consumer.run();

    assert_eq!(consumer.get_buffer(), Some("one\ntwo\n"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.get-buffer-fn/test]
#[test]
fn get_buffer_hands_back_truncated_capture_after_error() {
    let (source, _reads) = ReadThenFail::new("only this much\n");
    let mut consumer = ExtCommandConsume2String::new(BufReader::new(source));

    consumer.run();

    assert_eq!(consumer.get_buffer(), Some("only this much\n"));
}
