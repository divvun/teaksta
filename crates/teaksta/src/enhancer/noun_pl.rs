//! The output from the CG3 analysis in [`crate::pipeline::vislcg3`] is used to
//! enhance spans corresponding to the tags specified by the topic and the
//! activity that was chosen by the user. In this case the topic is North Sámi
//! nouns in plural form; the patterns in [`TOPIC`] extract the correct tokens
//! for enhancement.
//!
//! The enhancement pass itself is the shared one in
//! [`crate::enhancer::cg_enhancer`]. What is this topic's own is the plural
//! reading pattern and the pair of generator inputs below, which cut a
//! reading at its syntactic tag without first checking that it has one.
//!
//! Authors: Niels Ott, Adriane Boyd, Heli Uibo, Eduard Schaf.

use anyhow::{Result, bail};
use tracing::info;

use crate::enhancer::cg_enhancer::{self, TopicSpec, Trace, Unchecked};
use crate::types::Document;
use crate::util::jstring::char_index_of;

/// Since the analyses can be the following:
/// N+(Subclass)+(Semclass)+Number+Case(+Possessivesuffix)(+Clitic), all types
/// of optional tags are added so that for example the reading
/// beana+N+<sme>+Sem/Ani+Pl+Nom that was skipped is now considered valid.
const NUMBER_CASE_PATTERN: &str = concat!(
    r"([a-zA-Z]*[0-9]*\+)?(Sem/([a-zA-Z]*_*)*\+)?Pl\+Nom(\+<([a-zA-Z]*_*)*>)?(\+[a-zA-Z]*[0-9])?(\+[a-zA-Z]*)?(\+Foc/[a-zA-Z]*)?(\+[a-zA-Z]*)?|",
    r"([a-zA-Z]*[0-9]*\+)?(Sem/([a-zA-Z]*_*)*\+)?Pl\+Acc(\+<([a-zA-Z]*_*)*>)?(\+[a-zA-Z]*[0-9])?(\+[a-zA-Z]*)?(\+Foc/[a-zA-Z]*)?(\+[a-zA-Z]*)?|",
    r"([a-zA-Z]*[0-9]*\+)?(Sem/([a-zA-Z]*_*)*\+)?Pl\+Gen(\+<([a-zA-Z]*_*)*>)?(\+[a-zA-Z]*[0-9])?(\+[a-zA-Z]*)?(\+Foc/[a-zA-Z]*)?(\+[a-zA-Z]*)?|",
    r"([a-zA-Z]*[0-9]*\+)?(Sem/([a-zA-Z]*_*)*\+)?Pl\+Ill(\+<([a-zA-Z]*_*)*>)?(\+[a-zA-Z]*[0-9])?(\+[a-zA-Z]*)?(\+Foc/[a-zA-Z]*)?(\+[a-zA-Z]*)?|",
    r"([a-zA-Z]*[0-9]*\+)?(Sem/([a-zA-Z]*_*)*\+)?Pl\+Loc(\+<([a-zA-Z]*_*)*>)?(\+[a-zA-Z]*[0-9])?(\+[a-zA-Z]*)?(\+Foc/[a-zA-Z]*)?(\+[a-zA-Z]*)?|",
    r"([a-zA-Z]*[0-9]*\+)?(Sem/([a-zA-Z]*_*)*\+)?Pl\+Com(\+<([a-zA-Z]*_*)*>)?(\+[a-zA-Z]*[0-9])?(\+[a-zA-Z]*)?(\+Foc/[a-zA-Z]*)?(\+[a-zA-Z]*)?|",
    r"([a-zA-Z]*[0-9]*\+)?(Sem/([a-zA-Z]*_*)*\+)?Pl\+Ess(\+<([a-zA-Z]*_*)*>)?(\+[a-zA-Z]*[0-9])?(\+[a-zA-Z]*)?(\+Foc/[a-zA-Z]*)?(\+[a-zA-Z]*)?",
);

/// What this topic changes about the shared enhancement pass.
const TOPIC: TopicSpec = TopicSpec {
    label: "Noun Pl",
    span_class: "wertiviewSubstantivePlural",
    pos: r"N\+",
    selector: NUMBER_CASE_PATTERN,
    hints: None,
    strip_lang_tag: true,
    log_chosen_reading: false,
    unchecked: Unchecked::Propagate,
    trace: Trace {
        span_tag: true,
        enhancement: true,
        possible_forms: false,
    },
};

/// The cases both generator inputs sweep, in the order the first match wins.
const DISTRACT_FORMS_CASE: [&str; 7] = ["+Nom", "+Acc", "+Gen", "+Ill", "+Loc", "+Com", "+Ess"];

/// `substring(0, indexOf("@") - 1)` without the guard the sibling topics
/// apply: a string carrying no syntactic tag asks for `substring(0, -2)` and
/// one whose first character is `@` asks for `substring(0, -1)`, so a reading
/// the parser left untagged fails the whole call.
fn cut_at_syntactic_tag(analyses: &str) -> Result<String> {
    match char_index_of(analyses, '@') {
        Some(index) if index >= 1 => Ok(analyses.chars().take(index - 1).collect()),
        Some(_) => bail!("begin 0, end -1, length {}", analyses.chars().count()),
        None => bail!("begin 0, end -2, length {}", analyses.chars().count()),
    }
}

// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer]
#[derive(Debug, Default)]
pub struct Vislcg3NounPlEnhancer {
    n_pl_tags: Option<Vec<String>>,
}

impl Vislcg3NounPlEnhancer {
    /// Stands in for the enclosing instance captured by the Java inner
    /// classes: `Word` and `SpanTag` fold the enclosing enhancer's identity
    /// into their equality and hash, so instances only match when built by
    /// the same enhancer.
    pub(crate) fn outer_id(&self) -> usize {
        self as *const Self as usize
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.initialize-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.initialize-fn]
    pub fn initialize(&mut self, n_pl_tags: Option<&str>) -> Result<()> {
        // the log statement runs before the assignment, so it always reports
        // the field's previous value
        info!("Noun Pl tags {:?}", self.n_pl_tags);
        let param = match n_pl_tags {
            Some(p) => p,
            None => bail!("NPlTags configuration parameter is not set"),
        };
        self.n_pl_tags = Some(param.split(',').map(str::to_string).collect());
        Ok(())
    }

    pub fn new(n_pl_tags: Option<&str>) -> Result<Self> {
        let mut this = Self::default();
        this.initialize(n_pl_tags)?;
        Ok(this)
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.process-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.process-fn]
    pub fn process(&self, doc: &mut Document) -> Result<()> {
        let forms = |reading: &str| self.write_morphological_forms(reading);
        let analyses = |reading: &str| self.write_lemma_and_analyses(reading);
        cg_enhancer::run(doc, self.outer_id(), &TOPIC, &forms, &analyses)
    }

    /// Create all relevant morphological forms of the current token. It is the
    /// input for the distractor generation.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.write-morphological-forms-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.write-morphological-forms-fn]
    fn write_morphological_forms(&self, reading_str: &str) -> Result<String> {
        let mut reading_str = reading_str.to_string();
        let mut generation_input = String::new();
        let reading_str_input = cut_at_syntactic_tag(&reading_str)?;

        for a_case in DISTRACT_FORMS_CASE {
            if reading_str.contains(a_case) {
                // remove the case marker and the syntactic tag from the
                // reading
                reading_str = cg_enhancer::substring_to_index_of(&reading_str, a_case)?;
                // Assign distractorforms from the array
                for elem in DISTRACT_FORMS_CASE {
                    generation_input = generation_input + &reading_str + elem + "\n";
                }
                break;
            }
        }

        // add reading_str as last element in generationInput which will be
        // used as correct_answer
        generation_input = generation_input + &reading_str_input + "\n";

        // if generationInput contains tags_tbr, remove it
        Ok(cg_enhancer::remove_tags(&generation_input))
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.write-lemma-and-analyses-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.write-lemma-and-analyses-fn]
    fn write_lemma_and_analyses(&self, reading_str: &str) -> Result<String> {
        // substring(0, indexOf("+")): a reading without a "+" gives
        // substring(0, -1)
        let plus = match reading_str.find('+') {
            Some(i) => i,
            None => bail!("begin 0, end -1, length {}", reading_str.chars().count()),
        };
        let lemma_str = &reading_str[..plus];
        let analyses_str = reading_str[plus + 1..].replace("+<sme>", "");
        let analyses_str = cut_at_syntactic_tag(&analyses_str)?;

        // if analyses contains tags_tbr, remove it
        Ok(cg_enhancer::remove_tags(&format!(
            "{}+{}\n",
            lemma_str, analyses_str
        )))
    }
}

#[cfg(test)]
#[path = "noun_pl_tests.rs"]
mod tests;
