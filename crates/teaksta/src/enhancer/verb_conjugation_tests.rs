use super::*;
use crate::types::CgToken;

const SPAN_START: &str =
    "<span id=\"WERTi-span-1\" class=\"wertiviewtoken  wertiviewVerbConjugation\">";

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

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.process-fn/test]
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
         class=\"wertiviewtoken  wertiviewVerbConjugation\"lemma=\"boahtit\">"
    );
    assert_eq!(first.enhance_end, "</span>");

    // The second occurrence of the same reading takes the next counter
    // value; the adverb between them is not enhanced at all.
    let second = &doc.enhancements[1];
    assert_eq!((second.begin, second.end), (24, 30));
    assert_eq!(
        second.enhance_start,
        "<span id=\"WERTi-span-boahtit-V-IV-Ind-Prs-Sg1-2\" \
         class=\"wertiviewtoken  wertiviewVerbConjugation\"lemma=\"boahtit\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.process-fn/test]
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

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.remove-tags-fn/test]
#[test]
fn remove_tags_strips_literals_and_first_angle_tag() {
    let enhancer = Vislcg3VerbConjugationEnhancer::default();

    assert_eq!(
        enhancer.remove_tags("boahtit+V+Ind+Prs+Sg3+<sme>"),
        "boahtit+V+Ind+Prs+Sg3"
    );
    assert_eq!(
        enhancer.remove_tags("boahtit+V+Allegro+Ind+Prs+Sg3"),
        "boahtit+V+Ind+Prs+Sg3"
    );
    assert_eq!(
        enhancer.remove_tags("boahtit+V+Err/MissingHyph+Ind"),
        "boahtit+V+Ind"
    );

    // The first match's literal text is removed everywhere it occurs, but
    // a differently spelled angle tag survives.
    assert_eq!(enhancer.remove_tags("a+<sme>b+<sme>c"), "abc");
    assert_eq!(enhancer.remove_tags("a+<sme>b+<nob>c"), "ab+<nob>c");
    assert_eq!(enhancer.remove_tags("a+<a_b>c"), "ac");

    // Nothing to strip: the input is returned verbatim, untrimmed.
    assert_eq!(enhancer.remove_tags("  boahtit+V  "), "  boahtit+V  ");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.remove-tags-fn/test]
#[test]
fn remove_tags_leaves_shadowed_orthography_residue() {
    let enhancer = Vislcg3VerbConjugationEnhancer::default();

    assert_eq!(
        enhancer.remove_tags("boahtit+V+Err/Orth-a-á+Ind"),
        "boahtit+V-a-á+Ind"
    );
    assert_eq!(
        enhancer.remove_tags("boahtit+V+Err/Orth-nom-gen+Ind"),
        "boahtit+V-nom-gen+Ind"
    );
    assert_eq!(
        enhancer.remove_tags("boahtit+V+Err/Orth-nom-acc+Ind"),
        "boahtit+V-nom-acc+Ind"
    );
    assert_eq!(
        enhancer.remove_tags("boahtit+V+Err/Orth+Ind"),
        "boahtit+V+Ind"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-morphological-forms-fn/test]
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

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-morphological-forms-fn/test]
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

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-morphological-forms-fn/test]
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

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-morphological-forms-fn/test]
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

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-lemma-and-analyses-fn/test]
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

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-lemma-and-analyses-fn/test]
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

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-distractors-fn/test]
#[test]
fn span_tag_distractors_attach_forms_and_final_field() {
    let enhancer = Vislcg3VerbConjugationEnhancer::default();
    let word = Word::new(enhancer.outer_id(), 4, 10);
    let mut word_to_span_map = HashMap::new();
    word_to_span_map.insert(
        word.clone(),
        SpanTag::new(enhancer.outer_id(), SPAN_START.to_string()),
    );

    let generator_output = "boahtit+V+Ind+Prs+Sg1\tboadán\n\
                            boahtit+V+Ind+Prs+Sg2\tboaðát\n\
                            boahtit+V+Ind+Prs+Sg3\tboahtá\n\
                            boahtit+V+IV+Ind+Prs+Sg1\tboadán\n\
                            ñôŃßĘńŠē\n\
                            Word 4 10\n";

    let mut doc = Document::new("Mun boadán.", "sme");
    enhancer
        .generate_span_tag_with_distractors(&mut doc, generator_output, &mut word_to_span_map)
        .unwrap();

    let expected_start = "<span id=\"WERTi-span-1\" \
         class=\"wertiviewtoken  wertiviewVerbConjugation\"\
         distractors=\"boadán boaðát boahtá\"answer=\"boadán\">";

    assert_eq!(doc.enhancements.len(), 1);
    let e = &doc.enhancements[0];
    assert!(e.relevant);
    assert_eq!((e.begin, e.end), (4, 10));
    assert_eq!(e.enhance_start, expected_start);
    assert_eq!(e.enhance_end, "</span>");

    // The attributes are spliced into the stored span tag, not a copy.
    assert_eq!(word_to_span_map[&word].get_span_tag_start(), expected_start);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-distractors-fn/test]
#[test]
fn span_tag_distractors_reject_plus_and_hyphen_forms() {
    let enhancer = Vislcg3VerbConjugationEnhancer::default();
    let word = Word::new(enhancer.outer_id(), 0, 5);
    let mut word_to_span_map = HashMap::new();
    word_to_span_map.insert(
        word.clone(),
        SpanTag::new(enhancer.outer_id(), SPAN_START.to_string()),
    );

    let generator_output = "boahtit+V+Imprt+Sg1\t+?\n\
                            boahtit+V+Imprt+Sg2\tboaðe-dat\n\
                            boahtit+V+Imprt+Sg3\tbohtos\n\
                            boahtit+V+Imprt+Du1\tbohkku\n\
                            ñôŃßĘńŠē\n\
                            Word 0 5\n";

    let mut doc = Document::new("Boade!", "sme");
    enhancer
        .generate_span_tag_with_distractors(&mut doc, generator_output, &mut word_to_span_map)
        .unwrap();

    assert_eq!(doc.enhancements.len(), 1);
    assert_eq!(
        doc.enhancements[0].enhance_start,
        "<span id=\"WERTi-span-1\" \
         class=\"wertiviewtoken  wertiviewVerbConjugation\"\
         distractors=\"bohtos bohkku\"answer=\"bohkku\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-distractors-fn/test]
#[test]
fn span_tag_distractors_need_at_least_two_forms() {
    let enhancer = Vislcg3VerbConjugationEnhancer::default();
    let word = Word::new(enhancer.outer_id(), 4, 10);
    let mut word_to_span_map = HashMap::new();
    word_to_span_map.insert(
        word.clone(),
        SpanTag::new(enhancer.outer_id(), SPAN_START.to_string()),
    );

    // One surviving form after de-duplication is not enough for multiple
    // choice, so the Word line that follows is ignored entirely.
    let generator_output = "boahtit+V+Ind+Prs+Sg1\tboadán\n\
                            boahtit+V+IV+Ind+Prs+Sg1\tboadán\n\
                            ñôŃßĘńŠē\n\
                            Word 4 10\n";

    let mut doc = Document::new("Mun boadán.", "sme");
    enhancer
        .generate_span_tag_with_distractors(&mut doc, generator_output, &mut word_to_span_map)
        .unwrap();

    assert!(doc.enhancements.is_empty());
    assert_eq!(word_to_span_map[&word].get_span_tag_start(), SPAN_START);

    // A Word line seen before any marker is ignored for the same reason,
    // even when nothing in the map could have matched it.
    let mut empty_map: HashMap<Word, SpanTag> = HashMap::new();
    let mut doc = Document::new("Mun boadán.", "sme");
    enhancer
        .generate_span_tag_with_distractors(&mut doc, "Word 4 10\n", &mut empty_map)
        .unwrap();
    assert!(doc.enhancements.is_empty());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-distractors-fn/test]
#[test]
fn span_tag_distractors_fail_on_unmapped_or_short() {
    let enhancer = Vislcg3VerbConjugationEnhancer::default();
    let block = "boahtit+V+Ind+Prs+Sg2\tboaðát\n\
                 boahtit+V+Ind+Prs+Sg3\tboahtá\n\
                 ñôŃßĘńŠē\n";

    let mut empty_map: HashMap<Word, SpanTag> = HashMap::new();
    let mut doc = Document::new("Mun boadán.", "sme");
    let unmapped = enhancer
        .generate_span_tag_with_distractors(
            &mut doc,
            &format!("{block}Word 4 10\n"),
            &mut empty_map,
        )
        .unwrap_err();
    assert!(
        unmapped.to_string().contains("no span tag for Word 4 10"),
        "{unmapped}"
    );

    let mut word_to_span_map = HashMap::new();
    word_to_span_map.insert(
        Word::new(enhancer.outer_id(), 4, 10),
        SpanTag::new(enhancer.outer_id(), SPAN_START.to_string()),
    );
    let mut doc = Document::new("Mun boadán.", "sme");
    let short = enhancer
        .generate_span_tag_with_distractors(
            &mut doc,
            &format!("{block}Word 4\n"),
            &mut word_to_span_map,
        )
        .unwrap_err();
    assert_eq!(short.to_string(), "Index 2 out of bounds for length 2");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-possible-forms-fn/test]
#[test]
fn span_tag_possible_forms_enhance_each_block() {
    let enhancer = Vislcg3VerbConjugationEnhancer::default();
    let first = Word::new(enhancer.outer_id(), 4, 10);
    let second = Word::new(enhancer.outer_id(), 11, 16);
    let mut word_to_span_map = HashMap::new();
    word_to_span_map.insert(
        first.clone(),
        SpanTag::new(enhancer.outer_id(), SPAN_START.to_string()),
    );
    word_to_span_map.insert(
        second.clone(),
        SpanTag::new(enhancer.outer_id(), SPAN_START.to_string()),
    );

    let generator_output = "boahtit+V+Ind+Prs+Sg1\tboadán\n\
                            ñôŃßĘńŠē\n\
                            Word 4 10\n\
                            mannat+V+Ind+Prs+Sg3\tmanná\n\
                            mannat+V+Ind+Prt+Sg3\tmanai\n\
                            ñôŃßĘńŠē\n\
                            Word 11 16\n";

    let mut doc = Document::new("Mun boadán manná.", "sme");
    enhancer
        .generate_span_tag_with_possible_forms(&mut doc, generator_output, &mut word_to_span_map)
        .unwrap();

    assert_eq!(doc.enhancements.len(), 2);

    // A single generated form is enough here, and no answer attribute is
    // added.
    assert_eq!(
        (doc.enhancements[0].begin, doc.enhancements[0].end),
        (4, 10)
    );
    assert_eq!(
        doc.enhancements[0].enhance_start,
        "<span id=\"WERTi-span-1\" \
         class=\"wertiviewtoken  wertiviewVerbConjugation\"\
         possibleforms=\"boadán\">"
    );
    assert_eq!(doc.enhancements[0].enhance_end, "</span>");

    assert_eq!(
        (doc.enhancements[1].begin, doc.enhancements[1].end),
        (11, 16)
    );
    assert_eq!(
        doc.enhancements[1].enhance_start,
        "<span id=\"WERTi-span-1\" \
         class=\"wertiviewtoken  wertiviewVerbConjugation\"\
         possibleforms=\"manná manai\">"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-possible-forms-fn/test]
#[test]
fn span_tag_possible_forms_skip_ungenerated_and_unmapped() {
    let enhancer = Vislcg3VerbConjugationEnhancer::default();

    // Every form failed to generate, so no possible forms survive and the
    // Word line is ignored.
    let mut word_to_span_map = HashMap::new();
    word_to_span_map.insert(
        Word::new(enhancer.outer_id(), 4, 10),
        SpanTag::new(enhancer.outer_id(), SPAN_START.to_string()),
    );
    let mut doc = Document::new("Mun boadán.", "sme");
    enhancer
        .generate_span_tag_with_possible_forms(
            &mut doc,
            "boahtit+V+Ind+Prs+Sg1\t+?\nñôŃßĘńŠē\nWord 4 10\n",
            &mut word_to_span_map,
        )
        .unwrap();
    assert!(doc.enhancements.is_empty());

    let mut empty_map: HashMap<Word, SpanTag> = HashMap::new();
    let mut doc = Document::new("Mun boadán.", "sme");
    let unmapped = enhancer
        .generate_span_tag_with_possible_forms(
            &mut doc,
            "boahtit+V+Ind+Prs+Sg1\tboadán\nñôŃßĘńŠē\nWord 4 10\n",
            &mut empty_map,
        )
        .unwrap_err();
    assert!(
        unmapped.to_string().contains("no span tag for Word 4 10"),
        "{unmapped}"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.mutable-int.increment-fn/test]
#[test]
fn mutable_int_increment_counts_up_from_one() {
    let mut counter = MutableInt::default();
    assert_eq!(counter.value, 1);

    counter.increment();
    assert_eq!(counter.value, 2);

    counter.increment();
    assert_eq!(counter.value, 3);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.mutable-int.get-fn/test]
#[test]
fn mutable_int_get_returns_the_current_value() {
    assert_eq!(MutableInt::default().get(), 1);
    assert_eq!(MutableInt { value: 7 }.get(), 7);
    assert_eq!(MutableInt { value: -3 }.get(), -3);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.word-fn/test]
#[test]
fn word_constructor_stores_both_offsets_verbatim() {
    let word = Word::new(9, 42, 47);
    assert_eq!(word.begin, 42);
    assert_eq!(word.end, 47);

    // No validation: an end before the begin is stored as given.
    let inverted = Word::new(9, 47, 42);
    assert_eq!((inverted.begin, inverted.end), (47, 42));

    let empty = Word::empty(9);
    assert_eq!((empty.begin, empty.end), (0, 0));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.get-begin-fn/test]
#[test]
fn word_get_begin_returns_the_begin_offset() {
    assert_eq!(Word::new(9, 42, 47).get_begin(), 42);
    assert_eq!(Word::empty(9).get_begin(), 0);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.get-end-fn/test]
#[test]
fn word_get_end_returns_the_end_offset() {
    assert_eq!(Word::new(9, 42, 47).get_end(), 47);
    assert_eq!(Word::empty(9).get_end(), 0);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.set-begin-fn/test]
#[test]
fn word_set_begin_overwrites_offset_and_breaks_lookups() {
    let mut word = Word::new(9, 42, 47);
    word.set_begin(3);
    assert_eq!(word.get_begin(), 3);
    assert_eq!(word.get_end(), 47);

    let mut map = HashMap::new();
    map.insert(Word::new(9, 42, 47), "spantag");
    let mut key = Word::new(9, 42, 47);
    key.set_begin(3);
    assert!(map.get(&key).is_none());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.set-end-fn/test]
#[test]
fn word_set_end_overwrites_offset_and_breaks_lookups() {
    let mut word = Word::new(9, 42, 47);
    word.set_end(50);
    assert_eq!(word.get_end(), 50);
    assert_eq!(word.get_begin(), 42);

    let mut map = HashMap::new();
    map.insert(Word::new(9, 42, 47), "spantag");
    let mut key = Word::new(9, 42, 47);
    key.set_end(50);
    assert!(map.get(&key).is_none());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.to-string-fn/test]
#[test]
fn word_renders_generator_record_with_trailing_newline() {
    assert_eq!(Word::new(9, 42, 47).to_string(), "Word 42 47\n");
    assert_eq!(Word::empty(9).to_string(), "Word 0 0\n");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.hash-code-fn/test]
#[test]
fn word_hash_code_folds_outer_begin_then_end() {
    assert_eq!(Word::new(0, 2, 3).hash_code(), 29856);
    assert_eq!(Word::empty(0).hash_code(), 29791);

    // Swapping the offsets changes the hash, and so does the enclosing
    // instance.
    assert_ne!(
        Word::new(0, 2, 3).hash_code(),
        Word::new(0, 3, 2).hash_code()
    );
    assert_ne!(
        Word::new(0, 2, 3).hash_code(),
        Word::new(1, 2, 3).hash_code()
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.equals-fn/test]
#[test]
fn word_equality_covers_enclosing_instance_and_offsets() {
    assert!(Word::new(9, 42, 47).equals(&Word::new(9, 42, 47)));
    assert!(!Word::new(9, 42, 47).equals(&Word::new(8, 42, 47)));
    assert!(!Word::new(9, 42, 47).equals(&Word::new(9, 41, 47)));
    assert!(!Word::new(9, 42, 47).equals(&Word::new(9, 42, 48)));

    let word = Word::new(9, 42, 47);
    assert!(word.equals(&word));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.get-outer-type-fn/test]
#[test]
fn word_get_outer_type_returns_the_enclosing_instance() {
    let enhancer = Vislcg3VerbConjugationEnhancer::default();
    let word = Word::new(enhancer.outer_id(), 42, 47);
    assert_eq!(word.get_outer_type(), enhancer.outer_id());
    assert_ne!(Word::new(9, 42, 47).get_outer_type(), 8);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.span-tag-fn/test]
#[test]
fn span_tag_constructor_keeps_start_and_fixes_end() {
    let tag = SpanTag::new(9, SPAN_START.to_string());
    assert_eq!(tag.span_tag_start, SPAN_START);
    assert_eq!(tag.span_tag_end, "</span>");

    // No validation of the argument.
    let nonsense = SpanTag::new(9, "not markup".to_string());
    assert_eq!(nonsense.span_tag_start, "not markup");
    assert_eq!(nonsense.span_tag_end, "</span>");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.add-attribute-fn/test]
#[test]
fn add_attribute_splices_flush_against_every_closing_bracket() {
    let mut tag = SpanTag::new(
        9,
        "<span id=\"X\" class=\"wertiviewtoken  wertiviewVerbConjugation\">".to_string(),
    );
    tag.add_attribute("lemma", "boahtit");
    assert_eq!(
        tag.get_span_tag_start(),
        "<span id=\"X\" class=\"wertiviewtoken  wertiviewVerbConjugation\"lemma=\"boahtit\">"
    );

    // Attributes stack immediately before the closing bracket in call
    // order.
    tag.add_attribute("distractors", "boadán boaðát");
    assert_eq!(
        tag.get_span_tag_start(),
        "<span id=\"X\" class=\"wertiviewtoken  wertiviewVerbConjugation\"\
         lemma=\"boahtit\"distractors=\"boadán boaðát\">"
    );

    // The replacement is a plain replace-all, so every bracket in the
    // string is rewritten.
    let mut every = SpanTag::new(9, "<i>x</i>".to_string());
    every.add_attribute("k", "v");
    assert_eq!(every.get_span_tag_start(), "<ik=\"v\">x</ik=\"v\">");

    // The value is not escaped.
    let mut unescaped = SpanTag::new(9, "<span>".to_string());
    unescaped.add_attribute("answer", "a\"b");
    assert_eq!(unescaped.get_span_tag_start(), "<spananswer=\"a\"b\">");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-span-tag-start-fn/test]
#[test]
fn span_tag_get_start_reflects_added_attributes() {
    let mut tag = SpanTag::new(9, "<span>".to_string());
    assert_eq!(tag.get_span_tag_start(), "<span>");

    tag.add_attribute("lemma", "boahtit");
    assert_eq!(tag.get_span_tag_start(), "<spanlemma=\"boahtit\">");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.set-span-tag-start-fn/test]
#[test]
fn span_tag_set_start_discards_previously_added_attributes() {
    let mut tag = SpanTag::new(9, "<span>".to_string());
    tag.add_attribute("lemma", "boahtit");

    tag.set_span_tag_start("<span id=\"fresh\">".to_string());
    assert_eq!(tag.get_span_tag_start(), "<span id=\"fresh\">");
    assert_eq!(tag.get_span_tag_end(), "</span>");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-span-tag-end-fn/test]
#[test]
fn span_tag_get_end_defaults_to_closing_span() {
    let mut tag = SpanTag::new(9, SPAN_START.to_string());
    assert_eq!(tag.get_span_tag_end(), "</span>");

    // Adding attributes never touches the end tag.
    tag.add_attribute("lemma", "boahtit");
    assert_eq!(tag.get_span_tag_end(), "</span>");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.set-span-tag-end-fn/test]
#[test]
fn span_tag_set_end_overwrites_the_closing_markup() {
    let mut tag = SpanTag::new(9, SPAN_START.to_string());
    tag.set_span_tag_end("</div>".to_string());
    assert_eq!(tag.get_span_tag_end(), "</div>");
    assert_eq!(tag.get_span_tag_start(), SPAN_START);

    tag.set_span_tag_end(String::new());
    assert_eq!(tag.get_span_tag_end(), "");
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.hash-code-fn/test]
#[test]
fn span_tag_hash_code_folds_outer_end_start() {
    let tag = SpanTag::new(
        0,
        "<span id=\"X\" class=\"wertiviewtoken  wertiviewVerbConjugation\">".to_string(),
    );
    assert_eq!(tag.hash_code(), 1125471764);

    // The two string components are folded in different positions, so
    // swapping them changes the hash.
    let a = SpanTag {
        outer: 0,
        span_tag_start: "a".to_string(),
        span_tag_end: "b".to_string(),
    };
    let b = SpanTag {
        outer: 0,
        span_tag_start: "b".to_string(),
        span_tag_end: "a".to_string(),
    };
    assert_ne!(a.hash_code(), b.hash_code());

    // The hash changes as attributes are spliced in.
    let mut mutated = tag.clone();
    mutated.add_attribute("lemma", "boahtit");
    assert_ne!(mutated.hash_code(), tag.hash_code());
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.equals-fn/test]
#[test]
fn span_tag_equality_covers_enclosing_instance_and_fields() {
    let tag = SpanTag::new(9, SPAN_START.to_string());
    assert!(tag.equals(&SpanTag::new(9, SPAN_START.to_string())));
    assert!(tag.equals(&tag));

    assert!(!tag.equals(&SpanTag::new(8, SPAN_START.to_string())));
    assert!(!tag.equals(&SpanTag::new(9, "<span>".to_string())));

    let mut other_end = SpanTag::new(9, SPAN_START.to_string());
    other_end.set_span_tag_end("</div>".to_string());
    assert!(!tag.equals(&other_end));
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-outer-type-fn/test]
#[test]
fn span_tag_get_outer_type_returns_enclosing_instance() {
    let enhancer = Vislcg3VerbConjugationEnhancer::default();
    let tag = SpanTag::new(enhancer.outer_id(), SPAN_START.to_string());
    assert_eq!(tag.get_outer_type(), enhancer.outer_id());
    assert_ne!(SpanTag::new(9, SPAN_START.to_string()).get_outer_type(), 8);
}

// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.to-string-fn/test]
#[test]
fn span_tag_debug_rendering_names_both_fields() {
    let tag = SpanTag::new(9, "<span>".to_string());
    assert_eq!(
        tag.to_string(),
        "SpanTag [spanTagStart=<span>, spanTagEnd=</span>]"
    );

    let empty = SpanTag {
        outer: 9,
        span_tag_start: String::new(),
        span_tag_end: String::new(),
    };
    assert_eq!(empty.to_string(), "SpanTag [spanTagStart=, spanTagEnd=]");
}
