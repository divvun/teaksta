//! The enhancement pass the tag-driven topics hold in common.
//!
//! `Vislcg3SubjectEnhancer`, `Vislcg3ObjectEnhancer`,
//! `Vislcg3AdverbialEnhancer`, `Vislcg3ConjunctionEnhancer` and
//! `Vislcg3NounSgEnhancer` walk the configured tags, then the tokens, and
//! wrap the first reading carrying the tag in a span. What genuinely differs
//! between them — the log lines, the span classes, the configured tags, how a
//! reading is tested and what the span carries beside its classes — is named
//! by [`FunctionSpec`] and supplied per topic.
//!
//! Authors: Niels Ott?, Adriane Boyd, Heli Uibo

use std::collections::HashMap;

use anyhow::Result;
use tracing::{debug, info};

use crate::enhancer::cg_enhancer::GeneratorFailure;
use crate::enhancer::cg_span::{SpanTag, TOKEN_CLASS};
use crate::server::api::Mode;
use crate::types::{CgReading, CgToken, Document, Enhancement};
use crate::util::enhancer_utils;

/// What a topic reads off the reading a token was accepted on, to hang on
/// that token's span: one `name="value"` pair per entry, in the order they
/// reach the markup.
pub type Attributes<'a> = &'a dyn Fn(&CgReading) -> Result<Vec<(&'static str, String)>>;

/// One activity's configured tag list, split the way `String.split(",")`
/// splits it: trailing empty fields are dropped, but an input the separator
/// never matches stays whole as the single element. Every tag-driven topic
/// reads its own parameter and splits it exactly this way.
pub fn split_tags(configured: &str) -> Vec<String> {
    let mut tags: Vec<String> = configured.split(',').map(str::to_string).collect();
    if configured.contains(',') {
        while tags.last().is_some_and(|tag| tag.is_empty()) {
            tags.pop();
        }
    }
    tags
}

/// Everything one tag-driven topic changes about the shared pass.
pub struct FunctionSpec<'a> {
    /// Names the topic in the opening log line.
    pub start_log: &'a str,
    /// Names the topic in the closing log line. The Java classes were edited
    /// apart over time and no longer word this alike.
    pub finish_log: &'a str,
    /// The second CSS class on every enhanced span.
    pub span_class: &'a str,
    /// A third class, built from the tag the reading matched, for the one
    /// topic that names the tag in the markup as well as the topic.
    pub tag_class: Option<&'a dyn Fn(&str) -> String>,
    /// The tags the activity configured, in the order spans are numbered.
    pub tags: &'a [String],
    /// Whether the token is unambiguous enough for the exercise types that
    /// ask a question about it.
    pub is_safe: &'a dyn Fn(&CgToken) -> bool,
    /// Whether a reading carries the tag; each topic tests this its own way.
    pub contains_tag: &'a dyn Fn(&CgReading, &str) -> bool,
    /// The attributes the span carries beside its id and classes, read off
    /// the reading the token was accepted on. Exercises on syntactic
    /// functions need neither the lemma of the CG reading nor the
    /// distractors generated from it, so only the singular-noun topic sets
    /// this; a reading it cannot read them from is dropped on its own.
    pub attributes: Option<Attributes<'a>>,
}

impl<'a> FunctionSpec<'a> {
    /// The spec of a topic that marks the token and nothing more: it names
    /// no tag in the markup and hangs no field on the span, so the three
    /// syntactic-function topics supply only what they genuinely differ in.
    pub fn plain(
        start_log: &'a str,
        finish_log: &'a str,
        span_class: &'a str,
        tags: &'a [String],
        is_safe: &'a dyn Fn(&CgToken) -> bool,
        contains_tag: &'a dyn Fn(&CgReading, &str) -> bool,
    ) -> Self {
        FunctionSpec {
            start_log,
            finish_log,
            span_class,
            tag_class: None,
            tags,
            is_safe,
            contains_tag,
            attributes: None,
        }
    }
}

/// Run one tag-driven topic's enhancement pass over `doc`, for the exercise
/// `mode` names. The body of every `Vislcg3*Enhancer::process` among the
/// five; see [`FunctionSpec`] for what each topic changes about it.
pub fn run(doc: &mut Document, spec: &FunctionSpec<'_>, mode: Mode) -> Result<()> {
    info!("{}", spec.start_log);

    // keep track of ids for each annotation class
    let mut class_counts: HashMap<String, i32> = HashMap::new();
    for con_t in spec.tags {
        class_counts.insert(con_t.clone(), 0);
        debug!("Tag: {}", con_t);
    }

    // iterating over the configured tags instead of the class-count key set
    // because it is important to control the order in which spans are enhanced

    for con_t in spec.tags {
        let tag_class = spec.tag_class.map(|class_of| class_of(con_t));
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

                // the fields the exercise asks the span to carry, read off
                // this reading
                let attributes = match spec.attributes {
                    Some(read) => match read(reading) {
                        Ok(attributes) => attributes,
                        // the transducer seam failing is the deployment's
                        // fault, not this reading's: skipping it would answer
                        // the request with an exercise whose questions carry
                        // no answers to choose between, so it is raised
                        Err(e) if e.is::<GeneratorFailure>() => return Err(e),
                        // a reading whose base form or generator input cannot
                        // be built is dropped on its own, not together with
                        // the rest of the document
                        Err(e) => {
                            debug!("no exercise fields for {:?}: {}", reading, e);
                            continue;
                        }
                    },
                    None => Vec::new(),
                };

                // increment id
                let new_id = class_counts[con_t] + 1;
                let id = enhancer_utils::get_id("teaksta-span-", con_t, new_id);
                let mut span_tag = match &tag_class {
                    Some(tag_class) => SpanTag::new(id, &[TOKEN_CLASS, spec.span_class, tag_class]),
                    None => SpanTag::new(id, &[TOKEN_CLASS, spec.span_class]),
                };
                for (name, value) in &attributes {
                    span_tag.add_attribute(name, value);
                }
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

    /// A spec that changes nothing about the shared pass, so a test naming
    /// one field says what it is testing.
    fn spec<'a>(tags: &'a [String], span_class: &'a str) -> FunctionSpec<'a> {
        FunctionSpec::plain(
            "Starting",
            "Finished",
            span_class,
            tags,
            &|t: &CgToken| t.readings.len() == 1,
            &|cgr: &CgReading, tag: &str| cgr.iter().any(|rtag| rtag.as_str() == tag),
        )
    }

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

    /// The five topics carry the same pass, so its exercise-dependent
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

            run(&mut doc, &spec(&tags, "teaksta-Subject"), mode)
                .expect("a token the exercise never reaches is not a failure");

            assert_eq!(doc.enhancements.len(), expected, "{mode:?}");
        }
    }

    /// The two hooks the shared pass carries for one topic each: the third
    /// class the conjunction topic names its tag in, and the attributes the
    /// singular-noun topic reads off the accepted reading. A reading the
    /// attributes cannot be read from is skipped without failing the pass.
    #[test]
    fn the_tag_class_and_attributes_reach_the_span() {
        let tags = vec!["@SUBJ→".to_string()];
        let mut doc = Document::new("Mun oainnán mánáid.", PIPELINE_LANGUAGE);
        doc.cg_tokens
            .push(cg_token(0, 3, &[&["\"mun\"", "@SUBJ→"]]));
        doc.cg_tokens
            .push(cg_token(4, 11, &[&["\"skip\"", "@SUBJ→"]]));

        run(
            &mut doc,
            &FunctionSpec {
                tag_class: Some(&|tag: &str| format!("teaksta-{tag}")),
                attributes: Some(&|cgr: &CgReading| match cgr[0].as_str() {
                    "\"skip\"" => Err(anyhow::anyhow!("no base form")),
                    lemma => Ok(vec![("lemma", lemma.replace('"', ""))]),
                }),
                ..spec(&tags, "teaksta-Subject")
            },
            Mode::Colorize,
        )
        .expect("a reading without fields is not a failure");

        assert_eq!(doc.enhancements.len(), 1);
        assert_eq!(
            doc.enhancements[0].enhance_start,
            "<span id=\"teaksta-span-@SUBJ→-1\" \
             class=\"teaksta-token teaksta-Subject teaksta-@SUBJ→\" lemma=\"mun\">"
        );
    }

    /// A transducer the deployment cannot reach is not one reading the topic
    /// cannot use: skipping it would answer the request with an exercise
    /// whose questions carry nothing to answer them with.
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-sg-enhancer.vislcg3-noun-sg-enhancer.process-fn+5/test]
    #[test]
    fn a_seam_failure_ends_the_pass() {
        let tags = vec!["@SUBJ→".to_string()];

        let mut doc = Document::new("Mun oainnán mánáid.", PIPELINE_LANGUAGE);
        doc.cg_tokens
            .push(cg_token(0, 3, &[&["\"mun\"", "@SUBJ→"]]));

        let err = run(
            &mut doc,
            &FunctionSpec {
                attributes: Some(&|_: &CgReading| {
                    Err(GeneratorFailure("the generator is not set".to_string()).into())
                }),
                ..spec(&tags, "teaksta-Subject")
            },
            Mode::Colorize,
        )
        .expect_err("a seam failure reaches the caller");

        assert_eq!(err.to_string(), "the generator is not set");
        assert!(doc.enhancements.is_empty());
    }
}
