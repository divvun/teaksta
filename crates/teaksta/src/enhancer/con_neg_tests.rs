use super::*;
use crate::types::{CgToken, EnhancementId};

/// The opening tag a connegative token carries once `process` has built it
/// and spliced the lemma in.
const SPAN_START: &str = "<span id=\"WERTi-span-boahtit-V-Ind-Prs-ConNeg-1\" \
     class=\"wertiviewtoken  wertiviewConNeg\"lemma=\"boahtit\">";

fn tags(tags: &[&str]) -> Vec<String> {
    tags.iter().map(|t| t.to_string()).collect()
}

fn conneg_token(begin: usize, end: usize) -> CgToken {
    CgToken {
        begin,
        end,
        readings: vec![
            tags(&["\"boahtit\"", "<sme>", "N", "Sg", "Nom"]),
            tags(&[
                "\"boahtit\"",
                "<sme>",
                "V",
                "Ind",
                "Prs",
                "ConNeg",
                "@+FMAINV",
            ]),
            tags(&["\"mannat\"", "<sme>", "V", "Ind", "Prt", "ConNeg"]),
        ],
    }
}

fn word_to_span_map(
    enhancer: &Vislcg3ConNegEnhancer,
    begin: usize,
    end: usize,
) -> HashMap<Word, SpanTag> {
    let mut map = HashMap::new();
    map.insert(
        Word::new(enhancer.outer_id(), begin, end),
        SpanTag::new(enhancer.outer_id(), SPAN_START.to_string()),
    );
    map
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.initialize-fn/test]
#[test]
fn initialize_splits_conneg_tags_on_comma_without_trimming() {
    let mut enhancer = Vislcg3ConNegEnhancer::default();
    assert_eq!(enhancer.conneg_tags, None);

    enhancer
        .initialize(Some("Ind Prs ConNeg, Ind Prt ConNeg"))
        .expect("descriptor default");

    assert_eq!(
        enhancer.conneg_tags,
        Some(vec![
            "Ind Prs ConNeg".to_string(),
            " Ind Prt ConNeg".to_string(),
        ])
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.initialize-fn/test]
#[test]
fn initialize_without_the_parameter_fails() {
    let mut enhancer = Vislcg3ConNegEnhancer::default();

    let err = enhancer.initialize(None).expect_err("missing parameter");

    assert!(err.to_string().contains("connegTags"));
    assert_eq!(enhancer.conneg_tags, None);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.process-fn/test]
#[test]
fn process_leaves_a_cancelled_document_untouched() {
    let enhancer = Vislcg3ConNegEnhancer::default();
    let mut doc = Document::new("in boahtán deike", "sme");
    doc.cg_tokens = vec![conneg_token(3, 10)];
    doc.enhancement_ids = vec![EnhancementId {
        enh_id: -1,
        ..EnhancementId::default()
    }];

    enhancer.process(&mut doc).expect("process");

    assert!(doc.enhancements.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.process-fn/test]
#[test]
fn process_wraps_each_conneg_token_in_numbered_span() {
    let enhancer = Vislcg3ConNegEnhancer::default();
    let mut doc = Document::new("in boahtán, in boahtán", "sme");
    doc.cg_tokens = vec![conneg_token(3, 10), conneg_token(15, 22)];

    enhancer.process(&mut doc).expect("process");

    assert_eq!(doc.enhancements.len(), 2);

    let first = &doc.enhancements[0];
    assert!(first.relevant);
    assert_eq!((first.begin, first.end), (3, 10));
    assert_eq!(
        first.enhance_start,
        "<span id=\"WERTi-span-boahtit-xsmey-V-Ind-Prs-ConNeg-@-FMAINV-1\" \
         class=\"wertiviewtoken  wertiviewConNeg\"lemma=\"boahtit\">"
    );
    assert_eq!(first.enhance_end, "</span>");

    let second = &doc.enhancements[1];
    assert_eq!((second.begin, second.end), (15, 22));
    assert_eq!(
        second.enhance_start,
        "<span id=\"WERTi-span-boahtit-xsmey-V-Ind-Prs-ConNeg-@-FMAINV-2\" \
         class=\"wertiviewtoken  wertiviewConNeg\"lemma=\"boahtit\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.remove-tags-fn/test]
#[test]
fn remove_tags_leaves_the_orth_error_residue() {
    let enhancer = Vislcg3ConNegEnhancer::default();

    assert_eq!(
        enhancer.remove_tags("boahtit+Err/Orth-a-á+V+Ind+Prs+ConNeg"),
        "boahtit-a-á+V+Ind+Prs+ConNeg"
    );
    assert_eq!(
        enhancer.remove_tags("boahtit+Err/Orth-nom-gen+N+Sg+Gen"),
        "boahtit-nom-gen+N+Sg+Gen"
    );
    assert_eq!(
        enhancer.remove_tags("boahtit+Allegro+V+Ind+Prs+ConNeg"),
        "boahtit+V+Ind+Prs+ConNeg"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.remove-tags-fn/test]
#[test]
fn remove_tags_drops_only_first_distinct_bracket_tag() {
    let enhancer = Vislcg3ConNegEnhancer::default();

    assert_eq!(
        enhancer.remove_tags("boahtit+<sme>+V+<foo_bar>+Ind"),
        "boahtit+V+<foo_bar>+Ind"
    );
    assert_eq!(
        enhancer.remove_tags("boahtit+<sme>+V+Ind\nmannat+<sme>+V+Ind\n"),
        "boahtit+V+Ind\nmannat+V+Ind\n"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.write-morphological-forms-fn/test]
#[test]
fn morphological_forms_emit_six_distractors_and_answer() {
    let enhancer = Vislcg3ConNegEnhancer::default();

    let block = enhancer
        .write_morphological_forms("boahtit+V+Ind+Prs+ConNeg+@+FMAINV")
        .expect("generation input");

    assert_eq!(
        block,
        "boahtit+V+Ind+Prs+Sg1\n\
         boahtit+V+Ind+Prs+Sg2\n\
         boahtit+V+Ind+Prs+Sg3\n\
         boahtit+V+Ind+Prt+Sg1\n\
         boahtit+V+Ind+Prt+Sg2\n\
         boahtit+V+Ind+Prt+Sg3\n\
         boahtit+V+Ind+Prs+ConNeg\n"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.write-morphological-forms-fn/test]
#[test]
fn morphological_forms_keep_reading_without_syntax_tag() {
    let enhancer = Vislcg3ConNegEnhancer::default();

    let block = enhancer
        .write_morphological_forms("boahtit+V+Ind+Prs+ConNeg")
        .expect("generation input");

    assert!(block.ends_with("boahtit+V+Ind+Prt+Sg3\nboahtit+V+Ind+Prs+ConNeg\n"));

    let err = enhancer
        .write_morphological_forms("boahtit")
        .expect_err("no separator to take the lemma from");
    assert!(err.to_string().contains("end -1"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.write-lemma-and-analyses-fn/test]
#[test]
fn write_lemma_and_analyses_drops_the_language_tag() {
    let enhancer = Vislcg3ConNegEnhancer::default();

    let line = enhancer
        .write_lemma_and_analyses("boahtit+<sme>+V+Ind+Prs+ConNeg+@+FMAINV")
        .expect("cloze input");

    assert_eq!(line, "boahtit+V+Ind+Prs+ConNeg\n");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.write-lemma-and-analyses-fn/test]
#[test]
fn write_lemma_and_analyses_needs_a_separator() {
    let enhancer = Vislcg3ConNegEnhancer::default();

    let err = enhancer
        .write_lemma_and_analyses("boahtit")
        .expect_err("no separator to split the lemma at");

    assert!(err.to_string().contains("end -1"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-distractors-fn/test]
#[test]
fn span_tag_distractors_splice_forms_and_answer() {
    let enhancer = Vislcg3ConNegEnhancer::default();
    let mut map = word_to_span_map(&enhancer, 3, 10);
    let mut doc = Document::new("in boahtán deike", "sme");

    let generator_output = concat!(
        "boahtit+V+Ind+Prs+Sg1\tboađán\n",
        "boahtit+V+Ind+Prs+Sg2\tboađát\n",
        "boahtit+V+Ind+Prs+Sg3\tboahtá\n",
        "boahtit+V+Ind+Prt+Sg1\t+?\n",
        "boahtit+V+Ind+Prt+Sg2\tboahtá\n",
        "boahtit+V+Ind+Prt+Sg3\tbođii\n",
        "boahtit+V+Ind+Prs+ConNeg\tboađe\n",
        "\n",
        "ñôŃßĘńŠē\n",
        "Word 3 10\n",
    );

    enhancer
        .generate_span_tag_with_distractors(&mut doc, generator_output, &mut map)
        .expect("distractors");

    assert_eq!(doc.enhancements.len(), 1);
    let e = &doc.enhancements[0];
    assert!(e.relevant);
    assert_eq!((e.begin, e.end), (3, 10));
    assert_eq!(e.enhance_end, "</span>");
    assert_eq!(
        e.enhance_start,
        "<span id=\"WERTi-span-boahtit-V-Ind-Prs-ConNeg-1\" \
         class=\"wertiviewtoken  wertiviewConNeg\"lemma=\"boahtit\"\
         distractors=\"boađán boađát boahtá bođii boađe\"answer=\"boađe\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-distractors-fn/test]
#[test]
fn span_tag_distractors_skip_single_form_token() {
    let enhancer = Vislcg3ConNegEnhancer::default();
    let mut map = word_to_span_map(&enhancer, 3, 10);
    let mut doc = Document::new("in boahtán deike", "sme");

    let generator_output = concat!(
        "boahtit+V+Ind+Prs+Sg1\t+?\n",
        "boahtit+V+Ind+Prs+Sg2\t+?\n",
        "boahtit+V+Ind+Prs+ConNeg\tboađe\n",
        "ñôŃßĘńŠē\n",
        "Word 3 10\n",
    );

    enhancer
        .generate_span_tag_with_distractors(&mut doc, generator_output, &mut map)
        .expect("distractors");

    assert!(doc.enhancements.is_empty());
    let span_tag = &map[&Word::new(enhancer.outer_id(), 3, 10)];
    assert_eq!(span_tag.get_span_tag_start(), SPAN_START);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-distractors-fn/test]
#[test]
fn span_tag_distractors_fail_on_unmapped_word() {
    let enhancer = Vislcg3ConNegEnhancer::default();
    let mut map = word_to_span_map(&enhancer, 3, 10);
    let mut doc = Document::new("in boahtán deike", "sme");

    let generator_output = concat!(
        "boahtit+V+Ind+Prs+Sg1\tboađán\n",
        "boahtit+V+Ind+Prs+Sg2\tboađát\n",
        "ñôŃßĘńŠē\n",
        "Word 40 47\n",
    );

    let err = enhancer
        .generate_span_tag_with_distractors(&mut doc, generator_output, &mut map)
        .expect_err("offsets that were never stored");

    assert!(err.to_string().contains("no span tag for Word 40 47"));
    assert!(doc.enhancements.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-possible-forms-fn/test]
#[test]
fn span_tag_possible_forms_accept_single_form() {
    let enhancer = Vislcg3ConNegEnhancer::default();
    let mut map = word_to_span_map(&enhancer, 3, 10);
    let mut doc = Document::new("in boahtán deike", "sme");

    let generator_output = concat!(
        "boahtit+V+Ind+Prs+ConNeg\tboađe\n",
        "boahtit+V+Ind+Prs+ConNeg\tboađe\n",
        "boahtit+V+Ind+Prs+ConNeg+Err/Orth\t+?\n",
        "ñôŃßĘńŠē\n",
        "Word 3 10\n",
    );

    enhancer
        .generate_span_tag_with_possible_forms(&mut doc, generator_output, &mut map)
        .expect("possible forms");

    assert_eq!(doc.enhancements.len(), 1);
    let e = &doc.enhancements[0];
    assert!(e.relevant);
    assert_eq!((e.begin, e.end), (3, 10));
    assert_eq!(e.enhance_end, "</span>");
    assert_eq!(
        e.enhance_start,
        "<span id=\"WERTi-span-boahtit-V-Ind-Prs-ConNeg-1\" \
         class=\"wertiviewtoken  wertiviewConNeg\"lemma=\"boahtit\"\
         possibleforms=\"boađe\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.mutable-int.get-fn/test]
#[test]
fn mutable_int_reads_back_the_counter() {
    assert_eq!(MutableInt::default().get(), 1);
    assert_eq!(MutableInt { value: 7 }.get(), 7);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.mutable-int.increment-fn/test]
#[test]
fn mutable_int_increment_counts_up_in_place() {
    let mut counter = MutableInt::default();

    counter.increment();
    assert_eq!(counter.value, 2);

    counter.increment();
    assert_eq!(counter.value, 3);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.word-fn/test]
#[test]
fn word_stores_the_offset_pair_verbatim() {
    let word = Word::new(7, 12, 5);

    assert_eq!(word.outer, 7);
    assert_eq!(word.begin, 12);
    assert_eq!(word.end, 5);

    let empty = Word::empty(7);
    assert_eq!((empty.begin, empty.end), (0, 0));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.get-begin-fn/test]
#[test]
fn word_get_begin_returns_the_stored_offset() {
    let word = Word {
        outer: 7,
        begin: 3,
        end: 10,
    };

    assert_eq!(word.get_begin(), 3);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.set-begin-fn/test]
#[test]
fn word_set_begin_overwrites_the_offset() {
    let mut word = Word::new(7, 3, 10);

    word.set_begin(4);

    assert_eq!(word.begin, 4);
    assert_eq!(word.end, 10);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.get-end-fn/test]
#[test]
fn word_get_end_returns_the_stored_offset() {
    let word = Word {
        outer: 7,
        begin: 3,
        end: 10,
    };

    assert_eq!(word.get_end(), 10);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.set-end-fn/test]
#[test]
fn word_set_end_overwrites_the_offset() {
    let mut word = Word::new(7, 3, 10);

    word.set_end(11);

    assert_eq!(word.begin, 3);
    assert_eq!(word.end, 11);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.hash-code-fn/test]
#[test]
fn word_hash_code_folds_in_the_enclosing_instance() {
    assert_eq!(Word::new(7, 3, 10).hash_code(), 36621);
    assert_ne!(Word::new(8, 3, 10).hash_code(), 36621);
    assert_eq!(
        Word::new(7, 3, 10).hash_code(),
        Word::new(7, 3, 10).hash_code()
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.equals-fn/test]
#[test]
fn word_equality_covers_offsets_and_enclosing_instance() {
    let word = Word::new(7, 3, 10);

    assert!(word.equals(&Word::new(7, 3, 10)));
    assert!(!word.equals(&Word::new(8, 3, 10)));
    assert!(!word.equals(&Word::new(7, 4, 10)));
    assert!(!word.equals(&Word::new(7, 3, 11)));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.get-outer-type-fn/test]
#[test]
fn word_get_outer_type_returns_its_enhancer() {
    let enhancer = Vislcg3ConNegEnhancer::default();
    let other = Vislcg3ConNegEnhancer::default();
    let word = Word::new(enhancer.outer_id(), 3, 10);

    assert_eq!(word.get_outer_type(), enhancer.outer_id());
    assert_ne!(word.get_outer_type(), other.outer_id());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.word.to-string-fn/test]
#[test]
fn word_renders_as_a_generator_input_record() {
    assert_eq!(Word::new(7, 3, 10).to_string(), "Word 3 10\n");
    assert_eq!(Word::empty(7).to_string(), "Word 0 0\n");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.span-tag-fn/test]
#[test]
fn span_tag_stores_opening_tag_and_hardcodes_closing() {
    let span_tag = SpanTag::new(7, "<span id=\"x\">".to_string());

    assert_eq!(span_tag.outer, 7);
    assert_eq!(span_tag.span_tag_start, "<span id=\"x\">");
    assert_eq!(span_tag.span_tag_end, "</span>");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.get-span-tag-start-fn/test]
#[test]
fn span_tag_start_includes_every_added_attribute() {
    let mut span_tag = SpanTag::new(7, "<span id=\"x\">".to_string());

    assert_eq!(span_tag.get_span_tag_start(), "<span id=\"x\">");

    span_tag.add_attribute("lemma", "boahtit");

    assert_eq!(
        span_tag.get_span_tag_start(),
        "<span id=\"x\"lemma=\"boahtit\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.set-span-tag-start-fn/test]
#[test]
fn set_span_tag_start_discards_added_attributes() {
    let mut span_tag = SpanTag::new(7, "<span>".to_string());
    span_tag.add_attribute("lemma", "boahtit");

    span_tag.set_span_tag_start("<span id=\"x\">".to_string());

    assert_eq!(span_tag.span_tag_start, "<span id=\"x\">");
    assert_eq!(span_tag.span_tag_end, "</span>");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.add-attribute-fn/test]
#[test]
fn add_attribute_splices_in_without_a_separating_space() {
    let mut span_tag = SpanTag::new(
        7,
        "<span id=\"X\" class=\"wertiviewtoken  wertiviewConNeg\">".to_string(),
    );

    span_tag.add_attribute("lemma", "boahtit");

    assert_eq!(
        span_tag.get_span_tag_start(),
        "<span id=\"X\" class=\"wertiviewtoken  wertiviewConNeg\"lemma=\"boahtit\">"
    );

    span_tag.add_attribute("answer", "boađe");

    assert_eq!(
        span_tag.get_span_tag_start(),
        "<span id=\"X\" class=\"wertiviewtoken  wertiviewConNeg\"\
         lemma=\"boahtit\"answer=\"boađe\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.add-attribute-fn/test]
#[test]
fn add_attribute_rewrites_every_closing_angle_bracket() {
    let mut span_tag = SpanTag::new(7, "<span id=\"x\">".to_string());
    span_tag.add_attribute("answer", "a>b");
    assert_eq!(
        span_tag.get_span_tag_start(),
        "<span id=\"x\"answer=\"a>b\">"
    );

    span_tag.add_attribute("lemma", "boahtit");

    assert_eq!(
        span_tag.get_span_tag_start(),
        "<span id=\"x\"answer=\"alemma=\"boahtit\">b\"lemma=\"boahtit\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.get-span-tag-end-fn/test]
#[test]
fn get_span_tag_end_returns_the_closing_tag() {
    let mut span_tag = SpanTag::new(7, "<span>".to_string());

    assert_eq!(span_tag.get_span_tag_end(), "</span>");

    span_tag.add_attribute("lemma", "boahtit");
    assert_eq!(span_tag.get_span_tag_end(), "</span>");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.set-span-tag-end-fn/test]
#[test]
fn set_span_tag_end_overwrites_the_closing_tag() {
    let mut span_tag = SpanTag::new(7, "<span>".to_string());

    span_tag.set_span_tag_end("</div>".to_string());

    assert_eq!(span_tag.span_tag_end, "</div>");
    assert_eq!(span_tag.span_tag_start, "<span>");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.hash-code-fn/test]
#[test]
fn span_tag_hash_code_folds_in_enclosing_instance() {
    let span_tag = SpanTag::new(7, "<span>".to_string());

    assert_eq!(span_tag.hash_code(), 1183645597);
    assert_ne!(
        SpanTag::new(8, "<span>".to_string()).hash_code(),
        1183645597
    );

    let mut mutated = SpanTag::new(7, "<span>".to_string());
    mutated.set_span_tag_end("</div>".to_string());
    assert_ne!(mutated.hash_code(), span_tag.hash_code());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.equals-fn/test]
#[test]
fn span_tag_equality_covers_fields_and_enclosing_instance() {
    let span_tag = SpanTag::new(7, "<span>".to_string());

    assert!(span_tag.equals(&SpanTag::new(7, "<span>".to_string())));
    assert!(!span_tag.equals(&SpanTag::new(8, "<span>".to_string())));
    assert!(!span_tag.equals(&SpanTag::new(7, "<span id=\"x\">".to_string())));

    let mut other_end = SpanTag::new(7, "<span>".to_string());
    other_end.set_span_tag_end("</div>".to_string());
    assert!(!span_tag.equals(&other_end));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.get-outer-type-fn/test]
#[test]
fn span_tag_get_outer_type_returns_its_enhancer() {
    let enhancer = Vislcg3ConNegEnhancer::default();
    let other = Vislcg3ConNegEnhancer::default();
    let span_tag = SpanTag::new(enhancer.outer_id(), "<span>".to_string());

    assert_eq!(span_tag.get_outer_type(), enhancer.outer_id());
    assert_ne!(span_tag.get_outer_type(), other.outer_id());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.span-tag.to-string-fn/test]
#[test]
fn span_tag_renders_both_fields_for_the_log() {
    let span_tag = SpanTag::new(7, "<span id=\"x\">".to_string());

    assert_eq!(
        span_tag.to_string(),
        "SpanTag [spanTagStart=<span id=\"x\">, spanTagEnd=</span>]"
    );
}
