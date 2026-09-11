//! What the non-finite-verb topic does that its siblings do not: the
//! non-finite pattern and the two generator inputs. The pass they all share
//! is exercised in `cg_enhancer_tests`.

use super::*;
use crate::test_support::reading;
use crate::types::CgToken;
use crate::types::PIPELINE_LANGUAGE;
use crate::util::cas_utils;

/// The canned CG readings the tests draw on, as raw tag elements: an
/// infinitive and a perfect participle (both matched by the enhancer), a
/// finite verb form and a noun (both passed over).
const INF: &[&str] = &["\"boahtit\"", "V", "<sme>", "Inf"];
const PRF_PRC: &[&str] = &["\"boahtit\"", "V", "<sme>", "PrfPrc"];
const FINITE: &[&str] = &["\"boahtit\"", "V", "<sme>", "Ind", "Prs", "Sg3"];
const NOUN: &[&str] = &["\"beana\"", "N", "<sme>", "Pl", "Nom"];

fn cg_token(begin: usize, end: usize, readings: Vec<Vec<String>>) -> CgToken {
    CgToken {
        begin,
        end,
        readings,
    }
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.initialize-fn/test]
#[test]
fn initialize_splits_the_value_on_commas_without_trimming() {
    let mut enhancer = Vislcg3InfiniteVerbEnhancer::default();
    assert!(enhancer.infverb_tags.is_none());

    enhancer
        .initialize(Some(
            "V PrfPrc, V VGen, V VAbess, V Ger, V Actio Ess, V Inf, Ind Prs ConNeg, Ind Prt ConNeg",
        ))
        .expect("configured");

    assert_eq!(
        enhancer.infverb_tags.clone().expect("stored"),
        vec![
            "V PrfPrc",
            " V VGen",
            " V VAbess",
            " V Ger",
            " V Actio Ess",
            " V Inf",
            " Ind Prs ConNeg",
            " Ind Prt ConNeg",
        ]
    );

    // A value without a comma is a single-element list, and an empty value
    // is a list holding one empty element.
    enhancer.initialize(Some("V Inf")).expect("configured");
    assert_eq!(
        enhancer.infverb_tags.clone().expect("stored"),
        vec!["V Inf"]
    );
    enhancer.initialize(Some("")).expect("configured");
    assert_eq!(enhancer.infverb_tags.clone().expect("stored"), vec![""]);

    // An absent parameter fails instead of being defaulted.
    let mut unset = Vislcg3InfiniteVerbEnhancer::default();
    let err = unset.initialize(None).expect_err("no parameter");
    assert!(err.to_string().contains("infiniteverbTags"));
    assert!(unset.infverb_tags.is_none());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.process-fn+6/test]
#[test]
fn process_spans_infinite_verbs_and_numbers_repeated_readings() {
    let enhancer = Vislcg3InfiniteVerbEnhancer::default();
    let mut doc = Document::new("boahtit boahtit", PIPELINE_LANGUAGE);
    doc.cg_tokens = vec![
        cg_token(0, 7, vec![reading(INF)]),
        cg_token(8, 15, vec![reading(INF)]),
    ];

    enhancer.process(&mut doc, Mode::Colorize).expect("process");

    let first = concat!(
        "<span id=\"teaksta-span-boahtit-V-xsmey-Inf-1\" ",
        "class=\"teaksta-token teaksta-InfiniteVerbs\" lemma=\"boahtit\">"
    );
    let second = concat!(
        "<span id=\"teaksta-span-boahtit-V-xsmey-Inf-2\" ",
        "class=\"teaksta-token teaksta-InfiniteVerbs\" lemma=\"boahtit\">"
    );

    assert_eq!(doc.enhancements.len(), 2);
    assert!(doc.enhancements[0].relevant);
    assert_eq!(doc.enhancements[0].begin, 0);
    assert_eq!(doc.enhancements[0].end, 7);
    assert_eq!(doc.enhancements[0].enhance_start, first);
    assert_eq!(doc.enhancements[0].enhance_end, "</span>");
    assert_eq!(doc.enhancements[1].begin, 8);
    assert_eq!(doc.enhancements[1].end, 15);
    assert_eq!(doc.enhancements[1].enhance_start, second);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.process-fn+6/test]
#[test]
fn process_selects_the_first_reading_matching_both_patterns() {
    let enhancer = Vislcg3InfiniteVerbEnhancer::default();
    let mut doc = Document::new("boahtit", PIPELINE_LANGUAGE);
    doc.cg_tokens = vec![cg_token(
        0,
        7,
        vec![reading(FINITE), reading(INF), reading(PRF_PRC)],
    )];

    enhancer.process(&mut doc, Mode::Colorize).expect("process");

    assert_eq!(doc.enhancements.len(), 1);
    assert_eq!(
        doc.enhancements[0].enhance_start,
        concat!(
            "<span id=\"teaksta-span-boahtit-V-xsmey-Inf-1\" ",
            "class=\"teaksta-token teaksta-InfiniteVerbs\" lemma=\"boahtit\">"
        )
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.process-fn+6/test]
#[test]
fn process_ignores_tokens_without_infinite_verb_readings() {
    let enhancer = Vislcg3InfiniteVerbEnhancer::default();
    let mut doc = Document::new("boahtá beana", PIPELINE_LANGUAGE);
    doc.cg_tokens = vec![
        cg_token(0, 6, vec![reading(FINITE)]),
        cg_token(7, 12, vec![reading(NOUN)]),
        cg_token(13, 13, vec![]),
    ];

    enhancer.process(&mut doc, Mode::Colorize).expect("process");

    assert!(doc.enhancements.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.process-fn+6/test]
#[test]
fn process_returns_early_when_the_cas_was_cancelled() {
    let enhancer = Vislcg3InfiniteVerbEnhancer::default();
    let mut doc = Document::new("boahtit", PIPELINE_LANGUAGE);
    doc.cg_tokens = vec![cg_token(0, 7, vec![reading(INF)])];
    cas_utils::add_enh_id(&mut doc, -1);

    enhancer.process(&mut doc, Mode::Colorize).expect("process");

    assert!(doc.enhancements.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.process-fn+6/test]
#[test]
fn process_selection_ignores_the_configured_tags() {
    let configured =
        Vislcg3InfiniteVerbEnhancer::new(Some("N Pl Nom, N Pl Gen")).expect("configured");
    let mut doc = Document::new("boahtit", PIPELINE_LANGUAGE);
    doc.cg_tokens = vec![cg_token(0, 7, vec![reading(INF)])];

    configured
        .process(&mut doc, Mode::Colorize)
        .expect("process");

    // The infinite-verb reading is still selected and the plural-noun
    // reading is still rejected, whatever the parameter said.
    assert_eq!(doc.enhancements.len(), 1);
    assert_eq!(doc.enhancements[0].begin, 0);

    let mut nouns = Document::new("beanat", PIPELINE_LANGUAGE);
    nouns.cg_tokens = vec![cg_token(0, 6, vec![reading(NOUN)])];
    configured
        .process(&mut nouns, Mode::Colorize)
        .expect("process");
    assert!(nouns.enhancements.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.write-morphological-forms-fn+2/test]
#[test]
fn write_morphological_forms_emits_twelve_then_the_answer() {
    let enhancer = Vislcg3InfiniteVerbEnhancer::default();

    let block = enhancer
        .write_morphological_forms("boahtit+V+<sme>+Inf")
        .expect("built");
    let lines: Vec<&str> = block.lines().collect();

    assert_eq!(lines.len(), 13);
    assert_eq!(
        lines[..12],
        [
            "boahtit+V+Ind+Prs+Sg1",
            "boahtit+V+Ind+Prs+Sg2",
            "boahtit+V+Ind+Prs+Sg3",
            "boahtit+V+Ind+Prs+Du1",
            "boahtit+V+Ind+Prs+Du2",
            "boahtit+V+Ind+Prs+Du3",
            "boahtit+V+Ind+Prt+Sg1",
            "boahtit+V+Ind+Prt+Sg2",
            "boahtit+V+Ind+Prt+Sg3",
            "boahtit+V+Ind+Prt+Du1",
            "boahtit+V+Ind+Prt+Du2",
            "boahtit+V+Ind+Prt+Du3",
        ]
    );
    // The correct answer is last, with the angle tag stripped on the way
    // out; every line, including the last, is newline-terminated.
    assert_eq!(lines[12], "boahtit+V+Inf");
    assert!(block.ends_with("boahtit+V+Ind+Prt+Du3\nboahtit+V+Inf\n"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.write-morphological-forms-fn+2/test]
#[test]
fn write_morphological_forms_cuts_the_function_tag() {
    let enhancer = Vislcg3InfiniteVerbEnhancer::default();

    let block = enhancer
        .write_morphological_forms("boahtit+V+<sme>+Inf+@-FMAINV")
        .expect("built");
    assert_eq!(block.lines().last(), Some("boahtit+V+Inf"));

    // An `@` at the very front is not a syntactic function tag, so the
    // answer line keeps the whole reading and the lemma keeps the `@`.
    let leading = enhancer
        .write_morphological_forms("@boahtit+V+Inf")
        .expect("built");
    let lines: Vec<&str> = leading.lines().collect();
    assert_eq!(lines[0], "@boahtit+V+Ind+Prs+Sg1");
    assert_eq!(lines[12], "@boahtit+V+Inf");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.write-morphological-forms-fn+2/test]
#[test]
fn write_morphological_forms_fails_without_a_plus() {
    let enhancer = Vislcg3InfiniteVerbEnhancer::default();

    let err = enhancer
        .write_morphological_forms("boahtit")
        .expect_err("no plus");
    assert_eq!(err.to_string(), "begin 0, end -1, length 7");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.write-lemma-and-analyses-fn+2/test]
#[test]
fn write_lemma_and_analyses_chops_the_final_character() {
    let enhancer = Vislcg3InfiniteVerbEnhancer::default();

    assert_eq!(
        enhancer
            .write_lemma_and_analyses("boahtit+V+Inf")
            .expect("built"),
        "boahtit+V+In\n"
    );
    // The angle tag is removed before the chop is visible, but the chop
    // still lands on the final tag.
    assert_eq!(
        enhancer
            .write_lemma_and_analyses("boahtit+V+<sme>+Inf")
            .expect("built"),
        "boahtit+V+In\n"
    );
    // A reading whose analyses are exactly one character long collapses to
    // an empty analysis list.
    assert_eq!(
        enhancer.write_lemma_and_analyses("a+b").expect("built"),
        "a+\n"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.write-lemma-and-analyses-fn+2/test]
#[test]
fn write_lemma_and_analyses_truncates_at_function_tag() {
    let enhancer = Vislcg3InfiniteVerbEnhancer::default();

    // Truncating at `@` happens after the chop, so the final tag survives
    // intact here.
    assert_eq!(
        enhancer
            .write_lemma_and_analyses("boahtit+V+<sme>+Inf+@-FMAINV")
            .expect("built"),
        "boahtit+V+Inf\n"
    );
    // Error and allegro tags are stripped from the assembled line.
    assert_eq!(
        enhancer
            .write_lemma_and_analyses("boahtit+V+Allegro+Infx")
            .expect("built"),
        "boahtit+V+Inf\n"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.write-lemma-and-analyses-fn+2/test]
#[test]
fn write_lemma_and_analyses_fails_on_unsliceable_readings() {
    let enhancer = Vislcg3InfiniteVerbEnhancer::default();

    let err = enhancer
        .write_lemma_and_analyses("boahtit")
        .expect_err("no plus");
    assert_eq!(err.to_string(), "begin 0, end -1, length 7");

    // A reading ending on its first `+` gives an inverted slice.
    let err = enhancer
        .write_lemma_and_analyses("a+")
        .expect_err("empty analyses");
    assert_eq!(err.to_string(), "begin 2, end 1, length 2");
}
