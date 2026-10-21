//! The topic registry: what topics this deployment offers, and the pipeline
//! pair each of them runs.
//!
//! Topics are configuration, not code and not a directory tree. The canonical
//! list is `crates/teaksta/topics.toml`, compiled into the binary, so a build
//! carries a working registry with nothing beside it. A deployment that wants
//! to retune without a rebuild names a file in `TEAKSTA_TOPICS` and that file
//! is read instead — whole, not merged, so what is served is what one file
//! says. A file that will not parse fails the boot naming itself, because a
//! server that starts with half a registry serves a topic list nobody wrote.
//!
//! The pipelines are not configuration. Every topic preprocesses the same way
//! and postprocesses the same shape — the generic token enhancer, then the
//! topic's own — so the flows are built in [`crate::pipeline::flow`] from the
//! parameters a topic carries rather than described anywhere.
//!
//! Both flows are built once, at startup, and shared by every request: a
//! request pays for no model load, and two first requests cannot each build
//! one.

use std::collections::BTreeMap;

use anyhow::{Context as _, Result, bail};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::enhancer::adverbial::Vislcg3AdverbialEnhancer;
use crate::enhancer::con_neg::Vislcg3ConNegEnhancer;
use crate::enhancer::conjunction::Vislcg3ConjunctionEnhancer;
use crate::enhancer::infinite_verb::Vislcg3InfiniteVerbEnhancer;
use crate::enhancer::noun::Vislcg3NounEnhancer;
use crate::enhancer::noun_pl::Vislcg3NounPlEnhancer;
use crate::enhancer::noun_sg::Vislcg3NounSgEnhancer;
use crate::enhancer::object::Vislcg3ObjectEnhancer;
use crate::enhancer::subject::Vislcg3SubjectEnhancer;
use crate::enhancer::verb_conjugation::Vislcg3VerbConjugationEnhancer;
use crate::pipeline::flow::Flow;

/// The compiled-in registry, and what a `TEAKSTA_TOPICS` file replaces.
const BUILT_IN: &str = include_str!("../../topics.toml");

/// Which enhancer drives a topic. The name in the file names the enhancer
/// itself: there is no indirection left to configure, so there is nothing
/// else for a key to mean.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Enhancer {
    Adverbial,
    ConNeg,
    Conjunction,
    InfiniteVerb,
    Noun,
    NounPl,
    NounSg,
    Object,
    Subject,
    VerbConjugation,
}

impl Enhancer {
    /// Every enhancer a topic may name, so a test can walk them all and a
    /// new one cannot be added without being accounted for.
    pub const ALL: [Enhancer; 10] = [
        Enhancer::Adverbial,
        Enhancer::ConNeg,
        Enhancer::Conjunction,
        Enhancer::InfiniteVerb,
        Enhancer::Noun,
        Enhancer::NounPl,
        Enhancer::NounSg,
        Enhancer::Object,
        Enhancer::Subject,
        Enhancer::VerbConjugation,
    ];

    /// The class this enhancer marks its hits with. There is one scheme —
    /// `teaksta-` and the name the registry serves the topic under — and the
    /// client derives the class it looks for from the name it was served, so
    /// a topic naming an enhancer whose class is not its own name is inert in
    /// every exercise.
    pub fn span_class(self) -> &'static str {
        match self {
            Enhancer::Adverbial => Vislcg3AdverbialEnhancer::SPAN_CLASS,
            Enhancer::ConNeg => Vislcg3ConNegEnhancer::SPAN_CLASS,
            Enhancer::Conjunction => Vislcg3ConjunctionEnhancer::SPAN_CLASS,
            Enhancer::InfiniteVerb => Vislcg3InfiniteVerbEnhancer::SPAN_CLASS,
            Enhancer::Noun => Vislcg3NounEnhancer::SPAN_CLASS,
            Enhancer::NounPl => Vislcg3NounPlEnhancer::SPAN_CLASS,
            Enhancer::NounSg => Vislcg3NounSgEnhancer::SPAN_CLASS,
            Enhancer::Object => Vislcg3ObjectEnhancer::SPAN_CLASS,
            Enhancer::Subject => Vislcg3SubjectEnhancer::SPAN_CLASS,
            Enhancer::VerbConjugation => Vislcg3VerbConjugationEnhancer::SPAN_CLASS,
        }
    }
}

/// One topic as `topics.toml` declares it.
///
/// `deny_unknown_fields`: a key this build does not read is a knob the author
/// believes in and the server ignores, which is worse than a boot failure
/// naming it.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopicConfig {
    /// The registry name, which is also what the topic's enhancer builds its
    /// hit class from.
    pub name: String,
    /// The North Sámi name the learner is shown.
    pub label: String,
    pub enabled: bool,
    pub enhancer: Enhancer,
    /// The topic enhancer's own tag list, comma separated.
    pub tags: String,
    /// The generic token enhancer's tag list, comma separated.
    pub token_tags: String,
    /// Whether the generic token enhancer filters on the presence of a lemma.
    #[serde(default)]
    pub use_lemma_filter: bool,
    /// The ready-made texts the topic offers, in the order they are shown.
    #[serde(default)]
    pub text: Vec<SampleText>,
}

/// What kind of text a ready-made text is: the four kinds the webapp this
/// replaces described its recommended pages with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Genre {
    Story,
    Folktale,
    Factual,
    Textbook,
}

/// The tense a text's verbs are in, for a topic whose learner chooses a text
/// by it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Tense {
    Present,
    Past,
}

/// One ready-made text a topic offers: a page elsewhere, named here so a
/// learner with no page of their own in mind has one to start from.
// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.activities-fn+3]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SampleText {
    pub title: String,
    pub url: String,
    pub genre: Genre,
    /// About how many words the text runs to.
    pub words: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tense: Option<Tense>,
}

/// The file as a whole.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct TopicFile {
    #[serde(default)]
    topic: Vec<TopicConfig>,
}

/// One topic as `/api/activities` serves it.
#[derive(Debug, Clone, Serialize)]
pub struct Topic {
    pub name: String,
    pub label: String,
    pub enabled: bool,
    pub texts: Vec<SampleText>,
}

/// One topic's pipeline pair.
struct Pipelines {
    pre: Flow,
    post: Flow,
}

/// Every topic this deployment offers, with the flows that serve it.
pub struct Registry {
    /// Keyed by name, so the registry is ordered ascending by name however
    /// the file was written. That order is the one `/api/activities` serves
    /// and the one the client shows.
    pipelines: BTreeMap<String, Pipelines>,
    topics: Vec<Topic>,
}

impl Registry {
    /// Reads the compiled-in registry, or the file `TEAKSTA_TOPICS` names.
    pub fn from_config(override_file: Option<&std::path::Path>) -> Result<Self> {
        let (source, text) = match override_file {
            Some(path) => {
                let text = std::fs::read_to_string(path)
                    .with_context(|| format!("reading the topics file {}", path.display()))?;
                (path.display().to_string(), text)
            }
            None => ("the compiled-in topics".to_string(), BUILT_IN.to_string()),
        };

        Registry::parse(&text).with_context(|| format!("loading topics from {source}"))
    }

    /// Builds the registry from the text of a topics file.
    pub fn parse(text: &str) -> Result<Self> {
        let file: TopicFile = toml::from_str(text)?;
        if file.topic.is_empty() {
            bail!("no topics are declared, so the server would offer nothing");
        }

        let mut pipelines: BTreeMap<String, Pipelines> = BTreeMap::new();
        let mut topics: Vec<Topic> = Vec::new();

        for config in &file.topic {
            if config.name.is_empty() {
                bail!("a topic carries no name");
            }
            if pipelines.contains_key(&config.name) {
                bail!("the topic {} is declared twice", config.name);
            }
            // An enhancer splits its tag list on commas and keeps what that
            // yields, so an empty list is one empty tag — and every reading
            // contains the empty string. The topic would mark the whole page
            // as a hit rather than nothing, which is worse than either. No
            // enhancer refuses it on its own, so it is refused here.
            if config.tags.trim().is_empty() {
                bail!("the topic {} declares no tags to mark", config.name);
            }
            // A text is a link a learner follows, so one that could never be
            // fetched is refused here rather than offered.
            for text in &config.text {
                if text.title.trim().is_empty() {
                    bail!("the topic {} offers a text with no title", config.name);
                }
                if !text.url.starts_with("https://") && !text.url.starts_with("http://") {
                    bail!(
                        "the topic {} offers {} at {}, which is not a web address",
                        config.name,
                        text.title,
                        text.url
                    );
                }
            }

            let pre = Flow::preprocessor();
            let post = Flow::postprocessor(config)
                .with_context(|| format!("building the {} pipeline", config.name))?;
            debug!(
                "Topic {}: {} preprocessing stages, {} postprocessing",
                config.name,
                pre.len(),
                post.len()
            );

            pipelines.insert(config.name.clone(), Pipelines { pre, post });
        }

        // The served list follows the map, so it is ascending by name whatever
        // order the file was written in.
        topics.extend(file.topic.iter().map(|config| Topic {
            name: config.name.clone(),
            label: config.label.clone(),
            enabled: config.enabled,
            texts: config.text.clone(),
        }));
        topics.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(Registry { pipelines, topics })
    }

    /// The topics this deployment offers, ascending by name.
    pub fn topics(&self) -> &[Topic] {
        &self.topics
    }

    pub fn knows(&self, name: &str) -> bool {
        self.pipelines.contains_key(name)
    }

    /// The preprocessing flow for a topic, or nothing when the topic is not
    /// registered.
    ///
    /// A topic's name is the whole key. This deployment serves one language,
    /// so there is no second dimension to look anything up by.
    pub fn get_preprocessor(&self, name: &str) -> Option<&Flow> {
        self.pipelines.get(name).map(|pair| &pair.pre)
    }

    /// The postprocessing flow for a topic. The counterpart of
    /// [`Registry::get_preprocessor`].
    pub fn get_postprocessor(&self, name: &str) -> Option<&Flow> {
        self.pipelines.get(name).map(|pair| &pair.post)
    }

    /// One topic registered with flows of the caller's choosing, for tests
    /// that need a pipeline pair without the shipped one's models.
    #[cfg(test)]
    pub(crate) fn of_flows(topic: &str, pre: Flow, post: Flow) -> Self {
        Registry {
            pipelines: BTreeMap::from([(topic.to_string(), Pipelines { pre, post })]),
            topics: vec![Topic {
                name: topic.to_string(),
                label: topic.to_string(),
                enabled: true,
                texts: Vec::new(),
            }],
        }
    }

    /// A registry offering nothing, for tests that need a lookup to miss.
    #[cfg(test)]
    pub(crate) fn empty() -> Self {
        Registry {
            pipelines: BTreeMap::new(),
            topics: Vec::new(),
        }
    }
}

#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
