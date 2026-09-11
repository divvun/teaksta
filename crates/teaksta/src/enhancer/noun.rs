//! The output from the CG3 analysis in [`crate::pipeline::vislcg3`] is being
//! used to enhance spans corresponding to the tags specified by the topic and
//! the activity that was chosen by the user. In this case the topic is North
//! Sámi nouns in singular form; the patterns in [`TOPIC`] extract the correct
//! tokens for enhancement.
//!
//! This is the only topic that runs the preposition-hint pass and the only
//! one that drops a token because one of its readings is an unlikely part of
//! speech; everything else it does is the shared pass in
//! [`crate::enhancer::cg_enhancer`].
//!
//! Authors: Niels Ott, Adriane Boyd, Heli Uibo, Eduard Schaf.

use std::sync::LazyLock;

use anyhow::{Result, bail};
use regex::Regex;
use tracing::info;

use crate::enhancer::cg_enhancer::{self, HintRules, TopicSpec, Trace, Unchecked};
use crate::server::api::Mode;
use crate::types::Document;

/// The `A\+(?!.*Pred)` alternative of the exclude pattern. The regex crate has
/// no lookaround, so the negative lookahead is evaluated directly: an `A+`
/// occurrence qualifies only when no `Pred` follows it on the same line, since
/// Java's `.` does not cross a line terminator.
fn a_plus_without_pred(input: &str) -> bool {
    let mut from = 0;
    while let Some(rel) = input[from..].find("A+") {
        let after = from + rel + 2;
        let rest = &input[after..];
        let line_rest = match rest.find('\n') {
            Some(i) => &rest[..i],
            None => rest,
        };
        if !line_rest.contains("Pred") {
            return true;
        }
        from = after;
    }
    false
}

/// The alternatives of the exclude pattern that need no lookaround.
static EXCLUDE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"V\+|Det|Pr$|Pron\+|Pcle|Adv|Interj|CC|CS|ACR\+Dyn").expect("exclude pattern")
});

/// `excludePattern.matcher(input).find()` for
/// `"V\\+|A\\+(?!.*Pred)|Det|Pr$|Pron\\+|Pcle|Adv|Interj|CC|CS|ACR\\+Dyn"`,
/// with the lookahead branch spliced back in.
fn exclude_find(input: &str) -> bool {
    EXCLUDE_PATTERN.is_match(input) || a_plus_without_pred(input)
}

/// Since the analyses can be the following:
/// N+(Subclass)+(Semclass)+Number+Case(+Possessivesuffix)(+Clitic), all types
/// of optional tags are added so that for example the reading
/// čáhci+N+<sme>+Sem/Plc_Substnc_Wthr+Sg+Nom that was skipped is now
/// considered valid.
///
/// The number alternation is grouped, so a branch is one number followed by
/// one case — the thirteen combinations `NTags` names. The essive carries no
/// number, matching the bare `Ess` entry of that same list, and the
/// attributive branch stands on its own.
const NUMBER_PATTERN: &str = concat!(
    r"([a-zA-Z]*[0-9]*\+)?(Sem/([a-zA-Z]*_*)*\+)?(Sg|Pl)\+Nom(\+<([a-zA-Z]*_*)*>)?(\+[a-zA-Z]*[0-9])?(\+[a-zA-Z]*)?(\+Foc/[a-zA-Z]*)?(\+[a-zA-Z]*)?|",
    r"([a-zA-Z]*[0-9]*\+)?(Sem/([a-zA-Z]*_*)*\+)?(Sg|Pl)\+Acc(\+<([a-zA-Z]*_*)*>)?(\+[a-zA-Z]*[0-9])?(\+[a-zA-Z]*)?(\+Foc/[a-zA-Z]*)?(\+[a-zA-Z]*)?|",
    r"([a-zA-Z]*[0-9]*\+)?(Sem/([a-zA-Z]*_*)*\+)?(Sg|Pl)\+Gen(\+<([a-zA-Z]*_*)*>)?(\+[a-zA-Z]*[0-9])?(\+[a-zA-Z]*)?(\+Foc/[a-zA-Z]*)?(\+[a-zA-Z]*)?|",
    r"([a-zA-Z]*[0-9]*\+)?(Sem/([a-zA-Z]*_*)*\+)?(Sg|Pl)\+Ill(\+<([a-zA-Z]*_*)*>)?(\+[a-zA-Z]*[0-9])?(\+[a-zA-Z]*)?(\+Foc/[a-zA-Z]*)?(\+[a-zA-Z]*)?|",
    r"([a-zA-Z]*[0-9]*\+)?(Sem/([a-zA-Z]*_*)*\+)?(Sg|Pl)\+Loc(\+<([a-zA-Z]*_*)*>)?(\+[a-zA-Z]*[0-9])?(\+[a-zA-Z]*)?(\+Foc/[a-zA-Z]*)?(\+[a-zA-Z]*)?|",
    r"([a-zA-Z]*[0-9]*\+)?(Sem/([a-zA-Z]*_*)*\+)?(Sg|Pl)\+Com(\+<([a-zA-Z]*_*)*>)?(\+[a-zA-Z]*[0-9])?(\+[a-zA-Z]*)?(\+Foc/[a-zA-Z]*)?(\+[a-zA-Z]*)?|",
    r"([a-zA-Z]*[0-9]*\+)?(Sem/([a-zA-Z]*_*)*\+)?\+Ess(\+<([a-zA-Z]*_*)*>)?(\+[a-zA-Z]*[0-9])?(\+[a-zA-Z]*)?(\+Foc/[a-zA-Z]*)?(\+[a-zA-Z]*)?|",
    r"([a-zA-Z]*[0-9]*\+)?(Sem/([a-zA-Z]*_*)*\+)?\+Attr(\+<([a-zA-Z]*_*)*>)?(\+[a-zA-Z]*[0-9])?(\+[a-zA-Z]*)?(\+Foc/[a-zA-Z]*)?(\+[a-zA-Z]*)?",
);

/// What this topic changes about the shared enhancement pass.
const TOPIC: TopicSpec = TopicSpec {
    label: "Noun Sg",
    span_class: "teaksta-Substantive",
    pos: r"N\+",
    selector: NUMBER_PATTERN,
    hints: Some(HintRules {
        hint: r"Pr$",
        // the following tags are allowed between hint and noun
        valid_hint: r"A\+|Det|Adv",
        exclude: exclude_find,
    }),
    strip_lang_tag: true,
    log_chosen_reading: false,
    unchecked: Unchecked::Propagate,
    trace: Trace {
        span_tag: true,
        enhancement: true,
        possible_forms: true,
    },
};

/// The singular sweep: `(probe, cut marker, distractor rows)`. The probe and
/// the marker differ for every case but the nominative, which is why a
/// reading carrying a case tag without its number raises instead of matching.
const SG_CASES: [(&str, &str, [&str; 4]); 6] = [
    // check if the strings contains Sg+Nom instead of only Nom because in
    // case of NomAg: oahpaheaddji+N+NomAg+Sem/Hum+Sg+Gen+@ADVL> indexOf
    // returns -1, and substring(0,-1) returns an error
    (
        "+Sg+Nom",
        "+Sg+Nom",
        ["+Sg+Acc", "+Sg+Ill", "+Sg+Loc", "+Sg+Com"],
    ),
    (
        "+Acc",
        "+Sg+Acc",
        ["+Sg+Nom", "+Sg+Ill", "+Sg+Loc", "+Sg+Com"],
    ),
    (
        "+Gen",
        "+Sg+Gen",
        ["+Sg+Nom", "+Sg+Ill", "+Sg+Loc", "+Sg+Com"],
    ),
    ("+Ill", "+Sg+Ill", ["+Sg+Nom", "+Sg+Com", "+Ess", "+Sg+Acc"]),
    ("+Loc", "+Sg+Loc", ["+Sg+Nom", "+Sg+Com", "+Ess", "+Sg+Acc"]),
    ("+Com", "+Sg+Com", ["+Sg+Nom", "+Sg+Ill", "+Ess", "+Sg+Acc"]),
];

/// The essive sweep, which sits between the two number sweeps and is reached
/// whatever number the reading carries.
const ESS_CASES: [(&str, &str, [&str; 4]); 1] =
    [("+Ess", "+Ess", ["+Pl+Nom", "+Sg+Ill", "+Sg+Loc", "+Pl+Acc"])];

/// The plural sweep. Quirk: the comitative row ends on a singular accusative
/// where every sibling row stays plural.
const PL_CASES: [(&str, &str, [&str; 4]); 6] = [
    // same comment as for Sg
    (
        "+Pl+Nom",
        "+Pl+Nom",
        ["+Pl+Acc", "+Pl+Ill", "+Pl+Loc", "+Pl+Com"],
    ),
    (
        "+Acc",
        "+Pl+Acc",
        ["+Pl+Nom", "+Pl+Ill", "+Pl+Loc", "+Pl+Com"],
    ),
    (
        "+Gen",
        "+Pl+Gen",
        ["+Pl+Nom", "+Pl+Ill", "+Pl+Loc", "+Pl+Com"],
    ),
    ("+Ill", "+Pl+Ill", ["+Pl+Nom", "+Pl+Com", "+Ess", "+Pl+Acc"]),
    ("+Loc", "+Pl+Loc", ["+Pl+Nom", "+Pl+Ill", "+Ess", "+Pl+Acc"]),
    ("+Com", "+Pl+Com", ["+Pl+Nom", "+Pl+Ill", "+Ess", "+Sg+Acc"]),
];

/// The cases a cloze reading gains a counterpart for: `(probe, counterpart,
/// only when the counterpart is absent)`. The singular rows are unguarded and
/// the plural rows are not, so a reading already carrying both numbers grows
/// only in the singular direction.
const COUNTERPARTS: [(&str, &str, bool); 10] = [
    ("+Sg+Acc", "+Pl+Acc", false),
    ("+Sg+Gen", "+Pl+Gen", false),
    ("+Sg+Ill", "+Pl+Ill", false),
    ("+Sg+Com", "+Pl+Com", false),
    ("+Sg+Loc", "+Pl+Loc", false),
    ("+Pl+Acc", "+Sg+Acc", true),
    ("+Pl+Gen", "+Sg+Gen", true),
    ("+Pl+Ill", "+Sg+Ill", true),
    ("+Pl+Com", "+Sg+Com", true),
    ("+Pl+Loc", "+Sg+Loc", true),
];

/// The first generator input the Java builds, sweeping the bare case markers
/// instead of the number-and-case pairs. Its content never reaches the
/// generator — `writeMorphologicalForms` returns the second block — but it is
/// built all the same, so a reading it cannot cut fails the whole call.
fn discarded_case_sweep(reading_str: &str, answer_line: &str) -> Result<String> {
    let distract_forms_case = ["+Nom", "+Acc", "+Gen", "+Ill", "+Loc", "+Com", "+Ess"];
    let mut reading_str = reading_str.to_string();
    let mut generation_input = String::new();
    for a_case in distract_forms_case {
        if reading_str.contains(a_case) {
            // remove the case marker and the syntactic tag from the reading
            reading_str = cg_enhancer::substring_to_index_of(&reading_str, a_case)?;
            // Assign distractorforms from the array
            for elem in distract_forms_case {
                generation_input = generation_input + &reading_str + elem + "\n";
            }
            break;
        }
    }
    generation_input += answer_line;
    // if generationInput contains tags_tbr, remove it
    Ok(cg_enhancer::remove_tags(&generation_input))
}

/// Cut the reading back to each case marker it still carries and emit that
/// case's distractor rows. The cut carries over between rows, so a reading
/// with several case tags is trimmed progressively.
fn sweep_cases(
    reading: &mut String,
    generation_input: &mut String,
    cases: &[(&str, &str, [&str; 4])],
) -> Result<()> {
    for (probe, marker, distractors) in cases {
        if reading.contains(probe) {
            *reading = cg_enhancer::substring_to_index_of(reading, marker)?;
            // Assign distractorforms from the array
            for elem in distractors {
                generation_input.push_str(reading);
                generation_input.push_str(elem);
                generation_input.push('\n');
            }
        }
    }
    Ok(())
}

// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer]
#[derive(Default)]
pub struct Vislcg3NounEnhancer {
    pub n_tags: Option<Vec<String>>,
}

impl Vislcg3NounEnhancer {
    /// The class every hit of this topic carries: `teaksta-` and the name the
    /// activity registry serves the topic under, which is how the client tells
    /// a hit from a plain word.
    pub const SPAN_CLASS: &'static str = TOPIC.span_class;

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.initialize-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.initialize-fn]
    pub fn initialize(&mut self, n_tags: Option<&str>) -> Result<()> {
        let param = match n_tags {
            Some(p) => p,
            None => bail!("NTags configuration parameter is not set"),
        };
        self.n_tags = Some(param.split(',').map(str::to_string).collect());
        Ok(())
    }

    pub fn new(n_tags: Option<&str>) -> Result<Self> {
        let mut this = Self::default();
        this.initialize(n_tags)?;
        Ok(this)
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.process-fn+5]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.process-fn+5]
    pub fn process(&self, doc: &mut Document, mode: Mode) -> Result<()> {
        let forms = |reading: &str| self.write_morphological_forms(reading);
        let analyses = |reading: &str| self.write_lemma_and_analyses(reading);
        cg_enhancer::run(doc, &TOPIC, mode, &forms, &analyses)
    }

    /// Create all relevant morphological forms of the current token. It is the
    /// input for the distractor generation.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-morphological-forms-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-morphological-forms-fn+2]
    fn write_morphological_forms(&self, reading_str: &str) -> Result<String> {
        // add reading_str as last element in generationInput which will be
        // used as correct_answer
        let answer_line = cg_enhancer::correct_answer_line(reading_str);

        let mut reading_str2 = reading_str.to_string();
        let mut generation_input2 = String::new();

        if reading_str2.contains("+Sg") {
            sweep_cases(&mut reading_str2, &mut generation_input2, &SG_CASES)?;
        }
        if reading_str2.contains("+Ess") {
            sweep_cases(&mut reading_str2, &mut generation_input2, &ESS_CASES)?;
        }
        if reading_str2.contains("+Pl") {
            sweep_cases(&mut reading_str2, &mut generation_input2, &PL_CASES)?;
        }

        info!("generationInput2={}", generation_input2);

        discarded_case_sweep(reading_str, &answer_line)?;

        generation_input2 += &answer_line;
        // if generationInput contains tags_tbr, remove it
        Ok(cg_enhancer::remove_tags(&generation_input2))
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-lemma-and-analyses-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-lemma-and-analyses-fn+2]
    fn write_lemma_and_analyses(&self, reading_str: &str) -> Result<String> {
        let plus = match reading_str.find('+') {
            Some(i) => i,
            // substring(0, -1) when there is no "+"
            None => bail!("begin 0, end -1, length {}", reading_str.chars().count()),
        };
        let mut lem_and_an =
            cg_enhancer::cloze_line(&reading_str[..plus], &reading_str[plus + 1..]);

        for (probe, counterpart, only_when_absent) in COUNTERPARTS {
            if !lem_and_an.contains(probe) || (only_when_absent && lem_and_an.contains(counterpart))
            {
                continue;
            }
            let temp_str = cg_enhancer::substring_to_index_of(&lem_and_an, probe)?;
            lem_and_an = lem_and_an + &temp_str + counterpart + "\n";
        }

        Ok(lem_and_an)
    }
}

#[cfg(test)]
#[path = "noun_tests.rs"]
mod tests;
