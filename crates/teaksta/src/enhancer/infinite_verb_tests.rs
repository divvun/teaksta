use super::*;
use crate::test_support::reading;
use crate::types::CgToken;

const SPAN_START: &str =
    "<span id=\"WERTi-span-boahtit-V-Inf-1\" class=\"wertiviewtoken  wertiviewInfiniteVerb\">";

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

/// A span tag in the state `process` leaves it in before handing the map
/// to one of the generator-output readers.
fn lemma_span(outer: usize) -> SpanTag {
    let mut tag = SpanTag::new(outer, SPAN_START.to_string());
    tag.add_attribute("lemma", "boahtit");
    tag
}

fn span_map(outer: usize, spans: &[(usize, usize)]) -> HashMap<Word, SpanTag> {
    spans
        .iter()
        .map(|(begin, end)| (Word::new(outer, *begin, *end), lemma_span(outer)))
        .collect()
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

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.process-fn/test]
#[test]
fn process_spans_infinite_verbs_and_numbers_repeated_readings() {
    let enhancer = Vislcg3InfiniteVerbEnhancer::default();
    let mut doc = Document::new("boahtit boahtit", "sme");
    doc.cg_tokens = vec![
        cg_token(0, 7, vec![reading(INF)]),
        cg_token(8, 15, vec![reading(INF)]),
    ];

    enhancer.process(&mut doc).expect("process");

    let first = concat!(
        "<span id=\"WERTi-span-boahtit-V-xsmey-Inf-1\" ",
        "class=\"wertiviewtoken  wertiviewInfiniteVerb\"lemma=\"boahtit\">"
    );
    let second = concat!(
        "<span id=\"WERTi-span-boahtit-V-xsmey-Inf-2\" ",
        "class=\"wertiviewtoken  wertiviewInfiniteVerb\"lemma=\"boahtit\">"
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

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.process-fn/test]
#[test]
fn process_selects_the_first_reading_matching_both_patterns() {
    let enhancer = Vislcg3InfiniteVerbEnhancer::default();
    let mut doc = Document::new("boahtit", "sme");
    doc.cg_tokens = vec![cg_token(
        0,
        7,
        vec![reading(FINITE), reading(INF), reading(PRF_PRC)],
    )];

    enhancer.process(&mut doc).expect("process");

    assert_eq!(doc.enhancements.len(), 1);
    assert_eq!(
        doc.enhancements[0].enhance_start,
        concat!(
            "<span id=\"WERTi-span-boahtit-V-xsmey-Inf-1\" ",
            "class=\"wertiviewtoken  wertiviewInfiniteVerb\"lemma=\"boahtit\">"
        )
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.process-fn/test]
#[test]
fn process_ignores_tokens_without_infinite_verb_readings() {
    let enhancer = Vislcg3InfiniteVerbEnhancer::default();
    let mut doc = Document::new("boahtá beana", "sme");
    doc.cg_tokens = vec![
        cg_token(0, 6, vec![reading(FINITE)]),
        cg_token(7, 12, vec![reading(NOUN)]),
        cg_token(13, 13, vec![]),
    ];

    enhancer.process(&mut doc).expect("process");

    assert!(doc.enhancements.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.process-fn/test]
#[test]
fn process_returns_early_when_the_cas_was_cancelled() {
    let enhancer = Vislcg3InfiniteVerbEnhancer::default();
    let mut doc = Document::new("boahtit", "sme");
    doc.cg_tokens = vec![cg_token(0, 7, vec![reading(INF)])];
    cas_utils::add_enh_id(&mut doc, -1);

    enhancer.process(&mut doc).expect("process");

    assert!(doc.enhancements.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.process-fn/test]
#[test]
fn process_selection_ignores_the_configured_tags() {
    let configured =
        Vislcg3InfiniteVerbEnhancer::new(Some("N Pl Nom, N Pl Gen")).expect("configured");
    let mut doc = Document::new("boahtit", "sme");
    doc.cg_tokens = vec![cg_token(0, 7, vec![reading(INF)])];

    configured.process(&mut doc).expect("process");

    // The infinite-verb reading is still selected and the plural-noun
    // reading is still rejected, whatever the parameter said.
    assert_eq!(doc.enhancements.len(), 1);
    assert_eq!(doc.enhancements[0].begin, 0);

    let mut nouns = Document::new("beanat", "sme");
    nouns.cg_tokens = vec![cg_token(0, 6, vec![reading(NOUN)])];
    configured.process(&mut nouns).expect("process");
    assert!(nouns.enhancements.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.remove-tags-fn/test]
#[test]
fn remove_tags_strips_literals_and_the_angle_tag() {
    let enhancer = Vislcg3InfiniteVerbEnhancer::default();

    assert_eq!(
        enhancer.remove_tags("boahtit+V+<sme>+Allegro+Inf"),
        "boahtit+V+Inf"
    );
    assert_eq!(
        enhancer.remove_tags("boahtit+V+Err/CmpSub+Err/MissingSpace+Inf"),
        "boahtit+V+Inf"
    );
    assert_eq!(
        enhancer.remove_tags("boahtit+V+Err/Hyph+Err/SpaceCmp+Err/Spellrelax+Inf"),
        "boahtit+V+Inf"
    );
    assert_eq!(enhancer.remove_tags("boahtit+V+Inf"), "boahtit+V+Inf");
    assert_eq!(enhancer.remove_tags(""), "");

    // The argument is borrowed, never mutated.
    let input = String::from("boahtit+V+Allegro+Inf");
    assert_eq!(enhancer.remove_tags(&input), "boahtit+V+Inf");
    assert_eq!(input, "boahtit+V+Allegro+Inf");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.remove-tags-fn/test]
#[test]
fn remove_tags_leaves_shadowed_err_orth_suffixes() {
    let enhancer = Vislcg3InfiniteVerbEnhancer::default();

    assert_eq!(
        enhancer.remove_tags("boahtit+V+Err/Orth-a-á+Inf"),
        "boahtit+V-a-á+Inf"
    );
    assert_eq!(
        enhancer.remove_tags("boahtit+V+Err/Orth-nom-gen+Inf"),
        "boahtit+V-nom-gen+Inf"
    );
    assert_eq!(
        enhancer.remove_tags("boahtit+V+Err/Orth-nom-acc+Inf"),
        "boahtit+V-nom-acc+Inf"
    );
    // The bare prefix is the only one of the four that leaves nothing.
    assert_eq!(
        enhancer.remove_tags("boahtit+V+Err/Orth+Inf"),
        "boahtit+V+Inf"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.remove-tags-fn/test]
#[test]
fn remove_tags_reuses_the_first_angle_match_literally() {
    let enhancer = Vislcg3InfiniteVerbEnhancer::default();

    // Every occurrence of the matched text goes ...
    assert_eq!(
        enhancer.remove_tags("boahtit+<sme>+V+<sme>+Inf"),
        "boahtit+V+Inf"
    );
    // ... but a differently spelled second tag survives.
    assert_eq!(
        enhancer.remove_tags("boahtit+<sme>+V+<hum_x>+Inf"),
        "boahtit+V+<hum_x>+Inf"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.write-morphological-forms-fn/test]
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

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.write-morphological-forms-fn/test]
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

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.write-morphological-forms-fn/test]
#[test]
fn write_morphological_forms_fails_without_a_plus() {
    let enhancer = Vislcg3InfiniteVerbEnhancer::default();

    let err = enhancer
        .write_morphological_forms("boahtit")
        .expect_err("no plus");
    assert_eq!(err.to_string(), "begin 0, end -1, length 7");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.write-lemma-and-analyses-fn/test]
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

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.write-lemma-and-analyses-fn/test]
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

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.write-lemma-and-analyses-fn/test]
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

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-distractors-fn/test]
#[test]
fn distractors_attach_forms_and_trailing_answer() {
    let enhancer = Vislcg3InfiniteVerbEnhancer::default();
    let outer = enhancer.outer_id();
    let mut map = span_map(outer, &[(0, 7)]);
    let mut doc = Document::new("boahtit", "sme");

    let output = concat!(
        "boahtit+V+Ind+Prs+Sg1\tboađán\n",
        "\n",
        "boahtit+V+Ind+Prs+Sg2\tboađát\n",
        "   \n",
        "boahtit+V+Ind+Prs+Sg3\tboahtá\n",
        "boahtit+V+Inf\tboahtit\n",
        "ñôŃßĘńŠē\n",
        "Word 0 7\n",
    );

    enhancer
        .generate_span_tag_with_distractors(&mut doc, output, &mut map)
        .expect("parsed");

    let expected = concat!(
        "<span id=\"WERTi-span-boahtit-V-Inf-1\" ",
        "class=\"wertiviewtoken  wertiviewInfiniteVerb\"lemma=\"boahtit\"",
        "distractors=\"boađán boađát boahtá boahtit\"answer=\"boahtit\">"
    );

    assert_eq!(doc.enhancements.len(), 1);
    assert!(doc.enhancements[0].relevant);
    assert_eq!(doc.enhancements[0].begin, 0);
    assert_eq!(doc.enhancements[0].end, 7);
    assert_eq!(doc.enhancements[0].enhance_start, expected);
    assert_eq!(doc.enhancements[0].enhance_end, "</span>");
    // The attributes are spliced into the map's own span tag.
    assert_eq!(map[&Word::new(outer, 0, 7)].get_span_tag_start(), expected);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-distractors-fn/test]
#[test]
fn distractors_dedup_and_drop_ungenerated_forms() {
    let enhancer = Vislcg3InfiniteVerbEnhancer::default();
    let outer = enhancer.outer_id();
    let mut map = span_map(outer, &[(0, 3)]);
    let mut doc = Document::new("boa", "sme");

    let output = concat!(
        "a+V+Ind+Prs+Sg1\tboađán\n",
        "a+V+Ind+Prs+Sg2\tboađán\n",
        "a+V+Ind+Prs+Sg3\tboahtá\n",
        "a+V+Ind+Prt+Sg1\tmuitalit-eallin\n",
        "a+V+Inf\ta+V+Inf+?\n",
        "ñôŃßĘńŠē\n",
        "Word 0 3\n",
    );

    enhancer
        .generate_span_tag_with_distractors(&mut doc, output, &mut map)
        .expect("parsed");

    assert_eq!(doc.enhancements.len(), 1);
    assert_eq!(
        doc.enhancements[0].enhance_start,
        concat!(
            "<span id=\"WERTi-span-boahtit-V-Inf-1\" ",
            "class=\"wertiviewtoken  wertiviewInfiniteVerb\"lemma=\"boahtit\"",
            "distractors=\"boađán boahtá\"answer=\"a+V+Inf+?\">"
        )
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-distractors-fn/test]
#[test]
fn distractors_drop_tokens_with_fewer_than_two_forms() {
    let enhancer = Vislcg3InfiniteVerbEnhancer::default();
    let outer = enhancer.outer_id();
    let mut map = span_map(outer, &[(0, 7)]);
    let mut doc = Document::new("boahtit", "sme");

    let output = concat!("boahtit+V+Inf\tboahtit\n", "ñôŃßĘńŠē\n", "Word 0 7\n");

    enhancer
        .generate_span_tag_with_distractors(&mut doc, output, &mut map)
        .expect("parsed");

    assert!(doc.enhancements.is_empty());
    assert_eq!(
        map[&Word::new(outer, 0, 7)].get_span_tag_start(),
        lemma_span(outer).get_span_tag_start()
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-distractors-fn/test]
#[test]
fn distractors_handle_consecutive_records() {
    let enhancer = Vislcg3InfiniteVerbEnhancer::default();
    let outer = enhancer.outer_id();
    let mut map = span_map(outer, &[(0, 1), (2, 3)]);
    let mut doc = Document::new("a b", "sme");

    let output = concat!(
        "a+V+Ind+Prs+Sg1\tform1\n",
        "a+V+Ind+Prs+Sg2\tform2\n",
        "a+V+Inf\tformA\n",
        "ñôŃßĘńŠē\n",
        "Word 0 1\n",
        "b+V+Ind+Prs+Sg1\tform3\n",
        "b+V+Ind+Prs+Sg2\tform4\n",
        "b+V+Inf\tformB\n",
        "ñôŃßĘńŠē\n",
        "Word 2 3\n",
    );

    enhancer
        .generate_span_tag_with_distractors(&mut doc, output, &mut map)
        .expect("parsed");

    assert_eq!(doc.enhancements.len(), 2);
    assert!(
        doc.enhancements[0]
            .enhance_start
            .contains("distractors=\"form1 form2 formA\"answer=\"formA\"")
    );
    assert_eq!(doc.enhancements[0].begin, 0);
    assert_eq!(doc.enhancements[0].end, 1);
    assert!(
        doc.enhancements[1]
            .enhance_start
            .contains("distractors=\"form3 form4 formB\"answer=\"formB\"")
    );
    assert_eq!(doc.enhancements[1].begin, 2);
    assert_eq!(doc.enhancements[1].end, 3);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-distractors-fn/test]
#[test]
fn distractors_fail_on_unmapped_or_malformed_words() {
    let enhancer = Vislcg3InfiniteVerbEnhancer::default();
    let outer = enhancer.outer_id();
    let record = concat!("a+V+Ind+Prs+Sg1\tform1\n", "a+V+Inf\tformA\n", "ñôŃßĘńŠē\n");

    let mut doc = Document::new("a", "sme");
    let mut map = span_map(outer, &[(0, 1)]);
    let err = enhancer
        .generate_span_tag_with_distractors(&mut doc, &format!("{record}Word 4 5\n"), &mut map)
        .expect_err("offsets not in the map");
    assert!(err.to_string().starts_with("no span tag for Word 4 5"));

    let mut doc = Document::new("a", "sme");
    let mut map = span_map(outer, &[(0, 1)]);
    assert!(
        enhancer
            .generate_span_tag_with_distractors(&mut doc, &format!("{record}Word x y\n"), &mut map)
            .is_err()
    );

    let mut doc = Document::new("a", "sme");
    let mut map = span_map(outer, &[(0, 1)]);
    let err = enhancer
        .generate_span_tag_with_distractors(&mut doc, &format!("{record}Word\n"), &mut map)
        .expect_err("no offsets at all");
    assert_eq!(err.to_string(), "Index 2 out of bounds for length 1");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-possible-forms-fn/test]
#[test]
fn possible_forms_enhance_on_a_single_form() {
    let enhancer = Vislcg3InfiniteVerbEnhancer::default();
    let outer = enhancer.outer_id();
    let mut map = span_map(outer, &[(0, 7)]);
    let mut doc = Document::new("boahtit", "sme");

    let output = concat!("boahtit+V+Inf\tboahtit\n", "ñôŃßĘńŠē\n", "Word 0 7\n");

    enhancer
        .generate_span_tag_with_possible_forms(&mut doc, output, &mut map)
        .expect("parsed");

    let expected = concat!(
        "<span id=\"WERTi-span-boahtit-V-Inf-1\" ",
        "class=\"wertiviewtoken  wertiviewInfiniteVerb\"lemma=\"boahtit\"",
        "possibleforms=\"boahtit\">"
    );

    assert_eq!(doc.enhancements.len(), 1);
    assert!(doc.enhancements[0].relevant);
    assert_eq!(doc.enhancements[0].begin, 0);
    assert_eq!(doc.enhancements[0].end, 7);
    assert_eq!(doc.enhancements[0].enhance_start, expected);
    assert_eq!(doc.enhancements[0].enhance_end, "</span>");
    // No answer attribute is added on this path.
    assert!(!doc.enhancements[0].enhance_start.contains("answer="));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-possible-forms-fn/test]
#[test]
fn possible_forms_dedup_and_skip_empty_records() {
    let enhancer = Vislcg3InfiniteVerbEnhancer::default();
    let outer = enhancer.outer_id();
    let mut map = span_map(outer, &[(0, 1), (2, 3)]);
    let mut doc = Document::new("a b", "sme");

    let output = concat!(
        "a+V+Inf\tboahtit\n",
        "a+V+PrfPrc\tboahtit\n",
        "a+V+VGen\tboahtin-dihte\n",
        "ñôŃßĘńŠē\n",
        "Word 0 1\n",
        "b+V+Inf\tb+V+Inf+?\n",
        "ñôŃßĘńŠē\n",
        "Word 2 3\n",
    );

    enhancer
        .generate_span_tag_with_possible_forms(&mut doc, output, &mut map)
        .expect("parsed");

    // The duplicate and the hyphenated form are dropped, and the second
    // record generated nothing so its Word line is ignored.
    assert_eq!(doc.enhancements.len(), 1);
    assert!(
        doc.enhancements[0]
            .enhance_start
            .ends_with("possibleforms=\"boahtit\">")
    );
    assert_eq!(doc.enhancements[0].begin, 0);
    assert_eq!(
        map[&Word::new(outer, 2, 3)].get_span_tag_start(),
        lemma_span(outer).get_span_tag_start()
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-possible-forms-fn/test]
#[test]
fn possible_forms_fail_on_an_unmapped_word_line() {
    let enhancer = Vislcg3InfiniteVerbEnhancer::default();
    let outer = enhancer.outer_id();
    let mut map = span_map(outer, &[(0, 1)]);
    let mut doc = Document::new("a", "sme");

    let err = enhancer
        .generate_span_tag_with_possible_forms(
            &mut doc,
            "a+V+Inf\tboahtit\nñôŃßĘńŠē\nWord 4 5\n",
            &mut map,
        )
        .expect_err("offsets not in the map");

    assert!(err.to_string().starts_with("no span tag for Word 4 5"));
    assert!(doc.enhancements.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.mutable-int.increment-fn/test]
#[test]
fn mutable_int_increments_in_place_from_one() {
    let mut counter = MutableInt::default();
    assert_eq!(counter.value, 1);

    counter.increment();
    assert_eq!(counter.value, 2);
    counter.increment();
    counter.increment();
    assert_eq!(counter.value, 4);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.mutable-int.get-fn/test]
#[test]
fn mutable_int_get_reads_without_changing_the_counter() {
    let mut counter = MutableInt::default();
    assert_eq!(counter.get(), 1);
    assert_eq!(counter.get(), 1);

    counter.increment();
    assert_eq!(counter.get(), 2);
    assert_eq!(counter.value, 2);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.word-fn/test]
#[test]
fn word_stores_the_offsets_verbatim() {
    let word = Word::new(9, 12, 4);
    assert_eq!(word.begin, 12);
    assert_eq!(word.end, 4);
    assert_eq!(word.outer, 9);

    // The scratch constructor used by the generator-output readers.
    let scratch = Word::empty(9);
    assert_eq!(scratch.begin, 0);
    assert_eq!(scratch.end, 0);
    assert_eq!(scratch.outer, 9);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.get-begin-fn/test]
#[test]
fn word_get_begin_returns_the_stored_begin_offset() {
    let word = Word::new(0, 12, 4);
    assert_eq!(word.get_begin(), 12);
    assert_eq!(word.get_begin(), 12);
    assert_eq!(Word::empty(0).get_begin(), 0);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.set-begin-fn/test]
#[test]
fn word_set_begin_overwrites_the_begin_offset() {
    let mut word = Word::new(0, 12, 4);
    word.set_begin(3);

    assert_eq!(word.get_begin(), 3);
    assert_eq!(word.get_end(), 4);

    // Mutating a key already in a map strands its entry.
    let mut map: HashMap<Word, SpanTag> = HashMap::new();
    map.insert(Word::new(0, 12, 4), SpanTag::new(0, "<span>".to_string()));
    assert!(map.get(&word).is_none());
    assert!(map.get(&Word::new(0, 12, 4)).is_some());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.get-end-fn/test]
#[test]
fn word_get_end_returns_the_stored_end_offset() {
    let word = Word::new(0, 12, 4);
    assert_eq!(word.get_end(), 4);
    assert_eq!(word.get_end(), 4);
    assert_eq!(Word::empty(0).get_end(), 0);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.set-end-fn/test]
#[test]
fn word_set_end_overwrites_the_end_offset() {
    let mut word = Word::new(0, 12, 4);
    word.set_end(30);

    assert_eq!(word.get_end(), 30);
    assert_eq!(word.get_begin(), 12);

    let mut map: HashMap<Word, SpanTag> = HashMap::new();
    map.insert(Word::new(0, 12, 4), SpanTag::new(0, "<span>".to_string()));
    assert!(map.get(&word).is_none());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.to-string-fn/test]
#[test]
fn word_renders_the_generator_input_record_line() {
    assert_eq!(Word::new(0, 3, 7).to_string(), "Word 3 7\n");
    assert_eq!(Word::empty(0).to_string(), "Word 0 0\n");

    // The rendering is exactly what the readers re-parse.
    let rendered = Word::new(0, 12, 19).to_string();
    let parts = split_ws(java_trim(&rendered));
    assert_eq!(parts, vec!["Word", "12", "19"]);
    assert!(rendered.starts_with("Word"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.hash-code-fn/test]
#[test]
fn word_hash_code_folds_outer_begin_and_end() {
    // 29791 + 961 * outer + 31 * begin + end
    assert_eq!(Word::new(0, 0, 0).hash_code(), 29791);
    assert_eq!(Word::new(0, 3, 7).hash_code(), 29891);
    assert_eq!(Word::new(1, 0, 0).hash_code(), 30752);

    // 32-bit signed overflow wraps rather than saturating.
    assert_eq!(Word::new(0, 100_000_000, 7).hash_code(), -1_194_937_498);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.equals-fn/test]
#[test]
fn word_equality_covers_offsets_and_owning_enhancer() {
    let word = Word::new(1, 3, 7);

    assert!(word.equals(&word));
    assert!(word.equals(&Word::new(1, 3, 7)));
    assert!(!word.equals(&Word::new(1, 4, 7)));
    assert!(!word.equals(&Word::new(1, 3, 8)));
    assert!(!word.equals(&Word::new(2, 3, 7)));

    // This is the contract the span map relies on: offsets re-parsed from
    // a record line find the entry stored during the token loop.
    let mut map: HashMap<Word, SpanTag> = HashMap::new();
    map.insert(Word::new(1, 3, 7), SpanTag::new(1, "<span>".to_string()));
    assert!(map.contains_key(&Word::new(1, 3, 7)));
    assert!(!map.contains_key(&Word::new(2, 3, 7)));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.get-outer-type-fn/test]
#[test]
fn word_get_outer_type_drives_equality() {
    assert_eq!(Word::new(42, 1, 2).get_outer_type(), 42);
    assert_eq!(Word::empty(7).get_outer_type(), 7);

    let mine = Word::new(1, 1, 2);
    let theirs = Word::new(2, 1, 2);
    assert_ne!(mine.get_outer_type(), theirs.get_outer_type());
    assert!(!mine.equals(&theirs));
    assert_ne!(mine.hash_code(), theirs.hash_code());

    // The enhancer instance the token loop runs under is what a word
    // built by that loop carries.
    let enhancer = Vislcg3InfiniteVerbEnhancer::default();
    assert_eq!(
        Word::new(enhancer.outer_id(), 0, 1).get_outer_type(),
        enhancer.outer_id()
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.span-tag-fn/test]
#[test]
fn span_tag_stores_opening_and_defaults_closing() {
    let tag = SpanTag::new(5, "<span id=\"x\">".to_string());

    assert_eq!(tag.span_tag_start, "<span id=\"x\">");
    assert_eq!(tag.span_tag_end, "</span>");
    assert_eq!(tag.outer, 5);

    // The opening tag is stored without validation.
    let odd = SpanTag::new(5, "not a tag at all".to_string());
    assert_eq!(odd.span_tag_start, "not a tag at all");
    assert_eq!(odd.span_tag_end, "</span>");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.get-span-tag-start-fn/test]
#[test]
fn span_tag_start_reflects_every_added_attribute() {
    let mut tag = SpanTag::new(0, SPAN_START.to_string());
    assert_eq!(tag.get_span_tag_start(), SPAN_START);

    tag.add_attribute("lemma", "boahtit");
    assert_eq!(
        tag.get_span_tag_start(),
        concat!(
            "<span id=\"WERTi-span-boahtit-V-Inf-1\" ",
            "class=\"wertiviewtoken  wertiviewInfiniteVerb\"lemma=\"boahtit\">"
        )
    );
    assert_eq!(tag.get_span_tag_start(), tag.span_tag_start);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.set-span-tag-start-fn/test]
#[test]
fn span_tag_set_start_discards_added_attributes() {
    let mut tag = SpanTag::new(0, SPAN_START.to_string());
    tag.add_attribute("lemma", "boahtit");
    tag.set_span_tag_start("<span>".to_string());

    assert_eq!(tag.get_span_tag_start(), "<span>");
    assert_eq!(tag.get_span_tag_end(), "</span>");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.add-attribute-fn/test]
#[test]
fn span_tag_add_attribute_splices_without_a_space() {
    let mut tag = SpanTag::new(0, SPAN_START.to_string());

    tag.add_attribute("lemma", "boahtit");
    assert_eq!(
        tag.get_span_tag_start(),
        concat!(
            "<span id=\"WERTi-span-boahtit-V-Inf-1\" ",
            "class=\"wertiviewtoken  wertiviewInfiniteVerb\"lemma=\"boahtit\">"
        )
    );

    tag.add_attribute("answer", "boahtit");
    assert!(tag.get_span_tag_start().ends_with(
        "class=\"wertiviewtoken  wertiviewInfiniteVerb\"lemma=\"boahtit\"answer=\"boahtit\">"
    ));

    // The value is neither HTML-escaped nor quote-escaped.
    let mut raw = SpanTag::new(0, "<span>".to_string());
    raw.add_attribute("distractors", "a & b \"c\"");
    assert_eq!(
        raw.get_span_tag_start(),
        "<spandistractors=\"a & b \"c\"\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.add-attribute-fn/test]
#[test]
fn span_tag_add_attribute_rewrites_every_closing_bracket() {
    let mut tag = SpanTag::new(0, "<span>".to_string());

    tag.add_attribute("a", "x>y");
    assert_eq!(tag.get_span_tag_start(), "<spana=\"x>y\">");

    // The value's own bracket becomes a second splice point, corrupting
    // the tag on the next call.
    tag.add_attribute("b", "z");
    assert_eq!(tag.get_span_tag_start(), "<spana=\"xb=\"z\">y\"b=\"z\">");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.get-span-tag-end-fn/test]
#[test]
fn span_tag_get_end_returns_the_closing_tag() {
    let mut tag = SpanTag::new(0, SPAN_START.to_string());
    assert_eq!(tag.get_span_tag_end(), "</span>");

    // Adding attributes only touches the opening tag.
    tag.add_attribute("lemma", "boahtit");
    assert_eq!(tag.get_span_tag_end(), "</span>");

    tag.set_span_tag_end("</b>".to_string());
    assert_eq!(tag.get_span_tag_end(), "</b>");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.set-span-tag-end-fn/test]
#[test]
fn span_tag_set_end_replaces_the_default() {
    let mut tag = SpanTag::new(0, SPAN_START.to_string());
    tag.set_span_tag_end("</div>".to_string());

    assert_eq!(tag.span_tag_end, "</div>");
    assert_eq!(tag.get_span_tag_start(), SPAN_START);

    tag.set_span_tag_end(String::new());
    assert_eq!(tag.get_span_tag_end(), "");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.hash-code-fn/test]
#[test]
fn span_tag_hash_folds_outer_end_and_start() {
    // 29791 + 961 * outer + 31 * hash(end) + hash(start), where hash("")
    // is 0 and hash("A") is 65.
    let mut tag = SpanTag::new(0, "A".to_string());
    tag.set_span_tag_end(String::new());
    assert_eq!(tag.hash_code(), 29856);

    let mut other = SpanTag::new(2, "A".to_string());
    other.set_span_tag_end(String::new());
    assert_eq!(other.hash_code(), 31778);

    // 32-bit signed overflow wraps rather than saturating.
    let mut wrapped = SpanTag::new(4_000_000, "A".to_string());
    wrapped.set_span_tag_end(String::new());
    assert_eq!(wrapped.hash_code(), -450_937_440);

    // Equal tags hash equally; an added attribute changes the hash.
    let mut before = SpanTag::new(0, SPAN_START.to_string());
    let after_same = SpanTag::new(0, SPAN_START.to_string());
    assert_eq!(before.hash_code(), after_same.hash_code());
    before.add_attribute("lemma", "boahtit");
    assert_ne!(before.hash_code(), after_same.hash_code());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.equals-fn/test]
#[test]
fn span_tag_equality_covers_tags_and_owning_enhancer() {
    let tag = SpanTag::new(1, SPAN_START.to_string());

    assert!(tag.equals(&tag));
    assert!(tag.equals(&SpanTag::new(1, SPAN_START.to_string())));
    assert!(!tag.equals(&SpanTag::new(2, SPAN_START.to_string())));
    assert!(!tag.equals(&SpanTag::new(1, "<span>".to_string())));

    let mut different_end = SpanTag::new(1, SPAN_START.to_string());
    different_end.set_span_tag_end("</b>".to_string());
    assert!(!tag.equals(&different_end));

    let mut with_attribute = SpanTag::new(1, SPAN_START.to_string());
    with_attribute.add_attribute("lemma", "boahtit");
    assert!(!tag.equals(&with_attribute));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.get-outer-type-fn/test]
#[test]
fn span_tag_get_outer_type_drives_equality() {
    assert_eq!(SpanTag::new(42, "<span>".to_string()).get_outer_type(), 42);

    let mine = SpanTag::new(1, "<span>".to_string());
    let theirs = SpanTag::new(2, "<span>".to_string());
    assert_ne!(mine.get_outer_type(), theirs.get_outer_type());
    assert!(!mine.equals(&theirs));
    assert_ne!(mine.hash_code(), theirs.hash_code());

    let enhancer = Vislcg3InfiniteVerbEnhancer::default();
    assert_eq!(
        SpanTag::new(enhancer.outer_id(), "<span>".to_string()).get_outer_type(),
        enhancer.outer_id()
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.to-string-fn/test]
#[test]
fn span_tag_renders_both_stored_tags_unescaped() {
    let mut tag = SpanTag::new(0, "<span id=\"x\">".to_string());
    assert_eq!(
        tag.to_string(),
        "SpanTag [spanTagStart=<span id=\"x\">, spanTagEnd=</span>]"
    );
    assert!(!tag.to_string().ends_with('\n'));

    tag.add_attribute("lemma", "boahtit");
    tag.set_span_tag_end("</b>".to_string());
    assert_eq!(
        tag.to_string(),
        "SpanTag [spanTagStart=<span id=\"x\"lemma=\"boahtit\">, spanTagEnd=</b>]"
    );
}
