//! Use the TAG-B TAG-I sequences resulting from the CG3 analysis with
//! [`crate::pipeline::vislcg3`] to enhance spans corresponding to the tags
//! specified by the activity as tags of singular forms of substantives.
//!
//! The pass itself is the one the other tag-driven topics run, in
//! [`crate::enhancer::syntactic`]; what is this topic's own is the tag test,
//! and the base form and distractor forms the questioning exercises hang on
//! the span.
//!
//! Authors: Niels Ott, Adriane Boyd, Heli Uibo.

use anyhow::{Result, bail};
use tracing::info;

use crate::enhancer::cg_enhancer::GeneratorFailure;
use crate::enhancer::syntactic;
use crate::morpho::MorphoPipeline;
use crate::server::api::Mode;
use crate::types::{CgReading, CgToken, Document, ReadingJoin, flatten_reading};

/// The seven case slots a distractor set is generated over, in the order the
/// generator is asked for them.
const DISTRACT_FORMS: [&str; 7] = [
    "Sg+Nom", "Sg+Acc", "Sg+Gen", "Sg+Ill", "Sg+Loc", "Sg+Com", "Ess",
];

// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer]
#[derive(Default)]
pub struct Vislcg3NounSgEnhancer {
    n_sg_tags: Vec<String>,
}

impl Vislcg3NounSgEnhancer {
    /// The class every hit of this topic carries: `teaksta-` and the name the
    /// activity registry serves the topic under, which is how the client tells
    /// a hit from a plain word.
    pub const SPAN_CLASS: &'static str = "teaksta-SubstantiveSingular";
    pub const CHUNK_BEGIN_SUFFIX: &'static str = "-B";
    pub const CHUNK_INSIDE_SUFFIX: &'static str = "-I";

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.initialize-fn+1]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.initialize-fn+1]
    pub fn initialize(&mut self, n_sg_tags: Option<&str>) -> Result<()> {
        info!("Noun Sg tags {:?}", self.n_sg_tags);
        let param = match n_sg_tags {
            Some(p) => p,
            None => bail!("NSgTags configuration parameter is not set"),
        };
        self.n_sg_tags = param.split(',').map(str::to_string).collect();
        Ok(())
    }

    pub fn new(n_sg_tags: Option<&str>) -> Result<Self> {
        let mut this = Self::default();
        this.initialize(n_sg_tags)?;
        Ok(this)
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.process-fn+5]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.process-fn+5]
    pub fn process(&self, doc: &mut Document, mode: Mode) -> Result<()> {
        syntactic::run(
            doc,
            &syntactic::FunctionSpec {
                start_log: "Starting Noun Sg enhancement",
                finish_log: "Finished N Sg enhancement",
                span_class: Self::SPAN_CLASS,
                tag_class: None,
                tags: &self.n_sg_tags,
                is_safe: &|t| self.is_safe(t),
                contains_tag: &|cgr, tag| self.contains_tag(cgr, tag, mode),
                // this is the one tag-driven topic whose spans carry the
                // fields a question is built from
                attributes: Some(&|cgr| {
                    let (lemma, distractors) = self.reading_fields(cgr, mode)?;
                    Ok(vec![("lemma", lemma), ("distractors", distractors)])
                }),
            },
            mode,
        )
    }

    /// The base form and the distractor forms an exercise type needs. The
    /// two activities that only mark the token up carry neither, so both
    /// come back empty for them.
    fn reading_fields(&self, cgr: &CgReading, mode: Mode) -> Result<(String, String)> {
        let mut lemma = String::new();
        let mut distractors = String::new();

        if matches!(mode, Mode::Cloze | Mode::Mc) {
            // get lemma from the CG reading
            lemma = self.get_lemma(cgr)?;
        }
        if mode == Mode::Mc {
            // the reading is read twice more below, so it is flattened once
            let reading_str = flatten_reading(cgr, ReadingJoin::TrailingSpace);
            // Proper nouns have the tag "Prop" in the morphological
            // information. This is needed when generating distractors.
            let prop = tag_in_reading(cgr, &reading_str, "Prop", mode);
            // get stemtype from the CG reading, if any of these: G3, G7,
            // NomAg
            let stemtype = stem_type_of(&reading_str);
            // generate the distractors, based on the lemma, stemtype and if
            // it is a proper noun or not
            distractors = self.get_distractors(&lemma, stemtype, prop)?;
        }

        // Delete # from the lemma of compound words if any
        Ok((lemma.replace("#", ""), distractors))
    }

    /// Determines whether the given token is safe, i.e. unambiguous.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.is-safe-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.is-safe-fn]
    fn is_safe(&self, t: &CgToken) -> bool {
        t.readings.len() == 1
    }

    /// Determines whether the given reading contains the given tag.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.contains-tag-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.contains-tag-fn]
    fn contains_tag(&self, cgr: &CgReading, tag: &str, mode: Mode) -> bool {
        let reading_str = flatten_reading(cgr, ReadingJoin::TrailingSpace);
        tag_in_reading(cgr, &reading_str, tag, mode)
    }

    /// Obtains the lemma from the CG reading.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-lemma-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-lemma-fn+2]
    fn get_lemma(&self, cgr: &CgReading) -> Result<String> {
        let mut lemma = String::new();

        for rtag in cgr {
            let mut chars = rtag.chars();
            let first = match chars.next() {
                Some(c) => c,
                // charAt(0) on an empty tag is out of range
                None => bail!("string index out of range: 0"),
            };
            if first == '"' {
                let len = rtag.chars().count();
                // substring(1, len - 1) on a single-character tag is out of
                // range
                if len < 2 {
                    bail!("begin 1, end {}, length {}", len as i64 - 1, len);
                }
                lemma = rtag.chars().skip(1).take(len - 2).collect();
                info!("{:?} lemma: {}", cgr, lemma);
            }
        }

        // The lemma needs no UTF-8 re-encoding: the whole CG input and output
        // is already UTF-8.
        Ok(lemma)
    }

    /// Generates distractors for the multiple choice exercise.
    ///
    /// The stdout-draining plumbing the two external `lookup` processes
    /// needed — the consumer, its buffer and the shell pipelines the class
    /// assembled to spawn them — is subsumed by the morphological pipeline
    /// seam, which hands the transducer output back directly.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-distractors-fn+4]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-distractors-fn+4]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.ext-command-consume2-string-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.ext-command-consume2-string-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.run-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.run-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.is-done-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.is-done-fn]
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.get-buffer-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.ext-command-consume2-string.get-buffer-fn]
    fn get_distractors(&self, lemma: &str, stemtype: &str, propernoun: bool) -> Result<String> {
        let result = self.generated_forms(lemma, stemtype, propernoun)?;
        info!("Generated forms read from the outputfile: {}", result);
        Ok(result)
    }

    /// The surface forms the generator returns for one base form, each
    /// followed by a space. A transducer step that fails is the seam's
    /// failure and not this reading's: the Java printed it to standard output
    /// and returned nothing generated, where here it is raised so the request
    /// hears about it instead of being answered with an exercise whose
    /// questions have no wrong answers to choose between.
    fn generated_forms(&self, lemma: &str, stemtype: &str, propernoun: bool) -> Result<String> {
        let mut lemma = lemma.to_string();
        let mut generation_input = String::new();
        let prop_n = match propernoun {
            true => "+Prop",
            false => "",
        };

        let morpho = MorphoPipeline::shared();

        if lemma.contains('#') {
            // correct lemma for compound words = morf analysis - N+Sg+Nom
            lemma = lemma.replace("#", "");
            let from_fst = morpho
                .analyze_disambiguate(&[lemma.clone()])
                .map_err(|e| anyhow::Error::new(GeneratorFailure(e.to_string())))?;
            // the word may be morphologically ambiguous; take the first
            // analysis
            let analysis = from_fst.lines().next().unwrap_or_default();
            // the first field is the word to be analysed and the second field
            // is the morph analysis
            let fields: Vec<&str> = analysis.split('\t').collect();
            if fields.len() < 2 {
                bail!("Index 1 out of bounds for length {}", fields.len());
            }
            lemma = fields[1].replace("Sg+Nom", "");
            info!("lemma of the compound word: {}", lemma);

            for form in DISTRACT_FORMS {
                generation_input = generation_input + &lemma + form + "\n";
            }
        } else {
            for form in DISTRACT_FORMS {
                if !stemtype.is_empty() {
                    generation_input = format!(
                        "{}{}{}+N+{}+{}\n",
                        generation_input, lemma, prop_n, stemtype, form
                    );
                    generation_input = format!(
                        "{}{}{}+v1+N+{}+{}\n",
                        generation_input, lemma, prop_n, stemtype, form
                    );
                } else {
                    generation_input =
                        format!("{}{}{}+N+{}\n", generation_input, lemma, prop_n, form);
                    generation_input =
                        format!("{}{}{}+v1+N+{}\n", generation_input, lemma, prop_n, form);
                }
            }
        }

        let from_ifst = morpho
            .generate(&generation_input)
            .map_err(|e| anyhow::Error::new(GeneratorFailure(e.to_string())))?;

        let mut result = String::new();
        // StringTokenizer's default delimiter set
        for word in from_ifst
            .split(|c| matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0C'))
            .filter(|w| !w.is_empty())
        {
            info!("ifst output:{}", word);
            // forms that could not be generated are excluded, as well as
            // input strings of the iFST
            if !word.contains('+') && !word.contains('-') {
                result = result + word + " ";
            }
        }

        Ok(result)
    }
}

/// Whether a flattened reading carries the tag, over the exercise's own
/// exclusions. Both [`Vislcg3NounSgEnhancer::contains_tag`] and the
/// proper-noun probe read the same flattened reading through here.
fn tag_in_reading(cgr: &CgReading, reading_str: &str, tag: &str, mode: Mode) -> bool {
    // If the exercise type is "practice" (cloze) then the derived forms,
    // forms with clitics and proper nouns are excluded from the selection.
    if (reading_str.contains("Der/") || reading_str.contains("Qst"))
        && matches!(mode, Mode::Cloze | Mode::Mc)
    {
        info!("derived form or form with clitics");
        return false;
    }

    // Tag string contains the given tag sequence as a substring, plus the
    // POS tag 'N'.
    if reading_str.contains(tag) && reading_str.contains(" N ") {
        info!("{:?} contains {}", cgr, tag);
        return true;
    }

    false
}

/// Obtains the stem type from the morphological analysis if any (G3, G7,
/// NomAg), off the reading flattened the way the tag test reads it — the one
/// caller holds that string already.
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-stem-type-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-stem-type-fn]
fn stem_type_of(reading_str: &str) -> &'static str {
    if reading_str.contains("G3") {
        "G3"
    } else if reading_str.contains("G7") {
        "G7"
    } else if reading_str.contains("NomAg") {
        "NomAg"
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PIPELINE_LANGUAGE;

    fn reading(tags: &[&str]) -> CgReading {
        tags.iter().map(|tag| (*tag).to_string()).collect()
    }

    fn token(begin: usize, end: usize, readings: Vec<CgReading>) -> CgToken {
        CgToken {
            begin,
            end,
            readings,
        }
    }

    fn enhancer() -> Vislcg3NounSgEnhancer {
        Vislcg3NounSgEnhancer::default()
    }

    /// The stem type read off a reading, flattened the way the pass flattens
    /// it before handing the string on.
    fn stem_type(tags: &[&str]) -> &'static str {
        stem_type_of(&flatten_reading(&reading(tags), ReadingJoin::TrailingSpace))
    }

    fn span_start(id: &str) -> String {
        format!(
            "<span id=\"{}\" class=\"teaksta-token teaksta-SubstantiveSingular\" lemma=\"\" distractors=\"\">",
            id
        )
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.initialize-fn+1/test]
    #[test]
    fn initialize_splits_tag_list_on_commas_without_trimming() {
        let mut enhancer = Vislcg3NounSgEnhancer::default();

        enhancer
            .initialize(Some("Sg Nom, Sg Acc, Sg Gen, Sg Ill, Sg Loc, Sg Com, Ess"))
            .unwrap();

        assert_eq!(
            enhancer.n_sg_tags,
            vec![
                "Sg Nom", " Sg Acc", " Sg Gen", " Sg Ill", " Sg Loc", " Sg Com", " Ess"
            ]
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.initialize-fn+1/test]
    #[test]
    fn initialize_fails_when_the_tag_parameter_is_missing() {
        let mut enhancer = Vislcg3NounSgEnhancer::default();

        let err = enhancer.initialize(None).unwrap_err();

        assert!(err.to_string().contains("NSgTags"));
        assert!(enhancer.n_sg_tags.is_empty());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.is-safe-fn/test]
    #[test]
    fn only_token_with_exactly_one_reading_is_safe() {
        let enhancer = enhancer();
        let sole = reading(&["\"gietta\"", "N", "Sg", "Nom"]);

        assert!(!enhancer.is_safe(&token(0, 6, vec![])));
        assert!(enhancer.is_safe(&token(0, 6, vec![sole.clone()])));
        assert!(!enhancer.is_safe(&token(0, 6, vec![sole.clone(), sole])));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.contains-tag-fn/test]
    #[test]
    fn contains_tag_matches_substring_of_flattened_reading() {
        let enhancer = enhancer();
        let noun = reading(&["\"gietta\"", "N", "Sg", "Nom"]);

        assert!(enhancer.contains_tag(&noun, "Sg Nom", Mode::Colorize));
        assert!(enhancer.contains_tag(&noun, " Sg Nom", Mode::Colorize));
        assert!(enhancer.contains_tag(&noun, "\"gietta\"", Mode::Colorize));
        assert!(!enhancer.contains_tag(&noun, "Sg Acc", Mode::Colorize));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.contains-tag-fn/test]
    #[test]
    fn contains_tag_needs_the_space_delimited_noun_tag() {
        let enhancer = enhancer();

        let verb = reading(&["\"boahtit\"", "V", "Sg", "Nom"]);
        assert!(!enhancer.contains_tag(&verb, "Sg Nom", Mode::Colorize));

        let unquoted_first_tag = reading(&["N", "Sg", "Nom"]);
        assert!(!enhancer.contains_tag(&unquoted_first_tag, "Sg Nom", Mode::Colorize));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.contains-tag-fn/test]
    #[test]
    fn derived_and_clitic_excluded_only_for_cloze_mc() {
        let enhancer = enhancer();

        let derived = reading(&["\"gietta\"", "N", "Der/vuohta", "Sg", "Nom"]);
        assert!(!enhancer.contains_tag(&derived, "Sg Nom", Mode::Cloze));
        assert!(!enhancer.contains_tag(&derived, "Sg Nom", Mode::Mc));
        assert!(enhancer.contains_tag(&derived, "Sg Nom", Mode::Colorize));
        assert!(enhancer.contains_tag(&derived, "Sg Nom", Mode::Click));

        let clitic = reading(&["\"gietta\"", "N", "Sg", "Nom", "Qst"]);
        assert!(!enhancer.contains_tag(&clitic, "Sg Nom", Mode::Mc));
        assert!(enhancer.contains_tag(&clitic, "Sg Nom", Mode::Click));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-stem-type-fn/test]
    #[test]
    fn stem_type_reports_first_of_g3_g7_nomag() {
        assert_eq!(stem_type(&["\"bassi\"", "N", "G3", "G7", "NomAg"]), "G3");
        assert_eq!(stem_type(&["\"bassi\"", "N", "G7", "NomAg"]), "G7");
        assert_eq!(stem_type(&["\"lohkki\"", "N", "NomAg"]), "NomAg");
        assert_eq!(stem_type(&["\"gietta\"", "N", "Sg", "Nom"]), "");
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-stem-type-fn/test]
    #[test]
    fn stem_type_matches_inside_the_quoted_base_form() {
        assert_eq!(stem_type(&["\"G7-gáhkku\"", "N", "Sg", "Nom"]), "G7");
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-lemma-fn+2/test]
    #[test]
    fn lemma_strips_quotes_and_last_quoted_tag_wins() {
        let enhancer = enhancer();

        assert_eq!(
            enhancer
                .get_lemma(&reading(&["\"gietta\"", "N", "Sg", "Nom"]))
                .unwrap(),
            "gietta"
        );
        assert_eq!(
            enhancer
                .get_lemma(&reading(&["\"first\"", "N", "\"second\""]))
                .unwrap(),
            "second"
        );
        assert_eq!(
            enhancer
                .get_lemma(&reading(&["\"girji#gahppir\"", "N", "Sg", "Nom"]))
                .unwrap(),
            "girji#gahppir"
        );
        assert_eq!(
            enhancer.get_lemma(&reading(&["N", "Sg", "Nom"])).unwrap(),
            ""
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-lemma-fn+2/test]
    #[test]
    fn lemma_fails_on_empty_or_lone_quote_tag() {
        let enhancer = enhancer();

        let empty = enhancer.get_lemma(&reading(&["", "N"])).unwrap_err();
        assert!(
            empty.to_string().contains("string index out of range: 0"),
            "{empty}"
        );

        let lone_quote = enhancer.get_lemma(&reading(&["\"", "N"])).unwrap_err();
        assert!(
            lone_quote.to_string().contains("begin 1, end 0, length 1"),
            "{lone_quote}"
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.process-fn+5/test]
    #[test]
    fn process_wraps_tokens_in_numbered_substantive_spans() {
        let enhancer = Vislcg3NounSgEnhancer::new(Some("Sg Nom, Sg Acc")).unwrap();
        let mut doc = Document::new("gietta beana", PIPELINE_LANGUAGE);
        doc.cg_tokens = vec![
            token(0, 6, vec![reading(&["\"gietta\"", "N", "Sg", "Nom"])]),
            token(7, 12, vec![reading(&["\"beana\"", "N", "Sg", "Acc"])]),
        ];

        enhancer.process(&mut doc, Mode::Colorize).unwrap();

        assert_eq!(doc.enhancements.len(), 2);

        let first = &doc.enhancements[0];
        assert!(first.relevant);
        assert_eq!((first.begin, first.end), (0, 6));
        assert_eq!(first.enhance_start, span_start("teaksta-span-Sg Nom-1"));
        assert_eq!(first.enhance_end, "</span>");

        let second = &doc.enhancements[1];
        assert_eq!((second.begin, second.end), (7, 12));
        assert_eq!(second.enhance_start, span_start("teaksta-span- Sg Acc-1"));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.process-fn+5/test]
    #[test]
    fn process_numbers_per_tag_stopping_at_first_match() {
        let enhancer = Vislcg3NounSgEnhancer::new(Some("Sg,Nom")).unwrap();
        let mut doc = Document::new("gietta beana", PIPELINE_LANGUAGE);
        doc.cg_tokens = vec![
            token(
                0,
                6,
                vec![
                    reading(&["\"gietta\"", "N", "Sg", "Nom"]),
                    reading(&["\"gietta\"", "N", "Sg", "Gen"]),
                ],
            ),
            token(7, 12, vec![reading(&["\"beana\"", "N", "Sg", "Nom"])]),
        ];

        enhancer.process(&mut doc, Mode::Colorize).unwrap();

        let emitted: Vec<(usize, usize, &str)> = doc
            .enhancements
            .iter()
            .map(|e| (e.begin, e.end, e.enhance_start.as_str()))
            .collect();
        assert_eq!(
            emitted,
            vec![
                (0, 6, span_start("teaksta-span-Sg-1").as_str()),
                (7, 12, span_start("teaksta-span-Sg-2").as_str()),
                (0, 6, span_start("teaksta-span-Nom-1").as_str()),
                (7, 12, span_start("teaksta-span-Nom-2").as_str()),
            ]
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.process-fn+5/test]
    #[test]
    fn a_malformed_reading_reports_instead_of_unwinding() {
        let enhancer = enhancer();
        let broken = reading(&["", "N", "Sg", "Nom"]);

        // The two activities that mark the token up and nothing more need
        // neither field, so nothing can fail for them.
        assert_eq!(
            enhancer.reading_fields(&broken, Mode::Colorize).unwrap(),
            (String::new(), String::new())
        );

        let err = enhancer.reading_fields(&broken, Mode::Cloze).unwrap_err();

        assert!(
            err.to_string().contains("string index out of range"),
            "{err}"
        );

        // A well-formed reading still yields its base form, with the
        // compound boundary deleted.
        assert_eq!(
            enhancer
                .reading_fields(
                    &reading(&["\"girji#gahppir\"", "N", "Sg", "Nom"]),
                    Mode::Cloze
                )
                .unwrap(),
            ("girjigahppir".to_string(), String::new())
        );
    }

    /// The shape a generated distractor set has, whichever branch built it.
    fn assert_generated(result: &str) {
        assert!(result.is_empty() || result.ends_with(' '), "{result:?}");
        assert!(
            !result.contains('\n') && !result.contains('\t'),
            "{result:?}"
        );
        for word in result.split_whitespace() {
            assert!(!word.contains('+'), "{word} still carries generator tags");
            assert!(!word.contains('-'), "{word} is an ungenerated form");
        }
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-distractors-fn+4/test]
    #[test]
    fn distractors_hold_only_generated_surface_forms() {
        let enhancer = enhancer();

        for (lemma, stemtype, proper) in [
            ("gietta", "", false),
            ("lohkki", "NomAg", false),
            ("Deatnu", "G7", true),
        ] {
            match enhancer.get_distractors(lemma, stemtype, proper) {
                Ok(result) => assert_generated(&result),
                // a transducer the deployment cannot reach is reported, not
                // answered with a question that has no wrong answers
                Err(err) => assert!(err.is::<GeneratorFailure>(), "unexpected error: {err:#}"),
            }
        }
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-distractors-fn+4/test]
    #[test]
    fn a_compound_lemma_takes_the_analyser_branch() {
        let enhancer = enhancer();

        match enhancer.get_distractors("girji#gahppir", "", false) {
            Ok(result) => assert_generated(&result),
            // the analyser branch's own index error belongs to this reading,
            // and a seam that cannot be reached belongs to the deployment
            Err(err) => assert!(
                err.to_string().contains("Index 1 out of bounds") || err.is::<GeneratorFailure>(),
                "unexpected error: {err:#}"
            ),
        }
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.get-distractors-fn+4/test]
    #[test]
    fn a_seam_failure_is_not_an_unusable_reading() {
        // The two are told apart by type, not by message, because the pass
        // drops the one and raises the other.
        let unusable = anyhow::anyhow!("string index out of range: 0");
        assert!(!unusable.is::<GeneratorFailure>());

        let seam = anyhow::Error::new(GeneratorFailure("the generator is not set".to_string()));
        assert!(seam.is::<GeneratorFailure>());
        assert_eq!(seam.to_string(), "the generator is not set");
    }
}
