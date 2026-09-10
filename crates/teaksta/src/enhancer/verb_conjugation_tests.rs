//! What the finite-verb topic does that its siblings do not: the person
//! pattern and the two generator inputs. The pass they all share is
//! exercised in `cg_enhancer_tests`.

use super::*;
use crate::types::CgToken;
use crate::util::cas_utils;

fn reading(tags: &[&str]) -> Vec<String> {
    tags.iter().map(|t| t.to_string()).collect()
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.initialize-fn/test]
#[test]
fn initialize_splits_the_parameter_on_commas_without_trimming() {
    let mut enhancer = Vislcg3VerbConjugationEnhancer::default();
    enhancer.initialize(Some("Ind Prs, Ind Prt")).unwrap();

    assert_eq!(
        enhancer.fin_verb_tags.unwrap(),
        vec!["Ind Prs".to_string(), " Ind Prt".to_string()]
    );

    let mut single = Vislcg3VerbConjugationEnhancer::default();
    single.initialize(Some("Ind Prs")).unwrap();
    assert_eq!(single.fin_verb_tags.unwrap(), vec!["Ind Prs".to_string()]);

    let err = Vislcg3VerbConjugationEnhancer::default()
        .initialize(None)
        .unwrap_err();
    assert!(err.to_string().contains("finverbTags"), "{err}");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.process-fn+3/test]
#[test]
fn process_wraps_finite_verbs_in_numbered_spans() {
    let enhancer = Vislcg3VerbConjugationEnhancer::default();
    let mut doc = Document::new("Mun boadán ruoktot. Mun boadán.", "sme");
    doc.cg_tokens = vec![
        CgToken {
            begin: 4,
            end: 10,
            readings: vec![
                reading(&["\"boahtit\"", "N", "Sg", "Nom"]),
                reading(&["\"boahtit\"", "V", "IV", "Ind", "Prs", "Sg1"]),
                reading(&["\"boahtit\"", "V", "IV", "Ind", "Prt", "Sg3"]),
            ],
        },
        CgToken {
            begin: 11,
            end: 18,
            readings: vec![reading(&["\"ruoktot\"", "Adv"])],
        },
        CgToken {
            begin: 24,
            end: 30,
            readings: vec![reading(&["\"boahtit\"", "V", "IV", "Ind", "Prs", "Sg1"])],
        },
    ];

    enhancer.process(&mut doc).unwrap();

    assert_eq!(doc.enhancements.len(), 2);

    let first = &doc.enhancements[0];
    assert!(first.relevant);
    assert_eq!((first.begin, first.end), (4, 10));
    assert_eq!(
        first.enhance_start,
        "<span id=\"WERTi-span-boahtit-V-IV-Ind-Prs-Sg1-1\" \
         class=\"teaksta-token teaksta-VerbConjugation\" lemma=\"boahtit\">"
    );
    assert_eq!(first.enhance_end, "</span>");

    // The second occurrence of the same reading takes the next counter
    // value; the adverb between them is not enhanced at all.
    let second = &doc.enhancements[1];
    assert_eq!((second.begin, second.end), (24, 30));
    assert_eq!(
        second.enhance_start,
        "<span id=\"WERTi-span-boahtit-V-IV-Ind-Prs-Sg1-2\" \
         class=\"teaksta-token teaksta-VerbConjugation\" lemma=\"boahtit\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.process-fn+3/test]
#[test]
fn process_skips_readings_missing_verb_or_person_tag() {
    let enhancer = Vislcg3VerbConjugationEnhancer::default();
    let mut doc = Document::new("Mun in boade.", "sme");
    doc.cg_tokens = vec![
        CgToken {
            begin: 0,
            end: 3,
            readings: vec![reading(&["\"mun\"", "Pron", "Pers", "Sg1", "Nom"])],
        },
        CgToken {
            begin: 7,
            end: 12,
            readings: vec![reading(&["\"boahtit\"", "V", "IV", "Inf"])],
        },
    ];

    enhancer.process(&mut doc).unwrap();
    assert!(doc.enhancements.is_empty());

    // An invalidated CAS is left untouched even when a reading matches.
    let mut cancelled = Document::new("Mun boadán.", "sme");
    cancelled.cg_tokens = vec![CgToken {
        begin: 4,
        end: 10,
        readings: vec![reading(&["\"boahtit\"", "V", "IV", "Ind", "Prs", "Sg1"])],
    }];
    cas_utils::add_enh_id(&mut cancelled, -1);

    enhancer.process(&mut cancelled).unwrap();
    assert!(cancelled.enhancements.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-morphological-forms-fn+2/test]
#[test]
fn write_morphological_forms_emits_indicative_then_answer() {
    let enhancer = Vislcg3VerbConjugationEnhancer::default();

    let block = enhancer
        .write_morphological_forms("boahtit+V+IV+Ind+Prs+Sg1")
        .unwrap();

    assert_eq!(
        block,
        "boahtit+V+Ind+Prs+ConNeg\n\
         boahtit+V+Ind+Prt+ConNeg\n\
         boahtit+V+Actio+Ess\n\
         boahtit+V+Ind+Prt+Sg1\n\
         boahtit+V+Ind+Prt+Sg2\n\
         boahtit+V+Ind+Prt+Sg3\n\
         boahtit+V+Ind+Prs+Sg1\n\
         boahtit+V+Ind+Prs+Sg2\n\
         boahtit+V+Ind+Prs+Sg3\n\
         boahtit+V+Ind+Prt+Du1\n\
         boahtit+V+Ind+Prt+Du2\n\
         boahtit+V+Ind+Prt+Du3\n\
         boahtit+V+Ind+Prs+Du1\n\
         boahtit+V+Ind+Prs+Du2\n\
         boahtit+V+Ind+Prs+Du3\n\
         boahtit+V+IV+Ind+Prs+Sg1\n"
    );

    // The indicative table has no plural rows at all.
    assert!(!block.contains("Pl1"));
    assert!(!block.contains("Pl2"));
    assert!(!block.contains("Pl3"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-morphological-forms-fn+2/test]
#[test]
fn write_morphological_forms_selects_the_table_by_mood() {
    let enhancer = Vislcg3VerbConjugationEnhancer::default();

    let imprt = enhancer
        .write_morphological_forms("boahtit+V+IV+Imprt+Sg2")
        .unwrap();
    let imprt_lines: Vec<&str> = imprt.lines().collect();
    assert_eq!(imprt_lines.len(), 10);
    assert_eq!(imprt_lines[0], "boahtit+V+Imprt+Sg1");
    assert_eq!(imprt_lines[8], "boahtit+V+Imprt+Pl3");
    assert_eq!(imprt_lines[9], "boahtit+V+IV+Imprt+Sg2");

    let cond = enhancer
        .write_morphological_forms("boahtit+V+IV+Cond+Prs+Sg1")
        .unwrap();
    let cond_lines: Vec<&str> = cond.lines().collect();
    assert_eq!(cond_lines.len(), 10);
    assert_eq!(cond_lines[0], "boahtit+V+Cond+Prs+Sg1");
    assert_eq!(cond_lines[8], "boahtit+V+Cond+Prs+Pl3");

    let pot = enhancer
        .write_morphological_forms("boahtit+V+IV+Pot+Prs+Sg3")
        .unwrap();
    let pot_lines: Vec<&str> = pot.lines().collect();
    assert_eq!(pot_lines.len(), 10);
    assert_eq!(pot_lines[0], "boahtit+V+Pot+Prs+Sg1");

    let neg = enhancer
        .write_morphological_forms("ii+V+IV+Neg+Ind+Sg1")
        .unwrap();
    let neg_lines: Vec<&str> = neg.lines().collect();
    assert_eq!(neg_lines.len(), 10);
    assert_eq!(neg_lines[0], "ii+V+Neg+Ind+Sg1");
    assert_eq!(neg_lines[9], "ii+V+IV+Neg+Ind+Sg1");

    // An unrecognised mood fires no table, so only the answer line is left
    // and the token can never reach the two-distractor minimum.
    let unknown = enhancer
        .write_morphological_forms("boahtit+V+IV+Xyz+Prs+Sg3")
        .unwrap();
    assert_eq!(unknown, "boahtit+V+IV+Xyz+Prs+Sg3\n");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-morphological-forms-fn+2/test]
#[test]
fn write_morphological_forms_trims_answer_at_syntactic_tag() {
    let enhancer = Vislcg3VerbConjugationEnhancer::default();

    let with_at = enhancer
        .write_morphological_forms("boahtit+V+IV+Ind+Prs+Sg1+@+FMAINV")
        .unwrap();
    let lines: Vec<&str> = with_at.lines().collect();
    assert_eq!(lines.len(), 16);
    assert_eq!(lines[15], "boahtit+V+IV+Ind+Prs+Sg1");

    // A leading "@" is not treated as a syntactic tag.
    let at_first = enhancer.write_morphological_forms("@+V+IV+Ind").unwrap();
    assert_eq!(at_first.lines().last().unwrap(), "@+V+IV+Ind");

    // The whole block, answer line included, goes through remove_tags.
    let tagged = enhancer
        .write_morphological_forms("boahtit+V+IV+Ind+Prs+Sg1+Err/Spellrelax")
        .unwrap();
    assert_eq!(tagged.lines().last().unwrap(), "boahtit+V+IV+Ind+Prs+Sg1");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-morphological-forms-fn+2/test]
#[test]
fn write_morphological_forms_rejects_null_mood_and_overflow() {
    let enhancer = Vislcg3VerbConjugationEnhancer::default();

    let too_short = enhancer
        .write_morphological_forms("boahtit+V+IV")
        .unwrap_err();
    assert!(
        too_short.to_string().contains("mood is null"),
        "{too_short}"
    );

    let long: Vec<String> = (0..21).map(|i| format!("t{i}")).collect();
    let overflow = enhancer
        .write_morphological_forms(&long.join("+"))
        .unwrap_err();
    assert_eq!(overflow.to_string(), "Index 20 out of bounds for length 20");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-lemma-and-analyses-fn+2/test]
#[test]
fn write_lemma_and_analyses_drops_langtag_and_tail() {
    let enhancer = Vislcg3VerbConjugationEnhancer::default();

    assert_eq!(
        enhancer
            .write_lemma_and_analyses("boahtit+V+<sme>+Ind+Prs+Sg3")
            .unwrap(),
        "boahtit+V+Ind+Prs+Sg\n"
    );

    // A trailing "+<sme>" loses only its ">" to the length-1 bound, so the
    // literal removal no longer matches it.
    assert_eq!(
        enhancer
            .write_lemma_and_analyses("boahtit+V+Ind+Prs+Sg3+<sme>")
            .unwrap(),
        "boahtit+V+Ind+Prs+Sg3+<sme\n"
    );

    // The analyses go through remove_tags as well.
    assert_eq!(
        enhancer
            .write_lemma_and_analyses("boahtit+V+Allegro+Ind+Prs+Sg33")
            .unwrap(),
        "boahtit+V+Ind+Prs+Sg3\n"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-lemma-and-analyses-fn+2/test]
#[test]
fn write_lemma_and_analyses_truncates_or_rejects_input() {
    let enhancer = Vislcg3VerbConjugationEnhancer::default();

    assert_eq!(
        enhancer
            .write_lemma_and_analyses("boahtit+V+Ind+Prs+Sg3+@+FMAINV")
            .unwrap(),
        "boahtit+V+Ind+Prs+Sg3\n"
    );

    let no_plus = enhancer.write_lemma_and_analyses("boahtit").unwrap_err();
    assert_eq!(no_plus.to_string(), "begin 0, end -1, length 7");

    let empty_tail = enhancer.write_lemma_and_analyses("a+").unwrap_err();
    assert_eq!(empty_tail.to_string(), "begin 2, end 1, length 2");
}
