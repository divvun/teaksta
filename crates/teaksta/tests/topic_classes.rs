//! One topic-class scheme: every shipped topic's enhancer marks its hits with
//! `teaksta-` followed by the name the activity registry serves that topic
//! under.
//!
//! The client is handed those names by `/api/activities` and derives the class
//! it looks for from the name it was given, so a topic whose enhancer marks
//! its hits under any other class is inert in every exercise: nothing is
//! coloured, every pick is judged wrong, and no question is asked. The scheme
//! is therefore a contract between the registry and the enhancers, and this is
//! where the two are read against each other.
//!
//! The walk is over the deployment's own activity tree and loads no models: an
//! `activity.xml` carries exactly the tag parameter of its own enhancer, so the
//! one delegate that builds from a topic's post configuration is that topic's.

use std::path::{Path, PathBuf};

use teaksta::pipeline::flow::{Parameters, stage_named};
use teaksta::server::activities::Activities;
use teaksta::types::PIPELINE_LANGUAGE;

/// Every delegate key a shipped post-processing descriptor names for a topic,
/// paired with the parameter its activity configures it through. The generic
/// `TokenEnhancer` that runs alongside them stands for no topic.
const TOPIC_DELEGATES: &[(&str, &str)] = &[
    ("vislcg3AdverbialEnhancer", "AdvTags"),
    ("vislcg3ConNegEnhancer", "connegTags"),
    ("vislcg3ConjunctionEnhancer", "conjunctionTags"),
    ("vislcg3InfiniteVerbEnhancer", "infiniteverbTags"),
    ("vislcg3NounEnhancer", "NTags"),
    ("vislcg3NounPlEnhancer", "NPlTags"),
    ("vislcg3NounSgEnhancer", "NSgTags"),
    ("vislcg3ObjectEnhancer", "ObjTags"),
    ("vislcg3SubjectEnhancer", "SubjTags"),
    ("vislcg3VerbConjugationEnhancer", "finverbTags"),
];

/// The deployment's activity tree, which is what a running server scans.
fn shipped_activities() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sme/src/main/webapp/activities")
}

/// The descriptor tree the activity expressions resolve against.
fn shipped_descriptors() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sme/desc")
}

/// The names the registry serves, in the order it serves them.
fn registered_topics() -> Vec<String> {
    Activities::new(&shipped_activities(), &shipped_descriptors())
        .expect("the shipped activity tree is a registry")
        .iterator()
        .cloned()
        .collect()
}

/// The class the topic behind this post configuration marks its hits with.
fn span_class_of(post: &Parameters) -> Option<&'static str> {
    let mut found: Option<(&str, &'static str)> = None;

    for (delegate, _) in TOPIC_DELEGATES {
        let Ok(stage) = stage_named(delegate, post) else {
            continue;
        };
        let class = stage
            .topic_span_class()
            .expect("a topic delegate names its topic");
        if let Some((earlier, _)) = found {
            panic!("{earlier} and {delegate} both build from one activity's configuration");
        }
        found = Some((delegate, class));
    }

    found.map(|(_, class)| class)
}

#[test]
fn every_topic_marks_hits_with_its_registry_name() {
    let mut activities = Activities::new(&shipped_activities(), &shipped_descriptors())
        .expect("the shipped activity tree is a registry");
    let names = registered_topics();
    assert_eq!(names.len(), TOPIC_DELEGATES.len(), "{names:?}");

    for name in names {
        let config = activities
            .get_activity(&name)
            .expect("a name the registry just handed out");
        let post = config.get_server_post_config_as_prop(PIPELINE_LANGUAGE);

        let class = span_class_of(&post)
            .unwrap_or_else(|| panic!("{name} configures no topic enhancer of its own"));

        assert_eq!(
            class,
            format!("teaksta-{name}"),
            "the {name} topic marks its hits {class}, which the client — deriving \
             the class from the registry name it was served — never looks for"
        );
    }
}

#[test]
fn no_enhancer_marks_a_class_no_topic_names() {
    let named: Vec<String> = registered_topics()
        .iter()
        .map(|name| format!("teaksta-{name}"))
        .collect();

    for (delegate, parameter) in TOPIC_DELEGATES {
        let parameters = Parameters::from([(parameter.to_string(), "A,B".to_string())]);
        let class = stage_named(delegate, &parameters)
            .expect("a topic delegate builds from its own parameter")
            .topic_span_class()
            .expect("a topic delegate names its topic");

        assert!(
            named.contains(&class.to_string()),
            "{delegate} marks its hits {class}, which no shipped topic is named by"
        );
    }
}
