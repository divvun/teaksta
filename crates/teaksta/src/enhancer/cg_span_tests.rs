//! The value types are shared by every "big" CG3 topic enhancer, so each
//! test carries the `/test` facet of all five topics' rule for the method it
//! drives.

use std::collections::HashMap;

use super::*;
use crate::enhancer::con_neg::Vislcg3ConNegEnhancer;
use crate::enhancer::noun::Vislcg3NounEnhancer;

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.mutable-int.increment-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.mutable-int.increment-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.mutable-int.increment-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.mutable-int.increment-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.mutable-int.increment-fn/test]
#[test]
fn incrementing_counts_up_from_the_first_occurrence() {
    let mut count = MutableInt::default();

    count.increment();

    assert_eq!(count.value, 2);

    count.increment();

    assert_eq!(count.value, 3);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.mutable-int.get-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.mutable-int.get-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.mutable-int.get-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.mutable-int.get-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.mutable-int.get-fn/test]
#[test]
fn the_counter_reads_back_its_current_value() {
    let mut count = MutableInt::default();

    assert_eq!(count.get(), 1);

    count.increment();

    assert_eq!(count.get(), 2);
    assert_eq!(MutableInt { value: 7 }.get(), 7);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.word-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.word-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.word-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.word-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.word-fn/test]
#[test]
fn a_word_stores_its_offsets_verbatim() {
    let word = Word::new(7, 12, 5);

    assert_eq!(word.outer, 7);
    assert_eq!(word.begin, 12);
    assert_eq!(word.end, 5);

    let placeholder = Word::empty(7);

    assert_eq!((placeholder.begin, placeholder.end), (0, 0));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.get-begin-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-begin-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.get-begin-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.get-begin-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.get-begin-fn/test]
#[test]
fn the_begin_offset_is_the_inclusive_token_start() {
    assert_eq!(Word::new(0, 12, 18).get_begin(), 12);
    assert_eq!(Word::empty(0).get_begin(), 0);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.get-end-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-end-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.get-end-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.get-end-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.get-end-fn/test]
#[test]
fn the_end_offset_is_the_exclusive_token_end() {
    assert_eq!(Word::new(0, 12, 18).get_end(), 18);
    assert_eq!(Word::empty(0).get_end(), 0);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.set-begin-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.set-begin-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.set-begin-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.set-begin-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.set-begin-fn/test]
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
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.set-end-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.set-end-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.set-end-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.set-end-fn/test]
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
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.hash-code-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.hash-code-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.hash-code-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.hash-code-fn/test]
#[test]
fn word_hash_folds_outer_begin_end() {
    assert_eq!(Word::empty(0).hash_code(), 29791);
    assert_eq!(Word::new(0, 3, 9).hash_code(), 29893);
    assert_eq!(Word::new(1, 3, 9).hash_code(), 30854);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.equals-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.equals-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.equals-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.equals-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.equals-fn/test]
#[test]
fn words_match_on_offsets_and_enhancer() {
    let word = Word::new(1, 4, 8);

    assert!(word.equals(&Word::new(1, 4, 8)));
    assert!(!word.equals(&Word::new(2, 4, 8)));
    assert!(!word.equals(&Word::new(1, 5, 8)));
    assert!(!word.equals(&Word::new(1, 4, 9)));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.get-outer-type-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-outer-type-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.get-outer-type-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.get-outer-type-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.get-outer-type-fn/test]
#[test]
fn word_equality_folds_in_the_owning_enhancer() {
    let noun = Vislcg3NounEnhancer::default();
    let con_neg = Vislcg3ConNegEnhancer::default();

    let from_noun = Word::new(noun.outer_id(), 0, 5);
    let from_con_neg = Word::new(con_neg.outer_id(), 0, 5);

    assert_eq!(from_noun.get_outer_type(), noun.outer_id());
    assert_ne!(noun.outer_id(), con_neg.outer_id());
    // The type is shared between the topics now; the enclosing instance
    // folded into equality still is not.
    assert!(!from_noun.equals(&from_con_neg));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.to-string-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.to-string-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.to-string-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.to-string-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.to-string-fn/test]
#[test]
fn a_word_renders_as_the_generator_input_record() {
    assert_eq!(Word::new(0, 3, 9).to_string(), "Word 3 9\n");
    assert_eq!(Word::empty(0).to_string(), "Word 0 0\n");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.span-tag-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.span-tag-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.span-tag-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.span-tag-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.span-tag-fn/test]
#[test]
fn span_tag_keeps_opening_and_hard_codes_closing() {
    let tag = SpanTag::new(0, "<span id=\"x\">".to_string());

    assert_eq!(tag.get_span_tag_start(), "<span id=\"x\">");
    assert_eq!(tag.get_span_tag_end(), "</span>");

    let unvalidated = SpanTag::new(0, String::new());

    assert_eq!(unvalidated.get_span_tag_start(), "");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.get-span-tag-start-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-span-tag-start-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-span-tag-start-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.get-span-tag-start-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.get-span-tag-start-fn/test]
#[test]
fn start_tag_shows_every_attribute_added() {
    let mut tag = SpanTag::new(0, "<span id=\"x\">".to_string());

    assert_eq!(tag.get_span_tag_start(), "<span id=\"x\">");

    tag.add_attribute("lemma", "beana");

    assert_eq!(tag.get_span_tag_start(), "<span id=\"x\"lemma=\"beana\">");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.set-span-tag-start-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.set-span-tag-start-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.set-span-tag-start-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.set-span-tag-start-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.set-span-tag-start-fn/test]
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
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.add-attribute-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.add-attribute-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.add-attribute-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.add-attribute-fn/test]
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
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.add-attribute-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.add-attribute-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.add-attribute-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.add-attribute-fn/test]
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
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-span-tag-end-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-span-tag-end-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.get-span-tag-end-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.get-span-tag-end-fn/test]
#[test]
fn the_end_tag_is_unaffected_by_attribute_splicing() {
    let mut tag = SpanTag::new(0, "<span>".to_string());

    tag.add_attribute("lemma", "beana");

    assert_eq!(tag.get_span_tag_end(), "</span>");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.set-span-tag-end-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.set-span-tag-end-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.set-span-tag-end-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.set-span-tag-end-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.set-span-tag-end-fn/test]
#[test]
fn the_end_tag_can_be_overwritten_without_validation() {
    let mut tag = SpanTag::new(0, "<span>".to_string());

    tag.set_span_tag_end("</div>".to_string());

    assert_eq!(tag.get_span_tag_end(), "</div>");

    tag.set_span_tag_end(String::new());

    assert_eq!(tag.get_span_tag_end(), "");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.hash-code-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.hash-code-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.hash-code-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.hash-code-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.hash-code-fn/test]
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
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.equals-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.equals-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.equals-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.equals-fn/test]
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
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-outer-type-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-outer-type-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.get-outer-type-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.get-outer-type-fn/test]
#[test]
fn span_equality_folds_in_the_owning_enhancer() {
    let noun = Vislcg3NounEnhancer::default();
    let con_neg = Vislcg3ConNegEnhancer::default();

    let from_noun = SpanTag::new(noun.outer_id(), "<span>".to_string());
    let from_con_neg = SpanTag::new(con_neg.outer_id(), "<span>".to_string());

    assert_eq!(from_noun.get_outer_type(), noun.outer_id());
    assert_ne!(noun.outer_id(), con_neg.outer_id());
    assert!(!from_noun.equals(&from_con_neg));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.to-string-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.to-string-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.to-string-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.to-string-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.to-string-fn/test]
#[test]
fn a_span_tag_renders_both_halves() {
    let tag = SpanTag::new(0, "<span id=\"x\">".to_string());

    assert_eq!(
        tag.to_string(),
        "SpanTag [spanTagStart=<span id=\"x\">, spanTagEnd=</span>]"
    );
}
