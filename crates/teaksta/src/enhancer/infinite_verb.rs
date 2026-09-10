//! The output from the CG3 analysis in [`crate::pipeline::vislcg3`] is used to
//! enhance spans corresponding to the tags specified by the topic and the
//! activity that was chosen by the user. In this case the topic is North Sámi
//! verbs in non-finite forms; the patterns in [`TOPIC`] extract the correct
//! tokens for enhancement.
//!
//! The enhancement pass itself is the shared one in
//! [`crate::enhancer::cg_enhancer`]. What is this topic's own is the
//! non-finite pattern and the fixed distractor table below.
//!
//! Authors: Niels Ott, Adriane Boyd, Heli Uibo, Eduard Schaf.

use anyhow::Result;

use crate::enhancer::cg_enhancer::{self, TopicSpec, Trace, Unchecked};
use crate::types::Document;

/// What this topic changes about the shared enhancement pass.
const TOPIC: TopicSpec = TopicSpec {
    label: "InfiniteVerb",
    span_class: "teaksta-InfiniteVerb",
    pos: r"V\+",
    selector: r"PrfPrc|VGen|VAbess|Ger|Actio\+Ess|Inf|ConNeg",
    hints: None,
    strip_lang_tag: false,
    log_chosen_reading: false,
    unchecked: Unchecked::Propagate,
    trace: Trace {
        span_tag: false,
        enhancement: false,
        possible_forms: false,
    },
};

// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer]
#[derive(Debug, Default)]
pub struct Vislcg3InfiniteVerbEnhancer {
    /// Set from the `infiniteverbTags` descriptor parameter and never read
    /// again: `process` matches readings with [`TOPIC`] instead.
    pub infverb_tags: Option<Vec<String>>,
}

impl Vislcg3InfiniteVerbEnhancer {
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.initialize-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.initialize-fn]
    pub fn initialize(&mut self, infiniteverb_tags: Option<&str>) -> Result<()> {
        let param = match infiniteverb_tags {
            Some(p) => p,
            None => anyhow::bail!("infiniteverbTags configuration parameter is not set"),
        };
        self.infverb_tags = Some(param.split(',').map(str::to_string).collect());
        Ok(())
    }

    pub fn new(infiniteverb_tags: Option<&str>) -> Result<Self> {
        let mut this = Self::default();
        this.initialize(infiniteverb_tags)?;
        Ok(this)
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.process-fn+3]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.process-fn+3]
    pub fn process(&self, doc: &mut Document) -> Result<()> {
        let forms = |reading: &str| self.write_morphological_forms(reading);
        let analyses = |reading: &str| self.write_lemma_and_analyses(reading);
        cg_enhancer::run(doc, &TOPIC, &forms, &analyses)
    }

    /// Create all relevant morphological forms of the current token. It is the
    /// input for the distractor generation.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.write-morphological-forms-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.write-morphological-forms-fn+2]
    fn write_morphological_forms(&self, reading_str: &str) -> Result<String> {
        let distract_forms = [
            "V+Ind+Prs+Sg1",
            "V+Ind+Prs+Sg2",
            "V+Ind+Prs+Sg3",
            "V+Ind+Prs+Du1",
            "V+Ind+Prs+Du2",
            "V+Ind+Prs+Du3",
            "V+Ind+Prt+Sg1",
            "V+Ind+Prt+Sg2",
            "V+Ind+Prt+Sg3",
            "V+Ind+Prt+Du1",
            "V+Ind+Prt+Du2",
            "V+Ind+Prt+Du3",
        ];
        cg_enhancer::lemma_distractors(reading_str, &distract_forms)
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.write-lemma-and-analyses-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.write-lemma-and-analyses-fn+2]
    fn write_lemma_and_analyses(&self, reading_str: &str) -> Result<String> {
        let (lemma_str, an_tmp) = cg_enhancer::split_lemma_dropping_last(reading_str)?;
        Ok(cg_enhancer::cloze_line(&lemma_str, &an_tmp))
    }
}

#[cfg(test)]
#[path = "infinite_verb_tests.rs"]
mod tests;
