//! What the connegative topic does that its siblings do not: the
//! connegative pattern and the two generator inputs. The pass they all share
//! is exercised in `cg_enhancer_tests`.

use super::*;
use crate::types::{CgToken, EnhancementId};

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

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.process-fn+4/test]
#[test]
fn process_leaves_a_cancelled_document_untouched() {
    let enhancer = Vislcg3ConNegEnhancer::default();
    let mut doc = Document::new("in boahtán deike", "sme");
    doc.cg_tokens = vec![conneg_token(3, 10)];
    doc.enhancement_ids = vec![EnhancementId {
        enh_id: -1,
        ..EnhancementId::default()
    }];

    enhancer.process(&mut doc, Mode::Colorize).expect("process");

    assert!(doc.enhancements.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.process-fn+4/test]
#[test]
fn process_wraps_each_conneg_token_in_numbered_span() {
    let enhancer = Vislcg3ConNegEnhancer::default();
    let mut doc = Document::new("in boahtán, in boahtán", "sme");
    doc.cg_tokens = vec![conneg_token(3, 10), conneg_token(15, 22)];

    enhancer.process(&mut doc, Mode::Colorize).expect("process");

    assert_eq!(doc.enhancements.len(), 2);

    let first = &doc.enhancements[0];
    assert!(first.relevant);
    assert_eq!((first.begin, first.end), (3, 10));
    assert_eq!(
        first.enhance_start,
        "<span id=\"teaksta-span-boahtit-xsmey-V-Ind-Prs-ConNeg-@-FMAINV-1\" \
         class=\"teaksta-token teaksta-ConNeg\" lemma=\"boahtit\">"
    );
    assert_eq!(first.enhance_end, "</span>");

    let second = &doc.enhancements[1];
    assert_eq!((second.begin, second.end), (15, 22));
    assert_eq!(
        second.enhance_start,
        "<span id=\"teaksta-span-boahtit-xsmey-V-Ind-Prs-ConNeg-@-FMAINV-2\" \
         class=\"teaksta-token teaksta-ConNeg\" lemma=\"boahtit\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.write-morphological-forms-fn+2/test]
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

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.write-morphological-forms-fn+2/test]
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

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.write-lemma-and-analyses-fn+2/test]
#[test]
fn write_lemma_and_analyses_drops_the_language_tag() {
    let enhancer = Vislcg3ConNegEnhancer::default();

    let line = enhancer
        .write_lemma_and_analyses("boahtit+<sme>+V+Ind+Prs+ConNeg+@+FMAINV")
        .expect("cloze input");

    assert_eq!(line, "boahtit+V+Ind+Prs+ConNeg\n");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.write-lemma-and-analyses-fn+2/test]
#[test]
fn write_lemma_and_analyses_needs_a_separator() {
    let enhancer = Vislcg3ConNegEnhancer::default();

    let err = enhancer
        .write_lemma_and_analyses("boahtit")
        .expect_err("no separator to split the lemma at");

    assert!(err.to_string().contains("end -1"));
}
