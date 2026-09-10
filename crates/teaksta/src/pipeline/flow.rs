//! The fixed flow of an analysis-engine descriptor, executed as concrete
//! Rust stages.
//!
//! UIMA resolved each `<delegateAnalysisEngine>` import to a descriptor and
//! that descriptor to an annotator class name, which the framework loaded
//! reflectively. Every one of those annotator classes is a type in this
//! crate, so the import chain carries no information the binary needs: the
//! delegate key from `<fixedFlow>` names the stage directly, and the
//! aggregate's configuration-parameter settings — the descriptor defaults
//! with the activity's `server-cfg` entries laid over them — supply each
//! stage's parameters.
//!
//! Stage order is therefore still the descriptor's; only the indirection
//! between a delegate key and the code it runs is gone.

use std::collections::HashMap;

use anyhow::{Result, bail};

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
    HtmlSentenceAnnotator, OpenNlpSentenceDetector, PlainTextSentenceAnnotation,
};
use crate::pipeline::tokenizer::GiellateknoTokenizer;
use crate::pipeline::vislcg3::Vislcg3Annotator;
use crate::server::api::Mode;
use crate::types::Document;

/// The parameter table a stage is initialised from: the aggregate
/// descriptor's `configurationParameterSettings`, rendered as strings.
pub type Parameters = HashMap<String, String>;

/// One delegate of a fixed flow.
pub enum Stage {
    Relevance(GenericRelevanceAnnotator),
    Tokenizer(GiellateknoTokenizer),
    SentenceDetector(OpenNlpSentenceDetector),
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

/// The delegate keys the shipped `sme` descriptors use in their fixed flows.
/// A key outside this set names a delegate with no counterpart here.
pub fn stage_named(key: &str, parameters: &Parameters) -> Result<Stage> {
    let stage = match key {
        "GenericRelevanceAnnotator" => Stage::Relevance(GenericRelevanceAnnotator::new()),
        "GiellateknoTokenizer" => {
            let mut tokenizer = GiellateknoTokenizer::new();
            tokenizer.initialize()?;
            Stage::Tokenizer(tokenizer)
        }
        "OpenNlpSentenceDetector" => {
            let mut detector = OpenNlpSentenceDetector::new();
            detector.initialize()?;
            Stage::SentenceDetector(detector)
        }
        "HTMLSentenceAnnotator" => Stage::HtmlSentences(HtmlSentenceAnnotator::new()),
        "vislcg3Annotator" => Stage::Vislcg3(Box::new(Vislcg3Annotator::new())),
        "TokenEnhancer" => Stage::Token(token_enhancer(parameters)?),
        "vislcg3NounEnhancer" => {
            Stage::Noun(Vislcg3NounEnhancer::new(tag_list(parameters, "NTags"))?)
        }
        "vislcg3NounSgEnhancer" => Stage::NounSg(Box::new(Vislcg3NounSgEnhancer::new(tag_list(
            parameters, "NSgTags",
        ))?)),
        "vislcg3NounPlEnhancer" => {
            Stage::NounPl(Vislcg3NounPlEnhancer::new(tag_list(parameters, "NPlTags"))?)
        }
        "vislcg3VerbConjugationEnhancer" => Stage::VerbConjugation(
            Vislcg3VerbConjugationEnhancer::new(tag_list(parameters, "finverbTags"))?,
        ),
        "vislcg3ConNegEnhancer" => Stage::ConNeg(Vislcg3ConNegEnhancer::new(tag_list(
            parameters,
            "connegTags",
        ))?),
        "vislcg3InfiniteVerbEnhancer" => Stage::InfiniteVerb(Vislcg3InfiniteVerbEnhancer::new(
            tag_list(parameters, "infiniteverbTags"),
        )?),
        "vislcg3AdverbialEnhancer" => {
            let mut enhancer = Vislcg3AdverbialEnhancer::new();
            enhancer.initialize(parameters)?;
            Stage::Adverbial(enhancer)
        }
        "vislcg3ConjunctionEnhancer" => {
            let mut enhancer = Vislcg3ConjunctionEnhancer::new();
            enhancer.initialize(parameters)?;
            Stage::Conjunction(enhancer)
        }
        "vislcg3ObjectEnhancer" => {
            let mut enhancer = Vislcg3ObjectEnhancer::new();
            enhancer.initialize(parameters)?;
            Stage::Object(enhancer)
        }
        "vislcg3SubjectEnhancer" => {
            let mut enhancer = Vislcg3SubjectEnhancer::new();
            enhancer.initialize(parameters)?;
            Stage::Subject(enhancer)
        }
        other => bail!("no annotator is registered for delegate {other:?}"),
    };
    Ok(stage)
}

fn tag_list<'a>(parameters: &'a Parameters, key: &str) -> Option<&'a str> {
    parameters.get(key).map(String::as_str)
}

/// `UseLemmaFilter` is declared on the delegate rather than on the
/// aggregate, and delegate imports are not followed, so the delegate's own
/// default stands in when the aggregate does not carry the parameter.
fn token_enhancer(parameters: &Parameters) -> Result<TokenEnhancer> {
    let mut context = parameters.clone();
    context
        .entry("UseLemmaFilter".to_string())
        .or_insert_with(|| "false".to_string());
    let mut enhancer = TokenEnhancer::new();
    enhancer.initialize(&context)?;
    Ok(enhancer)
}

/// A descriptor's fixed flow, ready to run over a document.
#[derive(Default)]
pub struct Flow {
    stages: Vec<Stage>,
}

impl Flow {
    /// Builds one stage per delegate key, in flow order, initialising each
    /// from `parameters`.
    pub fn new(fixed_flow: &[String], parameters: &Parameters) -> Result<Self> {
        let mut stages = Vec::with_capacity(fixed_flow.len());
        for key in fixed_flow {
            stages.push(stage_named(key, parameters)?);
        }
        Ok(Flow { stages })
    }

    pub fn len(&self) -> usize {
        self.stages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stages.is_empty()
    }

    /// Runs every stage over `cas` in flow order, for the exercise `mode`
    /// names. The plain-text sentence list is the one value a stage hands to
    /// a later stage rather than to the document, so it is carried here.
    ///
    /// Only the postprocessing enhancers vary with the exercise; the stages
    /// that annotate the document take the text as it stands and are handed
    /// no mode.
    pub fn run(&self, cas: &mut Document, mode: Mode) -> Result<()> {
        let mut sentences: Vec<PlainTextSentenceAnnotation> = Vec::new();
        for stage in &self.stages {
            match stage {
                Stage::Relevance(s) => s.process(cas)?,
                Stage::Tokenizer(s) => s.process(cas)?,
                Stage::SentenceDetector(s) => sentences = s.process(cas)?,
                Stage::HtmlSentences(s) => s.process(cas, &sentences)?,
                Stage::Vislcg3(s) => s.process(cas)?,
                Stage::Token(s) => s.process(cas)?,
                Stage::Noun(s) => s.process(cas, mode)?,
                Stage::NounSg(s) => s.process(cas, mode)?,
                Stage::NounPl(s) => s.process(cas, mode)?,
                Stage::VerbConjugation(s) => s.process(cas, mode)?,
                Stage::ConNeg(s) => s.process(cas, mode)?,
                Stage::InfiniteVerb(s) => s.process(cas, mode)?,
                Stage::Adverbial(s) => s.process(cas, mode)?,
                Stage::Conjunction(s) => s.process(cas, mode)?,
                Stage::Object(s) => s.process(cas, mode)?,
                Stage::Subject(s) => s.process(cas, mode)?,
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PIPELINE_LANGUAGE;

    fn parameters(pairs: &[(&str, &str)]) -> Parameters {
        pairs
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    }

    #[test]
    fn the_preprocessor_flow_builds_every_stage() {
        let flow = Flow::new(
            &[
                "GenericRelevanceAnnotator".to_string(),
                "GiellateknoTokenizer".to_string(),
                "OpenNlpSentenceDetector".to_string(),
                "HTMLSentenceAnnotator".to_string(),
                "vislcg3Annotator".to_string(),
            ],
            &parameters(&[]),
        )
        .expect("the shipped preprocessor flow");

        assert_eq!(flow.len(), 5);
        assert!(!flow.is_empty());
        let Some(Stage::Vislcg3(annotator)) = flow.stages.last() else {
            panic!("the flow ends in the CG annotator");
        };
        assert_eq!(annotator.cg_sentence_boundary_token, ".");
    }

    #[test]
    fn each_topic_delegate_takes_its_own_parameter() {
        for (key, parameter) in [
            ("vislcg3NounEnhancer", "NTags"),
            ("vislcg3NounSgEnhancer", "NSgTags"),
            ("vislcg3NounPlEnhancer", "NPlTags"),
            ("vislcg3VerbConjugationEnhancer", "finverbTags"),
            ("vislcg3ConNegEnhancer", "connegTags"),
            ("vislcg3InfiniteVerbEnhancer", "infiniteverbTags"),
            ("vislcg3AdverbialEnhancer", "AdvTags"),
            ("vislcg3ConjunctionEnhancer", "conjunctionTags"),
            ("vislcg3ObjectEnhancer", "ObjTags"),
            ("vislcg3SubjectEnhancer", "SubjTags"),
        ] {
            assert!(
                stage_named(key, &parameters(&[(parameter, "A,B")])).is_ok(),
                "{key} rejected its own parameter"
            );
            assert!(
                stage_named(key, &parameters(&[])).is_err(),
                "{key} accepted a missing parameter"
            );
        }
    }

    #[test]
    fn the_token_enhancer_defaults_the_lemma_filter() {
        let Ok(Stage::Token(enhancer)) =
            stage_named("TokenEnhancer", &parameters(&[("Tags", "N,V")]))
        else {
            panic!("the token enhancer builds from Tags alone");
        };

        assert_eq!(enhancer.tags, vec!["N".to_string(), "V".to_string()]);
        assert!(!enhancer.use_lemma_filter);
    }

    #[test]
    fn an_unknown_delegate_key_is_rejected() {
        let Err(err) = stage_named("MaltParser", &parameters(&[])) else {
            panic!("a delegate with no counterpart must be rejected");
        };

        assert_eq!(
            err.to_string(),
            "no annotator is registered for delegate \"MaltParser\""
        );
    }

    #[test]
    fn an_empty_flow_runs_and_changes_nothing() {
        let flow = Flow::new(&[], &parameters(&[])).expect("an empty flow");
        let mut cas = Document::new("<p>Mun oidnen viesu.</p>", PIPELINE_LANGUAGE);

        flow.run(&mut cas, Mode::Colorize)
            .expect("an empty flow is a no-op");

        assert!(flow.is_empty());
        assert!(cas.enhancements.is_empty());
        assert!(cas.tokens.is_empty());
    }
}
