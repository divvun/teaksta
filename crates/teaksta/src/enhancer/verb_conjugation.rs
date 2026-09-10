//! The output from the CG3 analysis in [`crate::pipeline::vislcg3`] is used to
//! enhance spans corresponding to the tags specified by the topic and the
//! activity that was chosen by the user. In this case the topic is North Sámi
//! verbs in finite forms; the patterns in [`TOPIC`] select the tokens for
//! enhancement.
//!
//! The enhancement pass itself is the shared one in
//! [`crate::enhancer::cg_enhancer`]. What is this topic's own is the person
//! pattern and the mood-indexed distractor tables below.
//!
//! Authors: Niels Ott?, Adriane Boyd, Heli Uibo, Eduard Schaf.

use anyhow::{Result, bail};

use crate::enhancer::cg_enhancer::{self, TopicSpec, Trace, Unchecked};
use crate::types::Document;

/// What this topic changes about the shared enhancement pass.
const TOPIC: TopicSpec = TopicSpec {
    label: "VerbConjugation",
    span_class: "wertiviewVerbConjugation",
    pos: r"V\+",
    selector: r"Sg1|Sg2|Sg3|Du1|Du2|Du3|Pl1|Pl2|Pl3",
    hints: None,
    strip_lang_tag: false,
    log_chosen_reading: false,
    unchecked: Unchecked::Swallow,
    trace: Trace {
        span_tag: false,
        enhancement: false,
        possible_forms: false,
    },
};

/// Quirk: the indicative table has no plural rows at all, and opens with two
/// connegatives and an actio essive that are not finite forms.
const DISTRACT_FORMS_IND: [&str; 15] = [
    "V+Ind+Prs+ConNeg",
    "V+Ind+Prt+ConNeg",
    "V+Actio+Ess",
    "V+Ind+Prt+Sg1",
    "V+Ind+Prt+Sg2",
    "V+Ind+Prt+Sg3",
    "V+Ind+Prs+Sg1",
    "V+Ind+Prs+Sg2",
    "V+Ind+Prs+Sg3",
    "V+Ind+Prt+Du1",
    "V+Ind+Prt+Du2",
    "V+Ind+Prt+Du3",
    "V+Ind+Prs+Du1",
    "V+Ind+Prs+Du2",
    "V+Ind+Prs+Du3",
];

const DISTRACT_FORMS_IMPRT: [&str; 9] = [
    "V+Imprt+Sg1",
    "V+Imprt+Sg2",
    "V+Imprt+Sg3",
    "V+Imprt+Du1",
    "V+Imprt+Du2",
    "V+Imprt+Du3",
    "V+Imprt+Pl1",
    "V+Imprt+Pl2",
    "V+Imprt+Pl3",
];

const DISTRACT_FORMS_COND: [&str; 9] = [
    "V+Cond+Prs+Sg1",
    "V+Cond+Prs+Sg2",
    "V+Cond+Prs+Sg3",
    "V+Cond+Prs+Du1",
    "V+Cond+Prs+Du2",
    "V+Cond+Prs+Du3",
    "V+Cond+Prs+Pl1",
    "V+Cond+Prs+Pl2",
    "V+Cond+Prs+Pl3",
];

const DISTRACT_FORMS_POT: [&str; 9] = [
    "V+Pot+Prs+Sg1",
    "V+Pot+Prs+Sg2",
    "V+Pot+Prs+Sg3",
    "V+Pot+Prs+Du1",
    "V+Pot+Prs+Du2",
    "V+Pot+Prs+Du3",
    "V+Pot+Prs+Pl1",
    "V+Pot+Prs+Pl2",
    "V+Pot+Prs+Pl3",
];

const DISTRACT_FORMS_NEG: [&str; 9] = [
    "V+Neg+Ind+Sg1",
    "V+Neg+Ind+Sg2",
    "V+Neg+Ind+Sg3",
    "V+Neg+Ind+Du1",
    "V+Neg+Ind+Du2",
    "V+Neg+Ind+Du3",
    "V+Neg+Ind+Pl1",
    "V+Neg+Ind+Pl2",
    "V+Neg+Ind+Pl3",
];

/// The mood the reading carries picks its distractor table. A mood outside
/// this list fires no table at all, leaving the answer line on its own, so
/// the token can never reach the two-distractor minimum.
const MOOD_TABLES: [(&str, &[&str]); 5] = [
    ("Ind", &DISTRACT_FORMS_IND),
    ("Imprt", &DISTRACT_FORMS_IMPRT),
    ("Cond", &DISTRACT_FORMS_COND),
    ("Pot", &DISTRACT_FORMS_POT),
    ("Neg", &DISTRACT_FORMS_NEG),
];

/// The fixed-width array the Java splits a reading into: lemma, part of
/// speech, transitivity, mood, tense, person. Only the lemma and the mood are
/// ever read back.
const TAG_SLOTS: usize = 20;

/// Split the reading into its tag slots. The array is not grown, so a reading
/// with more than [`TAG_SLOTS`] tags raises instead of being truncated.
fn split_tags(reading_str: &str) -> Result<Vec<String>> {
    let mut tags: Vec<String> = Vec::new();
    for token in reading_str.split('+').filter(|t| !t.is_empty()) {
        if tags.len() >= TAG_SLOTS {
            bail!(
                "Index {} out of bounds for length {}",
                tags.len(),
                TAG_SLOTS
            );
        }
        tags.push(token.to_string());
    }
    Ok(tags)
}

// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer]
#[derive(Debug, Clone, Default)]
pub struct Vislcg3VerbConjugationEnhancer {
    pub fin_verb_tags: Option<Vec<String>>,
}

impl Vislcg3VerbConjugationEnhancer {
    /// Stands in for the enclosing instance captured by the Java inner
    /// classes: `Word` and `SpanTag` fold the enclosing enhancer's identity
    /// into their equality and hash, so instances only match when built by
    /// the same enhancer.
    pub(crate) fn outer_id(&self) -> usize {
        self as *const Self as usize
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.initialize-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.initialize-fn]
    pub fn initialize(&mut self, finverb_tags: Option<&str>) -> Result<()> {
        let param = match finverb_tags {
            Some(p) => p,
            // the Java cast of a missing parameter yields null and .split
            // raises a NullPointerException out of initialize
            None => bail!("finverbTags configuration parameter is not set"),
        };
        self.fin_verb_tags = Some(param.split(',').map(str::to_string).collect());
        Ok(())
    }

    pub fn new(finverb_tags: Option<&str>) -> Result<Self> {
        let mut this = Self::default();
        this.initialize(finverb_tags)?;
        Ok(this)
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.process-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.process-fn]
    pub fn process(&self, doc: &mut Document) -> Result<()> {
        let forms = |reading: &str| self.write_morphological_forms(reading);
        let analyses = |reading: &str| self.write_lemma_and_analyses(reading);
        cg_enhancer::run(doc, self.outer_id(), &TOPIC, &forms, &analyses)
    }

    /// Create all relevant morphological forms of the current token. It is the
    /// input for the distractor generation.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-morphological-forms-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-morphological-forms-fn]
    fn write_morphological_forms(&self, reading_str: &str) -> Result<String> {
        let tags = split_tags(reading_str)?;
        // null array slots concatenate as the text "null" in Java
        let lemma = tags.first().map(String::as_str).unwrap_or("null");
        let mood = match tags.get(3) {
            Some(m) => m.as_str(),
            // mood.equals on a null slot raises a NullPointerException
            None => bail!("mood is null"),
        };

        let mut generation_input = String::new();
        for (name, distract_forms) in MOOD_TABLES {
            if mood == name {
                for form in distract_forms {
                    generation_input = generation_input + lemma + "+" + form + "\n";
                }
            }
        }

        // add reading_str as last element in generationInput which will be
        // used as correct_answer
        generation_input += &cg_enhancer::correct_answer_line(reading_str);

        // if generationInput contains tags_tbr, remove it
        Ok(cg_enhancer::remove_tags(&generation_input))
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-lemma-and-analyses-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-lemma-and-analyses-fn]
    fn write_lemma_and_analyses(&self, reading_str: &str) -> Result<String> {
        let (lemma_str, an_tmp) = cg_enhancer::split_lemma_dropping_last(reading_str)?;
        Ok(cg_enhancer::cloze_line(&lemma_str, &an_tmp))
    }
}

#[cfg(test)]
#[path = "verb_conjugation_tests.rs"]
mod tests;
