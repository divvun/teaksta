use super::*;
use crate::types::CgToken;

fn enhancer() -> Vislcg3NounEnhancer {
    Vislcg3NounEnhancer {
        enhancement_type: String::new(),
        n_tags: None,
    }
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

fn span_map(enh: &Vislcg3NounEnhancer, entries: &[(usize, usize, &str)]) -> HashMap<Word, SpanTag> {
    entries
        .iter()
        .map(|(begin, end, start)| {
            (
                Word::new(enh.outer_id(), *begin, *end),
                SpanTag::new(enh.outer_id(), start.to_string()),
            )
        })
        .collect()
}

/// `process` only reaches the generator seam for `mc` and `cloze`; every
/// other value of the shared activity field takes the in-process branch
/// that the assertions below describe.
fn activity_reaches_the_generator() -> bool {
    let activity = crate::server::servlet::enhancement_type();
    activity == "mc" || activity == "cloze"
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

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.process-fn/test]
#[test]
fn a_singular_noun_span_carries_its_lemma() {
    if activity_reaches_the_generator() {
        return;
    }
    let enh = enhancer();
    let mut doc = Document::new("čáhci čáhci", "sme");
    let reading: &[&str] = &["\"čáhci\"", "N", "<sme>", "Sem/Plc", "Sg", "Nom"];
    doc.cg_tokens.push(cg_token(0, 7, &[reading]));
    doc.cg_tokens.push(cg_token(8, 15, &[reading]));

    enh.process(&mut doc).unwrap();

    assert_eq!(doc.enhancements.len(), 2);
    assert!(doc.enhancements[0].relevant);
    assert_eq!((doc.enhancements[0].begin, doc.enhancements[0].end), (0, 7));
    assert_eq!(doc.enhancements[0].enhance_end, "</span>");
    assert_eq!(
        doc.enhancements[0].enhance_start,
        "<span id=\"WERTi-span-čáhci-N-xsmey-Sem/Plc-Sg-Nom-1\" \
         class=\"wertiviewtoken  wertiviewSubstantive\"lemma=\"čáhci\">"
    );
    assert_eq!(
        doc.enhancements[1].enhance_start,
        "<span id=\"WERTi-span-čáhci-N-xsmey-Sem/Plc-Sg-Nom-2\" \
         class=\"wertiviewtoken  wertiviewSubstantive\"lemma=\"čáhci\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.process-fn/test]
#[test]
fn preposition_hint_is_linked_from_next_noun() {
    if activity_reaches_the_generator() {
        return;
    }
    let enh = enhancer();
    let mut doc = Document::new("maŋŋel beana", "sme");
    doc.cg_tokens.push(cg_token(0, 8, &[&["\"maŋŋel\"", "Pr"]]));
    doc.cg_tokens
        .push(cg_token(9, 14, &[&["\"beana\"", "N", "Sg", "Nom"]]));

    enh.process(&mut doc).unwrap();

    assert_eq!(doc.enhancements.len(), 2);
    assert_eq!(
        doc.enhancements[0].enhance_start,
        "<span id=\"WERTi-span-maŋŋel-Pr-1\" class=\"wertiviewhinttag\">"
    );
    assert_eq!((doc.enhancements[0].begin, doc.enhancements[0].end), (0, 8));
    assert_eq!(
        doc.enhancements[1].enhance_start,
        "<span id=\"WERTi-span-beana-N-Sg-Nom-1\" \
         class=\"wertiviewtoken  wertiviewSubstantive\"lemma=\"beana\"\
         hintid=\"WERTi-span-maŋŋel-Pr-1\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.process-fn/test]
#[test]
fn a_bare_sg_tag_without_case_still_qualifies() {
    if activity_reaches_the_generator() {
        return;
    }
    let enh = enhancer();
    let mut doc = Document::new("ruoktu", "sme");
    doc.cg_tokens
        .push(cg_token(0, 6, &[&["\"ruoktu\"", "N", "Sg"]]));

    enh.process(&mut doc).unwrap();

    assert_eq!(doc.enhancements.len(), 1);
    assert_eq!(
        doc.enhancements[0].enhance_start,
        "<span id=\"WERTi-span-ruoktu-N-Sg-1\" \
         class=\"wertiviewtoken  wertiviewSubstantive\"lemma=\"ruoktu\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.process-fn/test]
#[test]
fn one_excluded_reading_disqualifies_a_token() {
    if activity_reaches_the_generator() {
        return;
    }
    let enh = enhancer();
    let mut doc = Document::new("mun", "sme");
    doc.cg_tokens.push(cg_token(
        0,
        3,
        &[
            &["\"mun\"", "N", "Sg", "Nom"],
            &["\"mun\"", "Pron", "Pers", "Sg1", "Nom"],
        ],
    ));

    enh.process(&mut doc).unwrap();

    assert!(doc.enhancements.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.remove-tags-fn/test]
#[test]
fn literal_tags_are_deleted_wherever_they_occur() {
    let enh = enhancer();

    assert_eq!(
        enh.remove_tags("beana+N+Allegro+Err/Spellrelax+Sg+Nom"),
        "beana+N+Sg+Nom"
    );
    assert_eq!(enh.remove_tags("beana+N+<sme>+Sg+Nom"), "beana+N+Sg+Nom");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.remove-tags-fn/test]
#[test]
fn err_orth_prefix_leaves_hyphenated_suffix() {
    let enh = enhancer();

    assert_eq!(
        enh.remove_tags("beana+N+Err/Orth-a-á+Sg+Nom"),
        "beana+N-a-á+Sg+Nom"
    );
    assert_eq!(
        enh.remove_tags("beana+N+Err/Orth-nom-gen+Sg+Gen"),
        "beana+N-nom-gen+Sg+Gen"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.remove-tags-fn/test]
#[test]
fn only_the_first_angle_bracket_tag_matches() {
    let enh = enhancer();

    assert_eq!(
        enh.remove_tags("beana+N+<sme>+Sg+Nom+<compl_subj>"),
        "beana+N+Sg+Nom+<compl_subj>"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-morphological-forms-fn/test]
#[test]
fn four_distractor_analyses_precede_the_correct_one() {
    let enh = enhancer();

    let block = enh.write_morphological_forms("beana+N+Sg+Nom").unwrap();

    assert_eq!(
        block,
        "beana+N+Sg+Acc\nbeana+N+Sg+Ill\nbeana+N+Sg+Loc\nbeana+N+Sg+Com\nbeana+N+Sg+Nom\n"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-morphological-forms-fn/test]
#[test]
fn syntactic_tag_dropped_from_correct_answer_line() {
    let enh = enhancer();

    let block = enh
        .write_morphological_forms("beana+N+Sg+Nom+@SUBJ>")
        .unwrap();

    assert!(block.ends_with("beana+N+Sg+Nom\n"), "{block}");
    assert!(!block.contains('@'), "{block}");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-morphological-forms-fn/test]
#[test]
fn plural_readings_draw_from_the_plural_distractor_table() {
    let enh = enhancer();

    let block = enh.write_morphological_forms("beana+N+Pl+Com").unwrap();

    assert_eq!(
        block,
        "beana+N+Pl+Nom\nbeana+N+Pl+Ill\nbeana+N+Ess\nbeana+N+Sg+Acc\nbeana+N+Pl+Com\n"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-morphological-forms-fn/test]
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

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-morphological-forms-fn/test]
#[test]
fn a_bare_case_marker_without_number_fails() {
    let enh = enhancer();

    let err = enh
        .write_morphological_forms("guolli+N+Loc+Sg+Nom")
        .unwrap_err();

    assert!(err.to_string().contains("end -1"), "{err}");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-lemma-and-analyses-fn/test]
#[test]
fn a_singular_genitive_gains_its_plural_counterpart() {
    let enh = enhancer();

    let block = enh
        .write_lemma_and_analyses("beana+N+<sme>+Sem/Ani+Sg+Gen+@SUBJ>")
        .unwrap();

    assert_eq!(block, "beana+N+Sem/Ani+Sg+Gen\nbeana+N+Sem/Ani+Pl+Gen\n");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-lemma-and-analyses-fn/test]
#[test]
fn a_plural_locative_gains_its_singular_counterpart() {
    let enh = enhancer();

    let block = enh
        .write_lemma_and_analyses("beana+N+<sme>+Pl+Loc")
        .unwrap();

    assert_eq!(block, "beana+N+Pl+Loc\nbeana+N+Sg+Loc\n");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-lemma-and-analyses-fn/test]
#[test]
fn nominative_readings_yield_a_single_line() {
    let enh = enhancer();

    let block = enh
        .write_lemma_and_analyses("beana+N+<sme>+Sg+Nom")
        .unwrap();

    assert_eq!(block, "beana+N+Sg+Nom\n");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-lemma-and-analyses-fn/test]
#[test]
fn a_reading_without_a_plus_fails() {
    let enh = enhancer();

    let err = enh.write_lemma_and_analyses("beana").unwrap_err();

    assert!(err.to_string().contains("end -1"), "{err}");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-distractors-fn/test]
#[test]
fn distinct_forms_become_distractors_last_is_answer() {
    let enh = enhancer();
    let mut map = span_map(&enh, &[(0, 5, "<span id=\"s1\" class=\"c\">")]);
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

    enh.generate_span_tag_with_distractors(&mut doc, output, &mut map)
        .unwrap();

    assert_eq!(doc.enhancements.len(), 1);
    assert!(doc.enhancements[0].relevant);
    assert_eq!((doc.enhancements[0].begin, doc.enhancements[0].end), (0, 5));
    assert_eq!(doc.enhancements[0].enhance_end, "</span>");
    assert_eq!(
        doc.enhancements[0].enhance_start,
        "<span id=\"s1\" class=\"c\"\
         distractors=\"beana beatnagii beatnagis beatnagiin\"answer=\"beana\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-distractors-fn/test]
#[test]
fn a_single_surviving_form_leaves_the_token_unenhanced() {
    let enh = enhancer();
    let mut map = span_map(&enh, &[(0, 5, "<span id=\"s1\" class=\"c\">")]);
    let mut doc = Document::new("beana", "sme");
    let output = concat!(
        "beana+N+Sg+Nom\tbeana\n",
        "beana+N+Sg+Ill\t+?\n",
        "ñôŃßĘńŠē\n",
        "Word 0 5\n",
    );

    enh.generate_span_tag_with_distractors(&mut doc, output, &mut map)
        .unwrap();

    assert!(doc.enhancements.is_empty());
    assert_eq!(
        map[&Word::new(enh.outer_id(), 0, 5)].get_span_tag_start(),
        "<span id=\"s1\" class=\"c\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-distractors-fn/test]
#[test]
fn second_word_line_reuses_the_same_distractors() {
    let enh = enhancer();
    let mut map = span_map(
        &enh,
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

    enh.generate_span_tag_with_distractors(&mut doc, output, &mut map)
        .unwrap();

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
#[test]
fn offsets_that_were_never_registered_abort_the_run() {
    let enh = enhancer();
    let mut map = span_map(&enh, &[(0, 5, "<span id=\"s1\" class=\"c\">")]);
    let mut doc = Document::new("beana", "sme");
    let output = concat!(
        "beana+N+Sg+Acc\tbeana\n",
        "beana+N+Sg+Ill\tbeatnagii\n",
        "ñôŃßĘńŠē\n",
        "Word 20 25\n",
    );

    let err = enh
        .generate_span_tag_with_distractors(&mut doc, output, &mut map)
        .unwrap_err();

    assert!(err.to_string().contains("no span tag"), "{err}");
    assert!(doc.enhancements.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-possible-forms-fn/test]
#[test]
fn a_single_form_enhances_a_cloze_token() {
    let enh = enhancer();
    let mut map = span_map(&enh, &[(0, 5, "<span id=\"s1\" class=\"c\">")]);
    let mut doc = Document::new("beana", "sme");
    let output = concat!(
        "beana+N+Sg+Nom\tbeana\n",
        "beana+N+Sg+Ill\t+?\n",
        "\n",
        "ñôŃßĘńŠē\n",
        "Word 0 5\n",
    );

    enh.generate_span_tag_with_possible_forms(&mut doc, output, &mut map)
        .unwrap();

    assert_eq!(doc.enhancements.len(), 1);
    assert!(doc.enhancements[0].relevant);
    assert_eq!((doc.enhancements[0].begin, doc.enhancements[0].end), (0, 5));
    assert_eq!(doc.enhancements[0].enhance_end, "</span>");
    assert_eq!(
        doc.enhancements[0].enhance_start,
        "<span id=\"s1\" class=\"c\"possibleforms=\"beana\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-possible-forms-fn/test]
#[test]
fn duplicate_forms_collapse_and_no_answer_added() {
    let enh = enhancer();
    let mut map = span_map(&enh, &[(0, 5, "<span id=\"s1\" class=\"c\">")]);
    let mut doc = Document::new("beana", "sme");
    let output = concat!(
        "beana+N+Sg+Nom\tbeana\n",
        "beana+N+Sg+Ill\tbeatnagii\n",
        "beana+N+Sg+Acc\tbeana\n",
        "ñôŃßĘńŠē\n",
        "Word 0 5\n",
    );

    enh.generate_span_tag_with_possible_forms(&mut doc, output, &mut map)
        .unwrap();

    assert_eq!(
        doc.enhancements[0].enhance_start,
        "<span id=\"s1\" class=\"c\"possibleforms=\"beana beatnagii\">"
    );
    assert!(!doc.enhancements[0].enhance_start.contains("answer"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.mutable-int.increment-fn/test]
#[test]
fn incrementing_counts_up_from_the_first_occurrence() {
    let mut count = MutableInt::default();

    count.increment();
    count.increment();

    assert_eq!(count.value, 3);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.mutable-int.get-fn/test]
#[test]
fn the_counter_reads_back_its_current_value() {
    let mut count = MutableInt::default();

    assert_eq!(count.get(), 1);
    count.increment();
    assert_eq!(count.get(), 2);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.word-fn/test]
#[test]
fn a_word_stores_its_offsets_verbatim() {
    let word = Word::new(7, 3, 9);

    assert_eq!(word.get_begin(), 3);
    assert_eq!(word.get_end(), 9);

    let inverted = Word::new(7, 9, 3);

    assert_eq!(inverted.get_begin(), 9);
    assert_eq!(inverted.get_end(), 3);

    let placeholder = Word::empty(7);

    assert_eq!(placeholder.get_begin(), 0);
    assert_eq!(placeholder.get_end(), 0);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.get-begin-fn/test]
#[test]
fn the_begin_offset_is_the_inclusive_token_start() {
    assert_eq!(Word::new(0, 12, 18).get_begin(), 12);
    assert_eq!(Word::empty(0).get_begin(), 0);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.get-end-fn/test]
#[test]
fn the_end_offset_is_the_exclusive_token_end() {
    assert_eq!(Word::new(0, 12, 18).get_end(), 18);
    assert_eq!(Word::empty(0).get_end(), 0);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.set-begin-fn/test]
#[test]
fn moving_the_begin_offset_strands_a_map_entry() {
    let mut map: HashMap<Word, &str> = HashMap::new();
    map.insert(Word::new(0, 0, 5), "span");
    let mut key = Word::new(0, 0, 5);

    key.set_begin(1);

    assert_eq!(key.get_begin(), 1);
    assert!(map.get(&key).is_none());

    key.set_begin(0);

    assert_eq!(map.get(&key), Some(&"span"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.set-end-fn/test]
#[test]
fn moving_the_end_offset_strands_a_map_entry() {
    let mut map: HashMap<Word, &str> = HashMap::new();
    map.insert(Word::new(0, 0, 5), "span");
    let mut key = Word::new(0, 0, 5);

    key.set_end(6);

    assert_eq!(key.get_end(), 6);
    assert!(map.get(&key).is_none());

    key.set_end(5);

    assert_eq!(map.get(&key), Some(&"span"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.hash-code-fn/test]
#[test]
fn word_hash_folds_outer_begin_end() {
    assert_eq!(Word::empty(0).hash_code(), 29791);
    assert_eq!(Word::new(0, 3, 9).hash_code(), 29893);
    assert_eq!(Word::new(1, 3, 9).hash_code(), 30854);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.equals-fn/test]
#[test]
fn words_match_on_offsets_and_enhancer() {
    let word = Word::new(1, 4, 8);

    assert!(word.equals(&Word::new(1, 4, 8)));
    assert!(!word.equals(&Word::new(2, 4, 8)));
    assert!(!word.equals(&Word::new(1, 5, 8)));
    assert!(!word.equals(&Word::new(1, 4, 9)));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.get-outer-type-fn/test]
#[test]
fn a_word_reports_its_enhancer() {
    let first = enhancer();
    let second = enhancer();

    let from_first = Word::new(first.outer_id(), 0, 5);
    let from_second = Word::new(second.outer_id(), 0, 5);

    assert_eq!(from_first.get_outer_type(), first.outer_id());
    assert_ne!(first.outer_id(), second.outer_id());
    assert!(!from_first.equals(&from_second));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.to-string-fn/test]
#[test]
fn a_word_renders_as_the_generator_input_record() {
    assert_eq!(Word::new(0, 3, 9).to_string(), "Word 3 9\n");
    assert_eq!(Word::empty(0).to_string(), "Word 0 0\n");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.span-tag-fn/test]
#[test]
fn span_tag_keeps_opening_and_hard_codes_closing() {
    let tag = SpanTag::new(0, "<span id=\"x\">".to_string());

    assert_eq!(tag.get_span_tag_start(), "<span id=\"x\">");
    assert_eq!(tag.get_span_tag_end(), "</span>");

    let unvalidated = SpanTag::new(0, String::new());

    assert_eq!(unvalidated.get_span_tag_start(), "");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.get-span-tag-start-fn/test]
#[test]
fn start_tag_shows_every_attribute_added() {
    let mut tag = SpanTag::new(0, "<span id=\"x\">".to_string());

    assert_eq!(tag.get_span_tag_start(), "<span id=\"x\">");

    tag.add_attribute("lemma", "beana");

    assert_eq!(tag.get_span_tag_start(), "<span id=\"x\"lemma=\"beana\">");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.set-span-tag-start-fn/test]
#[test]
fn overwriting_the_start_tag_discards_earlier_attributes() {
    let mut tag = SpanTag::new(0, "<span id=\"x\">".to_string());
    tag.add_attribute("lemma", "beana");

    tag.set_span_tag_start("<span id=\"y\">".to_string());

    assert_eq!(tag.get_span_tag_start(), "<span id=\"y\">");

    tag.set_span_tag_start(String::new());

    assert_eq!(tag.get_span_tag_start(), "");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.add-attribute-fn/test]
#[test]
fn attributes_are_spliced_in_without_a_separating_space() {
    let mut tag = SpanTag::new(
        0,
        "<span id=\"X\" class=\"wertiviewtoken  wertiviewSubstantive\">".to_string(),
    );

    tag.add_attribute("lemma", "beana");

    assert_eq!(
        tag.get_span_tag_start(),
        "<span id=\"X\" class=\"wertiviewtoken  wertiviewSubstantive\"lemma=\"beana\">"
    );

    tag.add_attribute("hintid", "h-1");

    assert_eq!(
        tag.get_span_tag_start(),
        "<span id=\"X\" class=\"wertiviewtoken  wertiviewSubstantive\"\
         lemma=\"beana\"hintid=\"h-1\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.add-attribute-fn/test]
#[test]
fn every_closing_bracket_gets_an_attribute_spliced() {
    let mut tag = SpanTag::new(0, "<a><b>".to_string());

    tag.add_attribute("k", "v");

    assert_eq!(tag.get_span_tag_start(), "<ak=\"v\"><bk=\"v\">");

    let mut with_bracket_in_value = SpanTag::new(0, "<s>".to_string());
    with_bracket_in_value.add_attribute("a", "x>y");
    with_bracket_in_value.add_attribute("b", "z");

    assert_eq!(
        with_bracket_in_value.get_span_tag_start(),
        "<sa=\"xb=\"z\">y\"b=\"z\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.get-span-tag-end-fn/test]
#[test]
fn the_end_tag_is_unaffected_by_attribute_splicing() {
    let mut tag = SpanTag::new(0, "<span>".to_string());

    tag.add_attribute("lemma", "beana");

    assert_eq!(tag.get_span_tag_end(), "</span>");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.set-span-tag-end-fn/test]
#[test]
fn the_end_tag_can_be_overwritten_without_validation() {
    let mut tag = SpanTag::new(0, "<span>".to_string());

    tag.set_span_tag_end("</div>".to_string());

    assert_eq!(tag.get_span_tag_end(), "</div>");

    tag.set_span_tag_end(String::new());

    assert_eq!(tag.get_span_tag_end(), "");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.hash-code-fn/test]
#[test]
fn span_tag_hash_folds_outer_end_start() {
    assert_eq!(SpanTag::new(0, "<s>".to_string()).hash_code(), -643687099);
    assert_eq!(
        SpanTag::new(0, "<s>".to_string()).hash_code(),
        SpanTag::new(0, "<s>".to_string()).hash_code()
    );
    assert_ne!(
        SpanTag::new(1, "<s>".to_string()).hash_code(),
        SpanTag::new(0, "<s>".to_string()).hash_code()
    );

    let mut with_attribute = SpanTag::new(0, "<s>".to_string());
    with_attribute.add_attribute("k", "v");

    assert_ne!(
        with_attribute.hash_code(),
        SpanTag::new(0, "<s>".to_string()).hash_code()
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.equals-fn/test]
#[test]
fn span_tags_match_on_outer_and_both_halves() {
    let tag = SpanTag::new(1, "<span>".to_string());

    assert!(tag.equals(&SpanTag::new(1, "<span>".to_string())));
    assert!(!tag.equals(&SpanTag::new(2, "<span>".to_string())));
    assert!(!tag.equals(&SpanTag::new(1, "<div>".to_string())));

    let mut different_end = SpanTag::new(1, "<span>".to_string());
    different_end.set_span_tag_end("</div>".to_string());

    assert!(!tag.equals(&different_end));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.get-outer-type-fn/test]
#[test]
fn a_span_tag_reports_its_enhancer() {
    let first = enhancer();
    let second = enhancer();

    let from_first = SpanTag::new(first.outer_id(), "<span>".to_string());
    let from_second = SpanTag::new(second.outer_id(), "<span>".to_string());

    assert_eq!(from_first.get_outer_type(), first.outer_id());
    assert_ne!(first.outer_id(), second.outer_id());
    assert!(!from_first.equals(&from_second));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.to-string-fn/test]
#[test]
fn a_span_tag_renders_both_halves() {
    let tag = SpanTag::new(0, "<span id=\"x\">".to_string());

    assert_eq!(
        tag.to_string(),
        "SpanTag [spanTagStart=<span id=\"x\">, spanTagEnd=</span>]"
    );
}
