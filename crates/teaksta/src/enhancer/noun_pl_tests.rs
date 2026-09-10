use super::*;
use crate::types::CgToken;

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

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.remove-tags-fn/test]
#[test]
fn remove_tags_strips_literal_entries_and_angle_tag() {
    let enhancer = Vislcg3NounPlEnhancer::default();

    assert_eq!(
        enhancer.remove_tags("beana+N+<sme>+Pl+Nom\n"),
        "beana+N+Pl+Nom\n"
    );
    assert_eq!(
        enhancer.remove_tags("beana+N+Err/CmpSub+Allegro+Pl+Gen"),
        "beana+N+Pl+Gen"
    );
    assert_eq!(enhancer.remove_tags("beana+N+Pl+Loc"), "beana+N+Pl+Loc");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.remove-tags-fn/test]
#[test]
fn remove_tags_leaves_residue_from_err_orth_variants() {
    let enhancer = Vislcg3NounPlEnhancer::default();

    assert_eq!(
        enhancer.remove_tags("beana+N+Err/Orth-a-á+Pl+Nom"),
        "beana+N-a-á+Pl+Nom"
    );
    assert_eq!(
        enhancer.remove_tags("beana+N+Err/Orth-nom-gen+Pl+Gen"),
        "beana+N-nom-gen+Pl+Gen"
    );
    assert_eq!(
        enhancer.remove_tags("beana+N+Err/Orth-nom-acc+Pl+Acc"),
        "beana+N-nom-acc+Pl+Acc"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.remove-tags-fn/test]
#[test]
fn remove_tags_handles_only_first_distinct_angle_tag() {
    let enhancer = Vislcg3NounPlEnhancer::default();

    assert_eq!(
        enhancer.remove_tags("beana+N+<sme>+Pl+Nom+<sme>"),
        "beana+N+Pl+Nom"
    );
    assert_eq!(
        enhancer.remove_tags("beana+N+<sme>+Pl+<nob>+Nom"),
        "beana+N+Pl+<nob>+Nom"
    );
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

fn distractor_stream() -> String {
    format!(
        "beana+N+Pl+Nom\tbeanat\n\
         beana+N+Pl+Acc\tbeanaid\n\
         beana+N+Pl+Gen\tbeanaid\n\
         beana+N+Pl+Ill\tbeanaide\n\
         beana+N+Pl+Loc\tbeanain\n\
         beana+N+Pl+Com\tbeanaiguin\n\
         beana+N+Pl+Ess\t+?\n\
         beana+N+Pl+Nom\tbeanat\n\
         {}\n\
         Word 0 6\n",
        MARKER
    )
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-distractors-fn/test]
#[test]
fn distractor_span_attaches_unique_forms_and_answer() {
    let enhancer = Vislcg3NounPlEnhancer::default();
    let outer = enhancer.outer_id();
    let mut map = HashMap::new();
    map.insert(
        Word::new(outer, 0, 6),
        SpanTag::new(outer, "<span id=\"s1\" class=\"c\">".to_string()),
    );
    let mut doc = Document::new("beanat", "sme");

    enhancer
        .generate_span_tag_with_distractors(&mut doc, &distractor_stream(), &mut map)
        .expect("distractors");

    let expected = concat!(
        "<span id=\"s1\" class=\"c\"",
        "distractors=\"beanat beanaid beanaide beanain beanaiguin\"",
        "answer=\"beanat\">"
    );
    assert_eq!(doc.enhancements.len(), 1);
    assert!(doc.enhancements[0].relevant);
    assert_eq!(doc.enhancements[0].begin, 0);
    assert_eq!(doc.enhancements[0].end, 6);
    assert_eq!(doc.enhancements[0].enhance_start, expected);
    assert_eq!(doc.enhancements[0].enhance_end, "</span>");
    assert_eq!(
        map[&Word::new(outer, 0, 6)].get_span_tag_start(),
        expected,
        "the span tag in the map is amended in place"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-distractors-fn/test]
#[test]
fn distractor_span_skips_blocks_with_one_unique_form() {
    let enhancer = Vislcg3NounPlEnhancer::default();
    let outer = enhancer.outer_id();
    let start = "<span id=\"s1\" class=\"c\">".to_string();
    let mut map = HashMap::new();
    map.insert(Word::new(outer, 0, 6), SpanTag::new(outer, start.clone()));
    let mut doc = Document::new("beanat", "sme");
    let stream = format!(
        "beana+N+Pl+Nom\tbeanat\nbeana+N+Pl+Acc\t+?\n{}\nWord 0 6\n",
        MARKER
    );

    enhancer
        .generate_span_tag_with_distractors(&mut doc, &stream, &mut map)
        .expect("distractors");

    assert!(doc.enhancements.is_empty());
    assert_eq!(map[&Word::new(outer, 0, 6)].get_span_tag_start(), start);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-distractors-fn/test]
#[test]
fn distractor_span_fails_on_unmapped_word_record() {
    let enhancer = Vislcg3NounPlEnhancer::default();
    let outer = enhancer.outer_id();
    let mut map = HashMap::new();
    map.insert(
        Word::new(outer, 9, 15),
        SpanTag::new(outer, "<span>".to_string()),
    );
    let mut doc = Document::new("beanat", "sme");

    let err = enhancer
        .generate_span_tag_with_distractors(&mut doc, &distractor_stream(), &mut map)
        .unwrap_err();

    assert!(err.to_string().starts_with("no span tag for Word 0 6"));
    assert!(doc.enhancements.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-distractors-fn/test]
#[test]
fn distractor_span_fails_on_non_numeric_word_record() {
    let enhancer = Vislcg3NounPlEnhancer::default();
    let outer = enhancer.outer_id();
    let mut map = HashMap::new();
    map.insert(
        Word::new(outer, 0, 6),
        SpanTag::new(outer, "<span>".to_string()),
    );
    let mut doc = Document::new("beanat", "sme");
    let stream = distractor_stream().replace("Word 0 6", "Word zero 6");

    let err = enhancer
        .generate_span_tag_with_distractors(&mut doc, &stream, &mut map)
        .unwrap_err();

    assert_eq!(err.to_string(), "For input string: \"zero\"");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-possible-forms-fn/test]
#[test]
fn possible_forms_span_keeps_single_unique_form() {
    let enhancer = Vislcg3NounPlEnhancer::default();
    let outer = enhancer.outer_id();
    let mut map = HashMap::new();
    map.insert(
        Word::new(outer, 0, 6),
        SpanTag::new(outer, "<span id=\"s1\" class=\"c\">".to_string()),
    );
    let mut doc = Document::new("beanat", "sme");
    let stream = format!("beana+N+Pl+Nom\tbeanat\n{}\nWord 0 6\n", MARKER);

    enhancer
        .generate_span_tag_with_possible_forms(&mut doc, &stream, &mut map)
        .expect("possible forms");

    assert_eq!(doc.enhancements.len(), 1);
    assert_eq!(
        doc.enhancements[0].enhance_start,
        "<span id=\"s1\" class=\"c\"possibleforms=\"beanat\">"
    );
    assert_eq!(doc.enhancements[0].enhance_end, "</span>");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-possible-forms-fn/test]
#[test]
fn possible_forms_span_drops_duplicates_and_marked_forms() {
    let enhancer = Vislcg3NounPlEnhancer::default();
    let outer = enhancer.outer_id();
    let mut map = HashMap::new();
    map.insert(
        Word::new(outer, 2, 8),
        SpanTag::new(outer, "<span>".to_string()),
    );
    let mut doc = Document::new("  guolit", "sme");
    let stream = format!(
        "guolli+N+Pl+Nom\tguolit\n\
         guolli+N+Pl+Acc\tguliid\n\
         guolli+N+Pl+Gen\tguliid\n\
         guolli+N+Pl+Ess\t+?\n\
         guolli+N+Pl+Com\tguoli-guin\n\
         {}\n\
         Word 2 8\n",
        MARKER
    );

    enhancer
        .generate_span_tag_with_possible_forms(&mut doc, &stream, &mut map)
        .expect("possible forms");

    assert_eq!(doc.enhancements.len(), 1);
    assert_eq!(doc.enhancements[0].begin, 2);
    assert_eq!(doc.enhancements[0].end, 8);
    assert_eq!(
        doc.enhancements[0].enhance_start,
        "<spanpossibleforms=\"guolit guliid\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.mutable-int.increment-fn/test]
#[test]
fn mutable_int_increment_counts_up_from_one() {
    let mut counter = MutableInt::default();
    assert_eq!(counter.value, 1);

    counter.increment();
    counter.increment();

    assert_eq!(counter.value, 3);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.mutable-int.get-fn/test]
#[test]
fn mutable_int_get_returns_the_current_value() {
    let mut counter = MutableInt::default();
    assert_eq!(counter.get(), 1);

    counter.increment();

    assert_eq!(counter.get(), 2);
    assert_eq!(counter.get(), 2);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.word-fn/test]
#[test]
fn word_constructor_stores_offsets_verbatim() {
    let word = Word::new(4, 12, 3);

    assert_eq!(word.begin, 12);
    assert_eq!(word.end, 3);

    let empty = Word::empty(4);
    assert_eq!(empty.begin, 0);
    assert_eq!(empty.end, 0);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-begin-fn/test]
#[test]
fn word_get_begin_returns_the_start_offset() {
    assert_eq!(Word::new(0, 17, 23).get_begin(), 17);
    assert_eq!(Word::empty(0).get_begin(), 0);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.set-begin-fn/test]
#[test]
fn word_set_begin_overwrites_the_start_offset() {
    let mut word = Word::new(0, 17, 23);

    word.set_begin(99);

    assert_eq!(word.get_begin(), 99);
    assert_eq!(word.get_end(), 23);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-end-fn/test]
#[test]
fn word_get_end_returns_the_exclusive_end_offset() {
    assert_eq!(Word::new(0, 17, 23).get_end(), 23);
    assert_eq!(Word::empty(0).get_end(), 0);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.set-end-fn/test]
#[test]
fn word_set_end_overwrites_the_end_offset() {
    let mut word = Word::new(0, 17, 23);

    word.set_end(5);

    assert_eq!(word.get_end(), 5);
    assert_eq!(word.get_begin(), 17);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.hash-code-fn/test]
#[test]
fn word_hash_code_folds_outer_begin_and_end() {
    assert_eq!(Word::new(0, 3, 7).hash_code(), 29891);
    assert_eq!(Word::new(2, 3, 7).hash_code(), 31813);
    assert_eq!(
        Word::new(0, 3, 7).hash_code(),
        Word::new(0, 3, 7).hash_code()
    );
    assert_ne!(
        Word::new(0, 3, 7).hash_code(),
        Word::new(0, 7, 3).hash_code()
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.equals-fn/test]
#[test]
fn word_equals_requires_same_enhancer_and_both_offsets() {
    let word = Word::new(1, 3, 7);

    assert!(word.equals(&Word::new(1, 3, 7)));
    assert!(!word.equals(&Word::new(2, 3, 7)));
    assert!(!word.equals(&Word::new(1, 4, 7)));
    assert!(!word.equals(&Word::new(1, 3, 8)));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-outer-type-fn/test]
#[test]
fn word_get_outer_type_returns_the_enclosing_identity() {
    let enhancer = Vislcg3NounPlEnhancer::default();

    assert_eq!(Word::new(42, 1, 2).get_outer_type(), 42);
    assert_eq!(Word::empty(42).get_outer_type(), 42);
    assert_eq!(
        Word::new(enhancer.outer_id(), 1, 2).get_outer_type(),
        enhancer.outer_id()
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.to-string-fn/test]
#[test]
fn word_to_string_is_the_generator_wire_record() {
    assert_eq!(Word::new(0, 12, 19).to_string(), "Word 12 19\n");

    let record = Word::new(0, 12, 19).to_string();
    let parts = split_ws(java_trim(&record));
    assert_eq!(parts, vec!["Word", "12", "19"]);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.span-tag-fn/test]
#[test]
fn span_tag_constructor_stores_start_and_defaults_end() {
    let tag = SpanTag::new(3, "<span id=\"x\">".to_string());

    assert_eq!(tag.span_tag_start, "<span id=\"x\">");
    assert_eq!(tag.span_tag_end, "</span>");
    assert_eq!(tag.outer, 3);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-span-tag-start-fn/test]
#[test]
fn span_tag_get_start_includes_injected_attributes() {
    let mut tag = SpanTag::new(0, "<span id=\"x\">".to_string());
    assert_eq!(tag.get_span_tag_start(), "<span id=\"x\">");

    tag.add_attribute("lemma", "beana");

    assert_eq!(tag.get_span_tag_start(), "<span id=\"x\"lemma=\"beana\">");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.set-span-tag-start-fn/test]
#[test]
fn span_tag_set_start_discards_injected_attributes() {
    let mut tag = SpanTag::new(0, "<span id=\"x\">".to_string());
    tag.add_attribute("lemma", "beana");

    tag.set_span_tag_start("<span id=\"y\">".to_string());

    assert_eq!(tag.get_span_tag_start(), "<span id=\"y\">");
    assert_eq!(tag.get_span_tag_end(), "</span>");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.add-attribute-fn/test]
#[test]
fn span_tag_add_attribute_splices_without_a_space() {
    let mut tag = SpanTag::new(0, "<span id=\"x\" class=\"y\">".to_string());

    tag.add_attribute("lemma", "beana");

    assert_eq!(
        tag.get_span_tag_start(),
        "<span id=\"x\" class=\"y\"lemma=\"beana\">"
    );

    tag.add_attribute("answer", "beanat");

    assert_eq!(
        tag.get_span_tag_start(),
        "<span id=\"x\" class=\"y\"lemma=\"beana\"answer=\"beanat\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.add-attribute-fn/test]
#[test]
fn span_tag_add_attribute_corrupts_on_angle_value() {
    let mut tag = SpanTag::new(0, "<span>".to_string());

    tag.add_attribute("lemma", "a>b");
    assert_eq!(tag.get_span_tag_start(), "<spanlemma=\"a>b\">");

    tag.add_attribute("answer", "c");

    assert_eq!(
        tag.get_span_tag_start(),
        "<spanlemma=\"aanswer=\"c\">b\"answer=\"c\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-span-tag-end-fn/test]
#[test]
fn span_tag_get_end_returns_closing_tag() {
    let mut tag = SpanTag::new(0, "<span>".to_string());
    assert_eq!(tag.get_span_tag_end(), "</span>");

    tag.add_attribute("lemma", "beana");

    assert_eq!(tag.get_span_tag_end(), "</span>");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.set-span-tag-end-fn/test]
#[test]
fn span_tag_set_span_tag_end_overwrites_verbatim() {
    let mut tag = SpanTag::new(0, "<span>".to_string());

    tag.set_span_tag_end("</div>".to_string());

    assert_eq!(tag.get_span_tag_end(), "</div>");
    assert_eq!(tag.get_span_tag_start(), "<span>");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.hash-code-fn/test]
#[test]
fn span_tag_hash_code_folds_outer_end_start() {
    let mut tag = SpanTag::new(0, "<span>".to_string());
    assert_eq!(tag.hash_code(), 1183638870);
    assert_eq!(
        tag.hash_code(),
        SpanTag::new(0, "<span>".to_string()).hash_code()
    );
    assert_ne!(
        tag.hash_code(),
        SpanTag::new(1, "<span>".to_string()).hash_code()
    );

    let before = tag.hash_code();
    tag.add_attribute("lemma", "beana");

    assert_ne!(tag.hash_code(), before);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.equals-fn/test]
#[test]
fn span_tag_equals_compares_enclosing_instance_and_tags() {
    let tag = SpanTag::new(1, "<span>".to_string());

    assert!(tag.equals(&SpanTag::new(1, "<span>".to_string())));
    assert!(!tag.equals(&SpanTag::new(2, "<span>".to_string())));
    assert!(!tag.equals(&SpanTag::new(1, "<div>".to_string())));

    let mut other_end = SpanTag::new(1, "<span>".to_string());
    other_end.set_span_tag_end("</div>".to_string());
    assert!(!tag.equals(&other_end));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-outer-type-fn/test]
#[test]
fn span_tag_get_outer_type_returns_enclosing_identity() {
    let enhancer = Vislcg3NounPlEnhancer::default();

    assert_eq!(SpanTag::new(42, "<span>".to_string()).get_outer_type(), 42);
    assert_eq!(
        SpanTag::new(enhancer.outer_id(), "<span>".to_string()).get_outer_type(),
        enhancer.outer_id()
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.to-string-fn/test]
#[test]
fn span_tag_to_string_renders_both_fields() {
    let mut tag = SpanTag::new(0, "<span id=\"x\">".to_string());
    assert_eq!(
        tag.to_string(),
        "SpanTag [spanTagStart=<span id=\"x\">, spanTagEnd=</span>]"
    );

    tag.set_span_tag_end("</div>".to_string());

    assert_eq!(
        tag.to_string(),
        "SpanTag [spanTagStart=<span id=\"x\">, spanTagEnd=</div>]"
    );
}
