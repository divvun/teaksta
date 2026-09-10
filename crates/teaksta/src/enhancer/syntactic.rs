//! The enhancement pass the three syntactic-function topics hold in common.
//!
//! `Vislcg3SubjectEnhancer`, `Vislcg3ObjectEnhancer` and
//! `Vislcg3AdverbialEnhancer` walk the configured tags, then the tokens, and
//! wrap the first reading carrying the tag in a span. What genuinely differs
//! between them — the log lines, the span class, the configured tags and how
//! a reading is tested — is named by [`FunctionSpec`] and supplied per topic.
//!
//! Authors: Niels Ott?, Adriane Boyd, Heli Uibo

use std::collections::HashMap;

use anyhow::Result;
use tracing::info;

use crate::enhancer::cg_span::{SpanTag, TOKEN_CLASS};
use crate::server::api::Mode;
use crate::types::{CgReading, CgToken, Document, Enhancement};
use crate::util::enhancer_utils;

/// Everything one syntactic-function topic changes about the shared pass.
pub struct FunctionSpec<'a> {
    /// Names the topic in the opening log line.
    pub start_log: &'a str,
    /// Names the topic in the closing log line. The three Java classes were
    /// edited apart over time and no longer word this alike.
    pub finish_log: &'a str,
    /// The second CSS class on every enhanced span.
    pub span_class: &'a str,
    /// The tags the activity configured, in the order spans are numbered.
    pub tags: &'a [String],
    /// Whether the token is unambiguous enough for the exercise types that
    /// ask a question about it.
    pub is_safe: &'a dyn Fn(&CgToken) -> bool,
    /// Whether a reading carries the tag; each topic tests this its own way.
    pub contains_tag: &'a dyn Fn(&CgReading, &str) -> bool,
}

/// Run one syntactic-function topic's enhancement pass over `doc`, for the
/// exercise `mode` names. The body of every `Vislcg3*Enhancer::process` among
/// the three; see [`FunctionSpec`] for what each topic changes about it.
pub fn run(doc: &mut Document, spec: &FunctionSpec<'_>, mode: Mode) -> Result<()> {
    info!("{}", spec.start_log);

    // keep track of ids for each annotation class
    let mut class_counts: HashMap<String, i32> = HashMap::new();
    for con_t in spec.tags {
        class_counts.insert(con_t.clone(), 0);
        info!("Tag: {}", con_t);
    }

    // iterating over the configured tags instead of the class-count key set
    // because it is important to control the order in which spans are enhanced

    for con_t in spec.tags {
        // go through tokens
        for token_index in 0..doc.cg_tokens.len() {
            if matches!(mode, Mode::Cloze | Mode::Mc) {
                // more than one reading? don't mark up if the exercise type is
                // mc or cloze
                if !(spec.is_safe)(&doc.cg_tokens[token_index]) {
                    continue;
                }
            }

            let (begin, end, reading_count) = {
                let cgt = &doc.cg_tokens[token_index];
                (cgt.begin, cgt.end, cgt.readings.len())
            };

            // analyze reading(s)
            // Loop over all the readings. If there is one analysis that matches
            // the tag pattern then the token will be selected for the exercise.
            for i in 0..reading_count {
                let reading = &doc.cg_tokens[token_index].readings[i];
                if !(spec.contains_tag)(reading, con_t) {
                    continue;
                }

                // the lemma of the CG reading and the distractors generated
                // from it are not needed for exercises on syntactic functions

                // increment id
                let new_id = class_counts[con_t] + 1;
                let id = enhancer_utils::get_id(&format!("teaksta-span-{con_t}"), new_id);
                let span_tag = SpanTag::new(id, &[TOKEN_CLASS, spec.span_class]);
                // make new enhancement
                let e = Enhancement {
                    begin,
                    end,
                    enhance_start: span_tag.start_tag(),
                    enhance_end: span_tag.end_tag().to_string(),
                    relevant: true,
                };
                class_counts.insert(con_t.clone(), new_id);
                doc.enhancements.push(e);
                break;
            }
        }
    }

    info!("{}", spec.finish_log);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::cg_token;
    use crate::types::PIPELINE_LANGUAGE;

    /// A token carrying the configured tag on one of two readings, so the
    /// only thing that can keep it out of the output is the exercise.
    fn ambiguous() -> CgToken {
        cg_token(
            0,
            3,
            &[
                &["\"mun\"", "Pron", "Sg1", "Nom", "@SUBJ→"],
                &["\"mun\"", "N", "Sg", "Nom", "@OBJ→"],
            ],
        )
    }

    /// The three topics carry the same pass, so its exercise-dependent
    /// behaviour is exercised once, over the tags and reading test they all
    /// supply: any reading carrying the tag matches, and a token is
    /// unambiguous when it has exactly one reading.
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-subject-enhancer.vislcg3-subject-enhancer.process-fn+3/test]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-object-enhancer.vislcg3-object-enhancer.process-fn+3/test]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-adverbial-enhancer.vislcg3-adverbial-enhancer.process-fn+3/test]
    #[test]
    fn an_ambiguous_token_reaches_the_marking_exercises_only() {
        let tags = vec!["@SUBJ→".to_string()];

        for (mode, expected) in [
            (Mode::Mc, 0),
            (Mode::Cloze, 0),
            (Mode::Colorize, 1),
            (Mode::Click, 1),
        ] {
            let mut doc = Document::new("Mun oainnán mánáid.", PIPELINE_LANGUAGE);
            doc.cg_tokens.push(ambiguous());

            run(
                &mut doc,
                &FunctionSpec {
                    start_log: "Starting",
                    finish_log: "Finished",
                    span_class: "teaksta-Subject",
                    tags: &tags,
                    is_safe: &|t: &CgToken| t.readings.len() == 1,
                    contains_tag: &|cgr: &CgReading, tag: &str| {
                        cgr.iter().any(|rtag| rtag.as_str() == tag)
                    },
                },
                mode,
            )
            .expect("a token the exercise never reaches is not a failure");

            assert_eq!(doc.enhancements.len(), expected, "{mode:?}");
        }
    }
}
