//! The output from the CG3 analysis in [`crate::pipeline::vislcg3`] is used to
//! enhance spans corresponding to the tags specified by the topic and the
//! activity that was chosen by the user. In this case the topic is North Sámi
//! connegative verb forms; the patterns in [`TOPIC`] select the tokens for
//! enhancement.
//!
//! The enhancement pass itself is the shared one in
//! [`crate::enhancer::cg_enhancer`]. What is this topic's own is the
//! connegative pattern and the fixed distractor table below.
//!
//! Authors: Niels Ott, Adriane Boyd, Heli Uibo, Eduard Schaf.

use anyhow::{Result, bail};
use tracing::info;

use crate::enhancer::cg_enhancer::{self, TopicSpec, Trace, Unchecked};
use crate::types::Document;

/// What this topic changes about the shared enhancement pass.
const TOPIC: TopicSpec = TopicSpec {
    label: "ConNeg",
    span_class: "wertiviewConNeg",
    pos: r"V\+",
    selector: r"Ind\+Prs\+ConNeg|Ind\+Prt\+ConNeg",
    hints: None,
    strip_lang_tag: true,
    log_chosen_reading: true,
    unchecked: Unchecked::Swallow,
    trace: Trace {
        span_tag: true,
        enhancement: true,
        possible_forms: false,
    },
};

// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer]
#[derive(Debug, Default)]
pub struct Vislcg3ConNegEnhancer {
    conneg_tags: Option<Vec<String>>,
}

impl Vislcg3ConNegEnhancer {
    /// Stands in for the enclosing instance captured by the Java inner
    /// classes: `Word` and `SpanTag` fold the enclosing enhancer's identity
    /// into their equality and hash, so instances only match when built by
    /// the same enhancer.
    pub(crate) fn outer_id(&self) -> usize {
        self as *const Self as usize
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.initialize-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.initialize-fn]
    pub fn initialize(&mut self, conneg_tags: Option<&str>) -> Result<()> {
        // the log statement runs before the assignment, so on a freshly
        // constructed instance it always logs null
        info!("ConNeg tags {:?}", self.conneg_tags);
        let param = match conneg_tags {
            Some(p) => p,
            None => bail!("connegTags configuration parameter is not set"),
        };
        self.conneg_tags = Some(param.split(',').map(str::to_string).collect());
        Ok(())
    }

    pub fn new(conneg_tags: Option<&str>) -> Result<Self> {
        let mut this = Self::default();
        this.initialize(conneg_tags)?;
        Ok(this)
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.process-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.process-fn]
    pub fn process(&self, doc: &mut Document) -> Result<()> {
        let forms = |reading: &str| self.write_morphological_forms(reading);
        let analyses = |reading: &str| self.write_lemma_and_analyses(reading);
        cg_enhancer::run(doc, self.outer_id(), &TOPIC, &forms, &analyses)
    }

    /// Create all relevant morphological forms of the current token. It is the
    /// input for the distractor generation.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.write-morphological-forms-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.write-morphological-forms-fn]
    fn write_morphological_forms(&self, reading_str: &str) -> Result<String> {
        let distract_forms = [
            "V+Ind+Prs+Sg1",
            "V+Ind+Prs+Sg2",
            "V+Ind+Prs+Sg3",
            "V+Ind+Prt+Sg1",
            "V+Ind+Prt+Sg2",
            "V+Ind+Prt+Sg3",
        ];
        cg_enhancer::lemma_distractors(reading_str, &distract_forms)
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.write-lemma-and-analyses-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.write-lemma-and-analyses-fn]
    fn write_lemma_and_analyses(&self, reading_str: &str) -> Result<String> {
        let plus = match reading_str.find('+') {
            Some(i) => i,
            // substring(0, -1) when there is no "+"
            None => bail!("begin 0, end -1, length {}", reading_str.chars().count()),
        };
        Ok(cg_enhancer::cloze_line(
            &reading_str[..plus],
            &reading_str[plus + 1..],
        ))
    }
}

#[cfg(test)]
#[path = "con_neg_tests.rs"]
mod tests;
