//! The value types are shared by every "big" CG3 topic enhancer, so each
//! test carries the `/test` facet of all five topics' rule for the method it
//! drives.

use std::collections::HashMap;

use super::*;

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.word-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.word-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.word-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.word-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.word-fn+2/test]
#[test]
fn a_word_stores_its_offsets_verbatim() {
    let word = Word::new(12, 5);

    assert_eq!((word.begin, word.end), (12, 5));

    // The offsets are the whole identity, so two words over the same span
    // are one key whichever enhancer built them.
    let mut map: HashMap<Word, &str> = HashMap::new();
    map.insert(Word::new(0, 5), "span");

    assert_eq!(map.get(&Word::new(0, 5)), Some(&"span"));
    assert!(map.get(&Word::new(1, 5)).is_none());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.to-string-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.to-string-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.to-string-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.to-string-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.word.to-string-fn+2/test]
#[test]
fn a_word_renders_as_generator_input_record() {
    assert_eq!(Word::new(3, 9).to_string(), "Word 3 9");
    assert_eq!(Word::new(0, 0).to_string(), "Word 0 0");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.span-tag-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.span-tag-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.span-tag-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.span-tag-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.span-tag-fn+2/test]
#[test]
fn a_new_span_tag_holds_id_and_classes() {
    let tag = SpanTag::new("x", &[TOKEN_CLASS, "teaksta-Substantive"]);

    assert_eq!(
        tag.start_tag(),
        "<span id=\"x\" class=\"teaksta-token teaksta-Substantive\">"
    );
    assert_eq!(tag.end_tag(), "</span>");

    // Classes are separated by exactly one space, and a tag carrying none
    // gets no class attribute at all.
    assert_eq!(SpanTag::new("x", &[]).start_tag(), "<span id=\"x\">");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.add-attribute-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.add-attribute-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.add-attribute-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.add-attribute-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.add-attribute-fn+3/test]
#[test]
fn attributes_render_in_the_order_added() {
    let mut tag = SpanTag::new("X", &[TOKEN_CLASS, "teaksta-Substantive"]);

    tag.add_attribute("lemma", "beana");
    tag.add_attribute("hintid", "h-1");

    assert_eq!(
        tag.start_tag(),
        "<span id=\"X\" class=\"teaksta-token teaksta-Substantive\" \
         lemma=\"beana\" hintid=\"h-1\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.add-attribute-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.add-attribute-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.add-attribute-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.add-attribute-fn+3/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.add-attribute-fn+3/test]
#[test]
fn a_repeated_name_replaces_its_earlier_value() {
    let mut tag = SpanTag::new("x", &[]);

    tag.add_attribute("answer", "first");
    tag.add_attribute("lemma", "beana");
    tag.add_attribute("answer", "second");

    assert_eq!(
        tag.start_tag(),
        "<span id=\"x\" answer=\"second\" lemma=\"beana\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.get-span-tag-start-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-span-tag-start-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-span-tag-start-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.get-span-tag-start-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.get-span-tag-start-fn+2/test]
#[test]
fn markup_escapes_every_value_it_renders() {
    let mut tag = SpanTag::new("a<b&c\"d", &["x>y"]);
    tag.add_attribute("lemma", "x>y & \"z\"");

    assert_eq!(
        tag.start_tag(),
        "<span id=\"a&lt;b&amp;c&quot;d\" class=\"x&gt;y\" \
         lemma=\"x&gt;y &amp; &quot;z&quot;\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.get-span-tag-start-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-span-tag-start-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-span-tag-start-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.get-span-tag-start-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.get-span-tag-start-fn+2/test]
#[test]
fn the_start_tag_is_rebuilt_on_every_call() {
    let mut tag = SpanTag::new("x", &[TOKEN_CLASS]);

    assert_eq!(tag.start_tag(), "<span id=\"x\" class=\"teaksta-token\">");

    tag.add_attribute("lemma", "beana");

    assert_eq!(
        tag.start_tag(),
        "<span id=\"x\" class=\"teaksta-token\" lemma=\"beana\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.get-span-tag-end-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-span-tag-end-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-span-tag-end-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.get-span-tag-end-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.get-span-tag-end-fn+2/test]
#[test]
fn the_end_tag_never_varies() {
    let mut tag = SpanTag::new("x", &[TOKEN_CLASS]);
    tag.add_attribute("lemma", "beana");

    assert_eq!(tag.end_tag(), "</span>");
    assert_eq!(SpanTag::new("", &[]).end_tag(), "</span>");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.to-string-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.to-string-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.to-string-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.to-string-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.span-tag.to-string-fn+2/test]
#[test]
fn a_span_tag_renders_as_its_start_tag() {
    let mut tag = SpanTag::new("x", &[TOKEN_CLASS]);
    tag.add_attribute("lemma", "beana");

    assert_eq!(
        tag.to_string(),
        "<span id=\"x\" class=\"teaksta-token\" lemma=\"beana\">"
    );
}
