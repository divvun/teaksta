//! The enhancement pass every "big" CG3 topic enhancer shares is exercised
//! once here. Each test carries the `/test` facet of all five topics' rule
//! for the function it drives, because all five run this implementation.

use super::*;
use crate::types::PIPELINE_LANGUAGE;

/// Every shared log line off, as the two verb topics ask for them.
const QUIET: Trace = Trace {
    span_tag: false,
    enhancement: false,
    possible_forms: false,
};

/// Every shared log line on, as the singular-noun topic asks for them.
const LOUD: Trace = Trace {
    span_tag: true,
    enhancement: true,
    possible_forms: true,
};

/// The map `process` hands to a generator-output reader: offsets to the span
/// tag built for them, each on the one class this pass does not vary.
fn span_map(spans: &[(usize, usize, &str)]) -> HashMap<Word, SpanTag> {
    spans
        .iter()
        .map(|(begin, end, id)| (Word::new(*begin, *end), SpanTag::new(*id, &["c"])))
        .collect()
}

/// The markup `span_map` renders for one id before any attribute is added.
fn plain(id: &str) -> String {
    format!("<span id=\"{id}\" class=\"c\">")
}

/// A closed block of generated forms, marker line included, with no `Word`
/// record of its own.
fn one_block() -> String {
    format!("a+V+Ind+Prs+Sg1\tform1\na+V+Inf\tformA\n{}\n", MARKER)
}

/// A topic with nothing of its own: the shared pass is what is under test,
/// and the two generator inputs are supplied per case.
const PLAIN: TopicSpec = TopicSpec {
    label: "Plain",
    span_class: "teaksta-Plain",
    pos: r"N\+",
    selector: r"(Sg|Pl)\+Nom",
    hints: None,
    strip_lang_tag: false,
    log_chosen_reading: false,
    unchecked: Unchecked::Propagate,
    trace: QUIET,
};

/// The reading `select` would have accepted for the token below.
fn accepted() -> Selection {
    Selection {
        valid: true,
        reading: "beana+N+Sg+Nom".to_string(),
        lemma: "beana".to_string(),
        hint_tag: String::new(),
    }
}

/// One `Run` over [`PLAIN`] for the activity the flags name.
fn pass<'a>(
    mc: bool,
    cloze: bool,
    forms: &'a dyn Fn(&str) -> Result<String>,
    analyses: &'a dyn Fn(&str) -> Result<String>,
) -> Run<'a> {
    Run {
        spec: &PLAIN,
        matcher: Matcher::new(&PLAIN).expect("the topic patterns compile"),
        mc,
        cloze,
        forms,
        analyses,
    }
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.remove-tags-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.remove-tags-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.remove-tags-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.remove-tags-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.remove-tags-fn+2/test]
#[test]
fn literal_tags_are_deleted_wherever_they_occur() {
    assert_eq!(
        remove_tags("beana+N+Allegro+Err/Spellrelax+Sg+Nom"),
        "beana+N+Sg+Nom"
    );
    assert_eq!(
        remove_tags("boahtit+V+Err/CmpSub+Err/MissingSpace+Inf"),
        "boahtit+V+Inf"
    );
    assert_eq!(
        remove_tags("boahtit+V+Err/Hyph+Err/SpaceCmp+Err/Spellrelax+Inf"),
        "boahtit+V+Inf"
    );
    assert_eq!(remove_tags("beana+N+<sme>+Sg+Nom"), "beana+N+Sg+Nom");

    // Nothing to strip: the input comes back verbatim, untrimmed.
    assert_eq!(remove_tags("  boahtit+V  "), "  boahtit+V  ");
    assert_eq!(remove_tags(""), "");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.remove-tags-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.remove-tags-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.remove-tags-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.remove-tags-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.remove-tags-fn+2/test]
#[test]
fn err_orth_variants_go_whole_leaving_no_suffix() {
    assert_eq!(remove_tags("beana+N+Err/Orth-a-á+Sg+Nom"), "beana+N+Sg+Nom");
    assert_eq!(
        remove_tags("beana+N+Err/Orth-nom-gen+Sg+Gen"),
        "beana+N+Sg+Gen"
    );
    assert_eq!(
        remove_tags("beana+N+Err/Orth-nom-acc+Sg+Acc"),
        "beana+N+Sg+Acc"
    );
    assert_eq!(remove_tags("beana+N+Err/Orth+Sg+Nom"), "beana+N+Sg+Nom");

    // The bare prefix still goes on its own when a longer sibling shares
    // the line.
    assert_eq!(
        remove_tags("beana+N+Err/Orth+Err/Orth-a-á+Sg+Nom"),
        "beana+N+Sg+Nom"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.remove-tags-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.remove-tags-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.remove-tags-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.remove-tags-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.remove-tags-fn+2/test]
#[test]
fn every_angle_bracket_tag_is_removed() {
    assert_eq!(remove_tags("beana+N+<sme>+Sg+Nom+<sme>"), "beana+N+Sg+Nom");
    assert_eq!(
        remove_tags("boahtit+<sme>+V+Ind\nmannat+<sme>+V+Ind\n"),
        "boahtit+V+Ind\nmannat+V+Ind\n"
    );
    // A differently spelled second tag goes with the first.
    assert_eq!(
        remove_tags("beana+N+<sme>+Sg+Nom+<compl_subj>"),
        "beana+N+Sg+Nom"
    );
    assert_eq!(remove_tags("a+<a_b>c"), "ac");

    // The argument is borrowed, never mutated.
    let input = String::from("boahtit+V+Allegro+Inf");
    assert_eq!(remove_tags(&input), "boahtit+V+Inf");
    assert_eq!(input, "boahtit+V+Allegro+Inf");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-distractors-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-distractors-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-distractors-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-distractors-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-distractors-fn+3/test]
#[test]
fn distinct_forms_become_distractors_last_is_answer() {
    let mut map = span_map(&[(0, 5, "s1")]);
    let mut doc = Document::new("beana", PIPELINE_LANGUAGE);
    let output = concat!(
        "beana+N+Sg+Acc\tbeana\n",
        "beana+N+Sg+Ill\tbeatnagii\n",
        "beana+N+Sg+Loc\tbeatnagis\n",
        "beana+N+Sg+Com\tbeatnagiin\n",
        "beana+N+Sg+Ess\t+?\n",
        "\n",
        "beana+N+Sg+Nom\tbeana\n",
        "ñôŃßĘńŠē\n",
        "Word 0 5\n",
    );

    attach_distractors(&mut doc, LOUD, output, &mut map).unwrap();

    let expected = "<span id=\"s1\" class=\"c\" \
                    distractors=\"beana beatnagii beatnagis beatnagiin\" answer=\"beana\">";
    assert_eq!(doc.enhancements.len(), 1);
    assert!(doc.enhancements[0].relevant);
    assert_eq!((doc.enhancements[0].begin, doc.enhancements[0].end), (0, 5));
    assert_eq!(doc.enhancements[0].enhance_end, "</span>");
    assert_eq!(doc.enhancements[0].enhance_start, expected);
    // The attributes are spliced into the map's own span tag, not a copy.
    assert_eq!(map[&Word::new(0, 5)].start_tag(), expected);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-distractors-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-distractors-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-distractors-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-distractors-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-distractors-fn+3/test]
#[test]
fn a_single_surviving_form_leaves_a_token_alone() {
    let mut map = span_map(&[(0, 5, "s1")]);
    let mut doc = Document::new("beana", PIPELINE_LANGUAGE);
    let output = concat!(
        "beana+N+Sg+Nom\tbeana\n",
        "beana+N+Sg+Ill\t+?\n",
        "ñôŃßĘńŠē\n",
        "Word 0 5\n",
    );

    attach_distractors(&mut doc, QUIET, output, &mut map).unwrap();

    assert!(doc.enhancements.is_empty());
    assert_eq!(map[&Word::new(0, 5)].start_tag(), plain("s1"));

    // A Word line seen before any marker is ignored for the same reason,
    // even when nothing in the map could have matched it.
    let mut empty: HashMap<Word, SpanTag> = HashMap::new();
    let mut doc = Document::new("beana", PIPELINE_LANGUAGE);

    attach_distractors(&mut doc, QUIET, "Word 0 5\n", &mut empty).unwrap();

    assert!(doc.enhancements.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-distractors-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-distractors-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-distractors-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-distractors-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-distractors-fn+3/test]
#[test]
fn a_block_is_consumed_by_one_word_record() {
    let mut map = span_map(&[(0, 5, "s1"), (6, 11, "s2")]);
    let mut doc = Document::new("beana beana", PIPELINE_LANGUAGE);
    let output = concat!(
        "beana+N+Sg+Acc\tbeana\n",
        "beana+N+Sg+Ill\tbeatnagii\n",
        "ñôŃßĘńŠē\n",
        "Word 0 5\n",
        "Word 6 11\n",
    );

    attach_distractors(&mut doc, QUIET, output, &mut map).unwrap();

    // The second record has no block of its own, so it is not enhanced
    // with the first token's distractors.
    assert_eq!(doc.enhancements.len(), 1);
    assert_eq!(
        doc.enhancements[0].enhance_start,
        "<span id=\"s1\" class=\"c\" distractors=\"beana beatnagii\" answer=\"beatnagii\">"
    );
    assert_eq!((doc.enhancements[0].begin, doc.enhancements[0].end), (0, 5));
    assert_eq!(map[&Word::new(6, 11)].start_tag(), plain("s2"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-distractors-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-distractors-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-distractors-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-distractors-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-distractors-fn+3/test]
#[test]
fn a_later_marker_replaces_the_earlier_block() {
    let mut map = span_map(&[(0, 1, "s1"), (2, 3, "s2")]);
    let mut doc = Document::new("a b", PIPELINE_LANGUAGE);
    let output = concat!(
        "a+V+Ind+Prs+Sg1\tform1\n",
        "   \n",
        "a+V+Ind+Prs+Sg2\tform1\n",
        "a+V+Inf\tmuitalit-eallin\n",
        "a+V+Inf\tformA\n",
        "ñôŃßĘńŠē\n",
        "Word 0 1\n",
        "b+V+Ind+Prs+Sg1\tform3\n",
        "b+V+Ind+Prs+Sg2\tform4\n",
        "ñôŃßĘńŠē\n",
        "Word 2 3\n",
    );

    attach_distractors(&mut doc, QUIET, output, &mut map).unwrap();

    // The duplicate and the hyphenated form are dropped, and the answer is
    // the last whitespace-separated field of the block whether or not the
    // generator managed to produce it.
    assert_eq!(doc.enhancements.len(), 2);
    assert!(
        doc.enhancements[0]
            .enhance_start
            .contains("distractors=\"form1 formA\" answer=\"formA\"")
    );
    assert_eq!((doc.enhancements[0].begin, doc.enhancements[0].end), (0, 1));
    assert!(
        doc.enhancements[1]
            .enhance_start
            .contains("distractors=\"form3 form4\" answer=\"form4\"")
    );
    assert_eq!((doc.enhancements[1].begin, doc.enhancements[1].end), (2, 3));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-distractors-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-distractors-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-distractors-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-distractors-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-distractors-fn+3/test]
#[test]
fn malformed_word_records_abort_the_run() {
    let block = one_block();
    let cases = [
        ("Word 4 5\n", "no span tag for Word 4 5"),
        ("Word zero 1\n", "For input string: \"zero\""),
        ("Word\n", "Index 2 out of bounds for length 1"),
        ("Word 0\n", "Index 2 out of bounds for length 2"),
    ];

    for (record, message) in cases {
        let mut map = span_map(&[(0, 1, "s1")]);
        let mut doc = Document::new("a", PIPELINE_LANGUAGE);

        let err = attach_distractors(&mut doc, QUIET, &format!("{block}{record}"), &mut map)
            .expect_err(record);

        assert!(err.to_string().starts_with(message), "{err}");
        assert!(doc.enhancements.is_empty());
    }
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-possible-forms-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-possible-forms-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-possible-forms-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-possible-forms-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-possible-forms-fn+3/test]
#[test]
fn a_single_form_enhances_a_cloze_token() {
    let mut map = span_map(&[(0, 5, "s1")]);
    let mut doc = Document::new("beana", PIPELINE_LANGUAGE);
    let output = concat!(
        "beana+N+Sg+Nom\tbeana\n",
        "beana+N+Sg+Ill\t+?\n",
        "\n",
        "ñôŃßĘńŠē\n",
        "Word 0 5\n",
    );

    attach_possible_forms(&mut doc, LOUD, output, &mut map).unwrap();

    assert_eq!(doc.enhancements.len(), 1);
    assert!(doc.enhancements[0].relevant);
    assert_eq!((doc.enhancements[0].begin, doc.enhancements[0].end), (0, 5));
    assert_eq!(doc.enhancements[0].enhance_end, "</span>");
    assert_eq!(
        doc.enhancements[0].enhance_start,
        "<span id=\"s1\" class=\"c\" possibleforms=\"beana\">"
    );
    // No answer attribute is added on this path, and one form is enough.
    assert!(!doc.enhancements[0].enhance_start.contains("answer="));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-possible-forms-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-possible-forms-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-possible-forms-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-possible-forms-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-possible-forms-fn+3/test]
#[test]
fn duplicate_forms_collapse_and_empty_blocks_skip() {
    let mut map = span_map(&[(0, 1, "s1"), (2, 3, "s2")]);
    let mut doc = Document::new("a b", PIPELINE_LANGUAGE);
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

    attach_possible_forms(&mut doc, QUIET, output, &mut map).unwrap();

    // The duplicate and the hyphenated form are dropped, and the second
    // record generated nothing at all so its Word line is ignored.
    assert_eq!(doc.enhancements.len(), 1);
    assert_eq!(
        doc.enhancements[0].enhance_start,
        "<span id=\"s1\" class=\"c\" possibleforms=\"boahtit\">"
    );
    assert_eq!(map[&Word::new(2, 3)].start_tag(), plain("s2"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-possible-forms-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-possible-forms-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-possible-forms-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-possible-forms-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-possible-forms-fn+3/test]
#[test]
fn a_cloze_block_is_consumed_by_one_record() {
    let mut map = span_map(&[(0, 1, "s1"), (2, 3, "s2")]);
    let mut doc = Document::new("a b", PIPELINE_LANGUAGE);
    let output = concat!(
        "a+V+Inf\tboahtit\n",
        "ñôŃßĘńŠē\n",
        "Word 0 1\n",
        "Word 2 3\n",
    );

    attach_possible_forms(&mut doc, QUIET, output, &mut map).unwrap();

    // The second record has no block of its own, so it does not inherit
    // the first token's forms.
    assert_eq!(doc.enhancements.len(), 1);
    assert_eq!(
        doc.enhancements[0].enhance_start,
        "<span id=\"s1\" class=\"c\" possibleforms=\"boahtit\">"
    );
    assert_eq!(map[&Word::new(2, 3)].start_tag(), plain("s2"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-possible-forms-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-possible-forms-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-possible-forms-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-possible-forms-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-possible-forms-fn+3/test]
#[test]
fn the_cloze_reader_shares_the_record_failures() {
    let block = "a+V+Inf\tboahtit\nñôŃßĘńŠē\n";
    let cases = [
        ("Word 4 5\n", "no span tag for Word 4 5"),
        ("Word x 5\n", "For input string: \"x\""),
        ("Word\n", "Index 2 out of bounds for length 1"),
    ];

    for (record, message) in cases {
        let mut map = span_map(&[(0, 1, "s1")]);
        let mut doc = Document::new("a", PIPELINE_LANGUAGE);

        let err = attach_possible_forms(&mut doc, QUIET, &format!("{block}{record}"), &mut map)
            .expect_err(record);

        assert!(err.to_string().starts_with(message), "{err}");
        assert!(doc.enhancements.is_empty());
    }
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.process-fn+4/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.process-fn+4/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.process-fn+4/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.process-fn+4/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.process-fn+4/test]
#[test]
fn an_unusable_reading_is_dropped_on_its_own() {
    let refuse = |reading: &str| -> Result<String> {
        bail!("begin 0, end -1, length {}", reading.chars().count())
    };
    let block = |_: &str| -> Result<String> { Ok("beana+N+Sg+Acc\n".to_string()) };
    let token = CgToken {
        begin: 0,
        end: 5,
        readings: Vec::new(),
    };

    for (mc, cloze) in [(true, false), (false, true)] {
        let mut doc = Document::new("beana", PIPELINE_LANGUAGE);
        let mut scan = Scan::default();

        pass(mc, cloze, &refuse, &refuse).enhance_token(&mut doc, &token, &accepted(), &mut scan);

        // The reading contributes no generator record and no enhancement,
        // and the pass carries on rather than reporting a failure.
        assert!(scan.generator_input.is_empty());
        assert!(scan.generator_input_cloze.is_empty());
        assert!(doc.enhancements.is_empty());
        // The span is still registered; nothing looks it up without a
        // record naming its offsets.
        assert!(scan.word_to_span_map.contains_key(&Word::new(0, 5)));
    }

    let mut doc = Document::new("beana", PIPELINE_LANGUAGE);
    let mut scan = Scan::default();

    pass(true, false, &block, &refuse).enhance_token(&mut doc, &token, &accepted(), &mut scan);

    assert_eq!(
        scan.generator_input,
        format!("beana+N+Sg+Acc\n{}\nWord 0 5\n", MARKER)
    );
}
