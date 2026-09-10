//! The enhancement pass every "big" CG3 topic enhancer shares is exercised
//! once here. Each test carries the `/test` facet of all five topics' rule
//! for the function it drives, because all five run this implementation.

use super::*;

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
/// tag built for them.
fn span_map(outer: usize, spans: &[(usize, usize, &str)]) -> HashMap<Word, SpanTag> {
    spans
        .iter()
        .map(|(begin, end, start)| {
            (
                Word::new(outer, *begin, *end),
                SpanTag::new(outer, start.to_string()),
            )
        })
        .collect()
}

/// A closed block of generated forms, marker line included, with no `Word`
/// record of its own.
fn one_block() -> String {
    format!("a+V+Ind+Prs+Sg1\tform1\na+V+Inf\tformA\n{}\n", MARKER)
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.remove-tags-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.remove-tags-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.remove-tags-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.remove-tags-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.remove-tags-fn/test]
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

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.remove-tags-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.remove-tags-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.remove-tags-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.remove-tags-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.remove-tags-fn/test]
#[test]
fn err_orth_prefix_leaves_hyphenated_suffix() {
    assert_eq!(
        remove_tags("beana+N+Err/Orth-a-á+Sg+Nom"),
        "beana+N-a-á+Sg+Nom"
    );
    assert_eq!(
        remove_tags("beana+N+Err/Orth-nom-gen+Sg+Gen"),
        "beana+N-nom-gen+Sg+Gen"
    );
    assert_eq!(
        remove_tags("beana+N+Err/Orth-nom-acc+Sg+Acc"),
        "beana+N-nom-acc+Sg+Acc"
    );

    // The bare prefix is the only one of the four that leaves nothing.
    assert_eq!(remove_tags("beana+N+Err/Orth+Sg+Nom"), "beana+N+Sg+Nom");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.remove-tags-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.remove-tags-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.remove-tags-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.remove-tags-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.remove-tags-fn/test]
#[test]
fn only_the_first_angle_bracket_tag_matches() {
    // Every occurrence of the matched text goes ...
    assert_eq!(remove_tags("beana+N+<sme>+Sg+Nom+<sme>"), "beana+N+Sg+Nom");
    assert_eq!(
        remove_tags("boahtit+<sme>+V+Ind\nmannat+<sme>+V+Ind\n"),
        "boahtit+V+Ind\nmannat+V+Ind\n"
    );
    // ... but a differently spelled second tag survives.
    assert_eq!(
        remove_tags("beana+N+<sme>+Sg+Nom+<compl_subj>"),
        "beana+N+Sg+Nom+<compl_subj>"
    );
    assert_eq!(remove_tags("a+<a_b>c"), "ac");

    // The argument is borrowed, never mutated.
    let input = String::from("boahtit+V+Allegro+Inf");
    assert_eq!(remove_tags(&input), "boahtit+V+Inf");
    assert_eq!(input, "boahtit+V+Allegro+Inf");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-distractors-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-distractors-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-distractors-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-distractors-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-distractors-fn/test]
#[test]
fn distinct_forms_become_distractors_last_is_answer() {
    let mut map = span_map(0, &[(0, 5, "<span id=\"s1\" class=\"c\">")]);
    let mut doc = Document::new("beana", "sme");
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

    attach_distractors(&mut doc, 0, LOUD, output, &mut map).unwrap();

    let expected = "<span id=\"s1\" class=\"c\"\
                    distractors=\"beana beatnagii beatnagis beatnagiin\"answer=\"beana\">";
    assert_eq!(doc.enhancements.len(), 1);
    assert!(doc.enhancements[0].relevant);
    assert_eq!((doc.enhancements[0].begin, doc.enhancements[0].end), (0, 5));
    assert_eq!(doc.enhancements[0].enhance_end, "</span>");
    assert_eq!(doc.enhancements[0].enhance_start, expected);
    // The attributes are spliced into the map's own span tag, not a copy.
    assert_eq!(map[&Word::new(0, 0, 5)].get_span_tag_start(), expected);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-distractors-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-distractors-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-distractors-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-distractors-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-distractors-fn/test]
#[test]
fn a_single_surviving_form_leaves_a_token_alone() {
    let start = "<span id=\"s1\" class=\"c\">";
    let mut map = span_map(0, &[(0, 5, start)]);
    let mut doc = Document::new("beana", "sme");
    let output = concat!(
        "beana+N+Sg+Nom\tbeana\n",
        "beana+N+Sg+Ill\t+?\n",
        "ñôŃßĘńŠē\n",
        "Word 0 5\n",
    );

    attach_distractors(&mut doc, 0, QUIET, output, &mut map).unwrap();

    assert!(doc.enhancements.is_empty());
    assert_eq!(map[&Word::new(0, 0, 5)].get_span_tag_start(), start);

    // A Word line seen before any marker is ignored for the same reason,
    // even when nothing in the map could have matched it.
    let mut empty: HashMap<Word, SpanTag> = HashMap::new();
    let mut doc = Document::new("beana", "sme");

    attach_distractors(&mut doc, 0, QUIET, "Word 0 5\n", &mut empty).unwrap();

    assert!(doc.enhancements.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-distractors-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-distractors-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-distractors-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-distractors-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-distractors-fn/test]
#[test]
fn every_word_line_reuses_the_open_block() {
    let mut map = span_map(
        0,
        &[
            (0, 5, "<span id=\"s1\" class=\"c\">"),
            (6, 11, "<span id=\"s2\" class=\"c\">"),
        ],
    );
    let mut doc = Document::new("beana beana", "sme");
    let output = concat!(
        "beana+N+Sg+Acc\tbeana\n",
        "beana+N+Sg+Ill\tbeatnagii\n",
        "ñôŃßĘńŠē\n",
        "Word 0 5\n",
        "Word 6 11\n",
    );

    attach_distractors(&mut doc, 0, QUIET, output, &mut map).unwrap();

    assert_eq!(doc.enhancements.len(), 2);
    assert_eq!(
        doc.enhancements[0].enhance_start,
        "<span id=\"s1\" class=\"c\"distractors=\"beana beatnagii\"answer=\"beatnagii\">"
    );
    assert_eq!(
        doc.enhancements[1].enhance_start,
        "<span id=\"s2\" class=\"c\"distractors=\"beana beatnagii\"answer=\"beatnagii\">"
    );
    assert_eq!(
        (doc.enhancements[1].begin, doc.enhancements[1].end),
        (6, 11)
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-distractors-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-distractors-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-distractors-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-distractors-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-distractors-fn/test]
#[test]
fn a_later_marker_replaces_the_earlier_block() {
    let mut map = span_map(0, &[(0, 1, "<s>"), (2, 3, "<s>")]);
    let mut doc = Document::new("a b", "sme");
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

    attach_distractors(&mut doc, 0, QUIET, output, &mut map).unwrap();

    // The duplicate and the hyphenated form are dropped, and the answer is
    // the last whitespace-separated field of the block whether or not the
    // generator managed to produce it.
    assert_eq!(doc.enhancements.len(), 2);
    assert!(
        doc.enhancements[0]
            .enhance_start
            .contains("distractors=\"form1 formA\"answer=\"formA\"")
    );
    assert_eq!((doc.enhancements[0].begin, doc.enhancements[0].end), (0, 1));
    assert!(
        doc.enhancements[1]
            .enhance_start
            .contains("distractors=\"form3 form4\"answer=\"form4\"")
    );
    assert_eq!((doc.enhancements[1].begin, doc.enhancements[1].end), (2, 3));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-distractors-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-distractors-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-distractors-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-distractors-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-distractors-fn/test]
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
        let mut map = span_map(0, &[(0, 1, "<s>")]);
        let mut doc = Document::new("a", "sme");

        let err = attach_distractors(&mut doc, 0, QUIET, &format!("{block}{record}"), &mut map)
            .expect_err(record);

        assert!(err.to_string().starts_with(message), "{err}");
        assert!(doc.enhancements.is_empty());
    }
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-possible-forms-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-possible-forms-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-possible-forms-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-possible-forms-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-possible-forms-fn/test]
#[test]
fn a_single_form_enhances_a_cloze_token() {
    let mut map = span_map(0, &[(0, 5, "<span id=\"s1\" class=\"c\">")]);
    let mut doc = Document::new("beana", "sme");
    let output = concat!(
        "beana+N+Sg+Nom\tbeana\n",
        "beana+N+Sg+Ill\t+?\n",
        "\n",
        "ñôŃßĘńŠē\n",
        "Word 0 5\n",
    );

    attach_possible_forms(&mut doc, 0, LOUD, output, &mut map).unwrap();

    assert_eq!(doc.enhancements.len(), 1);
    assert!(doc.enhancements[0].relevant);
    assert_eq!((doc.enhancements[0].begin, doc.enhancements[0].end), (0, 5));
    assert_eq!(doc.enhancements[0].enhance_end, "</span>");
    assert_eq!(
        doc.enhancements[0].enhance_start,
        "<span id=\"s1\" class=\"c\"possibleforms=\"beana\">"
    );
    // No answer attribute is added on this path, and one form is enough.
    assert!(!doc.enhancements[0].enhance_start.contains("answer="));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-possible-forms-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-possible-forms-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-possible-forms-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-possible-forms-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-possible-forms-fn/test]
#[test]
fn duplicate_forms_collapse_and_empty_blocks_skip() {
    let mut map = span_map(0, &[(0, 1, "<s1>"), (2, 3, "<s2>")]);
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

    attach_possible_forms(&mut doc, 0, QUIET, output, &mut map).unwrap();

    // The duplicate and the hyphenated form are dropped, and the second
    // record generated nothing at all so its Word line is ignored.
    assert_eq!(doc.enhancements.len(), 1);
    assert_eq!(
        doc.enhancements[0].enhance_start,
        "<s1possibleforms=\"boahtit\">"
    );
    assert_eq!(map[&Word::new(0, 2, 3)].get_span_tag_start(), "<s2>");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-possible-forms-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-possible-forms-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-possible-forms-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-possible-forms-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-possible-forms-fn/test]
#[test]
fn the_cloze_reader_shares_the_record_failures() {
    let block = "a+V+Inf\tboahtit\nñôŃßĘńŠē\n";
    let cases = [
        ("Word 4 5\n", "no span tag for Word 4 5"),
        ("Word x 5\n", "For input string: \"x\""),
        ("Word\n", "Index 2 out of bounds for length 1"),
    ];

    for (record, message) in cases {
        let mut map = span_map(0, &[(0, 1, "<s>")]);
        let mut doc = Document::new("a", "sme");

        let err = attach_possible_forms(&mut doc, 0, QUIET, &format!("{block}{record}"), &mut map)
            .expect_err(record);

        assert!(err.to_string().starts_with(message), "{err}");
        assert!(doc.enhancements.is_empty());
    }
}
