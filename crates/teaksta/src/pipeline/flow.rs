//! The two pipelines a topic runs, built as concrete Rust stages.
//!
//! Which stages run and in which order is architecture, not configuration.
//! Every topic preprocesses identically — relevance, tokenizer, sentence
//! detection, HTML sentences, constraint grammar — and postprocesses the same
//! shape: the generic token enhancer that wraps every word, then the one
//! enhancer that marks the topic's own hits. What varies between topics is
//! the tags those two enhancers read, and that is what
//! [`crate::server::registry`] carries.
//!
//! The order is the order the stages need: nothing can be tokenised before
//! the relevant text is known, and nothing can be enhanced before the
//! constraint grammar has read the sentences.

use std::collections::HashMap;

use anyhow::Result;

use crate::enhancer::adverbial::Vislcg3AdverbialEnhancer;
use crate::enhancer::con_neg::Vislcg3ConNegEnhancer;
use crate::enhancer::conjunction::Vislcg3ConjunctionEnhancer;
use crate::enhancer::infinite_verb::Vislcg3InfiniteVerbEnhancer;
use crate::enhancer::noun::Vislcg3NounEnhancer;
use crate::enhancer::noun_pl::Vislcg3NounPlEnhancer;
use crate::enhancer::noun_sg::Vislcg3NounSgEnhancer;
use crate::enhancer::object::Vislcg3ObjectEnhancer;
use crate::enhancer::subject::Vislcg3SubjectEnhancer;
use crate::enhancer::token::TokenEnhancer;
use crate::enhancer::verb_conjugation::Vislcg3VerbConjugationEnhancer;
use crate::pipeline::relevance::GenericRelevanceAnnotator;
use crate::pipeline::sentences::{
    HtmlSentenceAnnotator, PlainTextSentenceAnnotation, SentenceDetector,
};
use crate::pipeline::tokenizer::GiellateknoTokenizer;
use crate::pipeline::vislcg3::Vislcg3Annotator;
use crate::server::api::Mode;
use crate::server::registry::{Enhancer, TopicConfig};
use crate::types::Document;

/// One stage of a flow.
pub enum Stage {
    Relevance(GenericRelevanceAnnotator),
    Tokenizer(GiellateknoTokenizer),
    SentenceDetector(SentenceDetector),
    HtmlSentences(HtmlSentenceAnnotator),
    Vislcg3(Box<Vislcg3Annotator>),
    Token(TokenEnhancer),
    Noun(Vislcg3NounEnhancer),
    NounSg(Box<Vislcg3NounSgEnhancer>),
    NounPl(Vislcg3NounPlEnhancer),
    VerbConjugation(Vislcg3VerbConjugationEnhancer),
    ConNeg(Vislcg3ConNegEnhancer),
    InfiniteVerb(Vislcg3InfiniteVerbEnhancer),
    Adverbial(Vislcg3AdverbialEnhancer),
    Conjunction(Vislcg3ConjunctionEnhancer),
    Object(Vislcg3ObjectEnhancer),
    Subject(Vislcg3SubjectEnhancer),
}

impl Stage {
    /// The class this stage marks a hit of its topic with, for the stages that
    /// stand for a topic. There is one scheme: the class is `teaksta-` and the
    /// name the activity registry serves that topic under, which is what the
    /// client derives the class from. The stages that annotate the document,
    /// and the generic token enhancer — which marks a hit for whatever the
    /// topic's own stage does not, under a class of its own — name no topic.
    pub fn topic_span_class(&self) -> Option<&'static str> {
        match self {
            Stage::Noun(_) => Some(Vislcg3NounEnhancer::SPAN_CLASS),
            Stage::NounSg(_) => Some(Vislcg3NounSgEnhancer::SPAN_CLASS),
            Stage::NounPl(_) => Some(Vislcg3NounPlEnhancer::SPAN_CLASS),
            Stage::VerbConjugation(_) => Some(Vislcg3VerbConjugationEnhancer::SPAN_CLASS),
            Stage::ConNeg(_) => Some(Vislcg3ConNegEnhancer::SPAN_CLASS),
            Stage::InfiniteVerb(_) => Some(Vislcg3InfiniteVerbEnhancer::SPAN_CLASS),
            Stage::Adverbial(_) => Some(Vislcg3AdverbialEnhancer::SPAN_CLASS),
            Stage::Conjunction(_) => Some(Vislcg3ConjunctionEnhancer::SPAN_CLASS),
            Stage::Object(_) => Some(Vislcg3ObjectEnhancer::SPAN_CLASS),
            Stage::Subject(_) => Some(Vislcg3SubjectEnhancer::SPAN_CLASS),
            Stage::Relevance(_)
            | Stage::Tokenizer(_)
            | Stage::SentenceDetector(_)
            | Stage::HtmlSentences(_)
            | Stage::Vislcg3(_)
            | Stage::Token(_) => None,
        }
    }
}

/// The enhancer a topic's postprocessing flow ends in, built from the topic's
/// own tag list.
///
/// Each enhancer reads its tags under a parameter name of its own, so the
/// one tag list a topic carries is handed over under the name that enhancer
/// expects.
fn topic_stage(config: &TopicConfig) -> Result<Stage> {
    let tags = Some(config.tags.as_str());
    let named = |parameter: &str| HashMap::from([(parameter.to_string(), config.tags.clone())]);

    let stage = match config.enhancer {
        Enhancer::Noun => Stage::Noun(Vislcg3NounEnhancer::new(tags)?),
        Enhancer::NounSg => Stage::NounSg(Box::new(Vislcg3NounSgEnhancer::new(tags)?)),
        Enhancer::NounPl => Stage::NounPl(Vislcg3NounPlEnhancer::new(tags)?),
        Enhancer::VerbConjugation => {
            Stage::VerbConjugation(Vislcg3VerbConjugationEnhancer::new(tags)?)
        }
        Enhancer::ConNeg => Stage::ConNeg(Vislcg3ConNegEnhancer::new(tags)?),
        Enhancer::InfiniteVerb => Stage::InfiniteVerb(Vislcg3InfiniteVerbEnhancer::new(tags)?),
        Enhancer::Adverbial => Stage::Adverbial(Vislcg3AdverbialEnhancer::new(&named("AdvTags"))?),
        Enhancer::Conjunction => {
            Stage::Conjunction(Vislcg3ConjunctionEnhancer::new(&named("conjunctionTags"))?)
        }
        Enhancer::Object => Stage::Object(Vislcg3ObjectEnhancer::new(&named("ObjTags"))?),
        Enhancer::Subject => Stage::Subject(Vislcg3SubjectEnhancer::new(&named("SubjTags"))?),
    };
    Ok(stage)
}

/// The generic enhancer that wraps every word of the page, so the topic's own
/// hits have something to be picked out from.
fn token_stage(config: &TopicConfig) -> Result<TokenEnhancer> {
    TokenEnhancer::new(&HashMap::from([
        ("Tags".to_string(), config.token_tags.clone()),
        (
            "UseLemmaFilter".to_string(),
            config.use_lemma_filter.to_string(),
        ),
    ]))
}

/// A pipeline, ready to run over a document.
#[derive(Default)]
pub struct Flow {
    stages: Vec<Stage>,
}

impl Flow {
    /// The preprocessing flow, which every topic shares: find the text worth
    /// reading, tokenise it, find its sentences, map those onto the page, and
    /// run the constraint grammar over them.
    ///
    /// It takes no parameters. Nothing in it varies with the topic, and its
    /// output is what the analysis cache holds — one cached document answers
    /// every topic's request for the same page.
    pub fn preprocessor() -> Self {
        Flow {
            stages: vec![
                Stage::Relevance(GenericRelevanceAnnotator),
                Stage::Tokenizer(GiellateknoTokenizer),
                Stage::SentenceDetector(SentenceDetector),
                Stage::HtmlSentences(HtmlSentenceAnnotator),
                Stage::Vislcg3(Box::default()),
            ],
        }
    }

    /// The postprocessing flow for one topic: the generic token enhancer,
    /// then the enhancer that marks this topic's hits.
    ///
    /// The token enhancer runs first because the topic's enhancer adds its
    /// own class to spans the token enhancer already wrote.
    pub fn postprocessor(config: &TopicConfig) -> Result<Self> {
        Ok(Flow {
            stages: vec![Stage::Token(token_stage(config)?), topic_stage(config)?],
        })
    }

    /// A flow of exactly these stages, for tests that need a pipeline whose
    /// stages load no models.
    #[cfg(test)]
    pub(crate) fn of_stages(stages: Vec<Stage>) -> Self {
        Flow { stages }
    }

    /// The classes the stages of this flow mark their topic's hits with. A
    /// postprocessing flow carries exactly one; a preprocessing flow none.
    pub fn topic_span_classes(&self) -> Vec<&'static str> {
        self.stages
            .iter()
            .filter_map(Stage::topic_span_class)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.stages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stages.is_empty()
    }

    /// Runs every stage over `doc` in flow order, for the exercise `mode`
    /// names. The plain-text sentence list is the one value a stage hands to
    /// a later stage rather than to the document, so it is carried here.
    ///
    /// Only the postprocessing enhancers vary with the exercise; the stages
    /// that annotate the document take the text as it stands and are handed
    /// no mode.
    pub fn run(&self, doc: &mut Document, mode: Mode) -> Result<()> {
        let mut sentences: Vec<PlainTextSentenceAnnotation> = Vec::new();
        for stage in &self.stages {
            match stage {
                Stage::Relevance(s) => s.process(doc)?,
                Stage::Tokenizer(s) => s.process(doc)?,
                Stage::SentenceDetector(s) => sentences = s.process(doc)?,
                Stage::HtmlSentences(s) => s.process(doc, &sentences)?,
                Stage::Vislcg3(s) => s.process(doc)?,
                Stage::Token(s) => s.process(doc)?,
                Stage::Noun(s) => s.process(doc, mode)?,
                Stage::NounSg(s) => s.process(doc, mode)?,
                Stage::NounPl(s) => s.process(doc, mode)?,
                Stage::VerbConjugation(s) => s.process(doc, mode)?,
                Stage::ConNeg(s) => s.process(doc, mode)?,
                Stage::InfiniteVerb(s) => s.process(doc, mode)?,
                Stage::Adverbial(s) => s.process(doc, mode)?,
                Stage::Conjunction(s) => s.process(doc, mode)?,
                Stage::Object(s) => s.process(doc, mode)?,
                Stage::Subject(s) => s.process(doc, mode)?,
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A topic carrying whatever the caller wants to vary, with the rest of
    /// the fields at a value no test reads.
    fn topic(enhancer: Enhancer, tags: &str, token_tags: &str) -> TopicConfig {
        TopicConfig {
            name: "Topic".to_string(),
            label: "Fáddá".to_string(),
            enabled: true,
            enhancer,
            tags: tags.to_string(),
            token_tags: token_tags.to_string(),
            use_lemma_filter: false,
        }
    }

    #[test]
    fn the_preprocessor_is_five_shared_stages() {
        let flow = Flow::preprocessor();

        assert_eq!(flow.len(), 5);
        assert!(!flow.is_empty());
        assert!(matches!(flow.stages.first(), Some(Stage::Relevance(_))));
        let Some(Stage::Vislcg3(annotator)) = flow.stages.last() else {
            panic!("the flow ends in the CG annotator");
        };
        assert_eq!(annotator.cg_sentence_boundary_token, ".");
        // Nothing in it stands for a topic, so a cached preprocessing output
        // answers every topic's request for the same page.
        assert!(flow.topic_span_classes().is_empty());
    }

    #[test]
    fn a_postprocessor_is_token_then_topic() {
        let flow = Flow::postprocessor(&topic(Enhancer::Noun, "Sg Nom", "N"))
            .expect("the noun topic builds");

        assert_eq!(flow.len(), 2);
        let Some(Stage::Token(token)) = flow.stages.first() else {
            panic!("the flow opens with the token enhancer");
        };
        assert_eq!(token.tags, vec!["N".to_string()]);
        assert!(!token.use_lemma_filter);
        assert_eq!(
            flow.topic_span_classes(),
            vec![Vislcg3NounEnhancer::SPAN_CLASS]
        );
    }

    #[test]
    fn every_enhancer_marks_exactly_one_class() {
        for enhancer in Enhancer::ALL {
            let flow = Flow::postprocessor(&topic(enhancer, "A,B", "N"))
                .unwrap_or_else(|e| panic!("{enhancer:?} rejected its tag list: {e:#}"));

            assert_eq!(
                flow.topic_span_classes(),
                vec![enhancer.span_class()],
                "{enhancer:?} built a stage marking another topic's class"
            );
        }
    }

    /// Each enhancer reads its tags under a parameter name of its own, and
    /// refuses to build without it. The one tag list a topic carries therefore
    /// has to reach each of them under a different name, which is what makes
    /// the mapping in `topic_stage` load-bearing rather than decorative.
    #[test]
    fn each_enhancer_reads_its_own_parameter() {
        use crate::enhancer::object::Vislcg3ObjectEnhancer;

        assert!(
            Vislcg3ObjectEnhancer::new(&HashMap::from([(
                "SubjTags".to_string(),
                "OBJ".to_string(),
            )]))
            .is_err(),
            "the object enhancer built from another enhancer's parameter"
        );
        assert!(
            Vislcg3ObjectEnhancer::new(&HashMap::from([(
                "ObjTags".to_string(),
                "OBJ".to_string(),
            )]))
            .is_ok()
        );
        // And the token enhancer refuses to build without its own two.
        assert!(TokenEnhancer::new(&HashMap::new()).is_err());
    }

    #[test]
    fn the_token_enhancer_carries_the_lemma_filter() {
        let mut config = topic(Enhancer::Noun, "Sg Nom", "N,V");
        config.use_lemma_filter = true;
        let flow = Flow::postprocessor(&config).expect("the topic builds");

        let Some(Stage::Token(token)) = flow.stages.first() else {
            panic!("the flow opens with the token enhancer");
        };
        assert_eq!(token.tags, vec!["N".to_string(), "V".to_string()]);
        assert!(token.use_lemma_filter);
    }

    #[test]
    fn an_empty_flow_runs_and_changes_nothing() {
        let flow = Flow::default();
        let mut doc = Document::new("<p>Mun oidnen viesu.</p>");

        flow.run(&mut doc, Mode::Colorize)
            .expect("an empty flow is a no-op");

        assert!(flow.is_empty());
        assert!(doc.enhancements.is_empty());
        assert!(doc.tokens.is_empty());
    }
}
