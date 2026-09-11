//! One topic-class scheme: every topic's enhancer marks its hits with
//! `teaksta-` followed by the name the registry serves that topic under.
//!
//! The client is handed those names by `/api/activities` and derives the class
//! it looks for from the name it was given, so a topic whose enhancer marks
//! its hits under any other class is inert in every exercise: nothing is
//! coloured, every pick is judged wrong, and no question is asked. The scheme
//! is therefore a contract between the registry and the enhancers, and this is
//! where the two are read against each other.
//!
//! The walk is over the registry compiled into the binary, and loads no
//! models: a topic declares exactly one enhancer, so the enhancer that stands
//! for a topic is the one that topic named.

use teaksta::server::registry::{Enhancer, Registry};

/// The registry a deployment naming no topics file is served.
fn shipped() -> Registry {
    Registry::from_config(None).expect("the compiled-in topics are a registry")
}

#[test]
fn every_topic_marks_hits_with_its_registry_name() {
    let registry = shipped();
    let names: Vec<&str> = registry
        .topics()
        .iter()
        .map(|topic| topic.name.as_str())
        .collect();
    // One topic per enhancer this build carries: an enhancer no topic names
    // is dead code, and a topic is the only way one is reached.
    assert_eq!(names.len(), Enhancer::ALL.len(), "{names:?}");

    for name in names {
        let post = registry
            .get_postprocessor(name)
            .expect("a name the registry just handed out");

        let classes = post.topic_span_classes();
        assert_eq!(
            classes.len(),
            1,
            "the {name} topic runs {} enhancers that stand for a topic",
            classes.len()
        );
        assert_eq!(
            classes[0],
            format!("teaksta-{name}"),
            "the {name} topic marks its hits {}, which the client — deriving \
             the class from the registry name it was served — never looks for",
            classes[0]
        );
    }
}

#[test]
fn no_enhancer_marks_a_class_no_topic_names() {
    let registry = shipped();
    let named: Vec<String> = registry
        .topics()
        .iter()
        .map(|topic| format!("teaksta-{}", topic.name))
        .collect();

    for enhancer in Enhancer::ALL {
        let class = enhancer.span_class();

        assert!(
            named.contains(&class.to_string()),
            "{enhancer:?} marks its hits {class}, which no topic is named by"
        );
    }
}

/// The preprocessing flow stands for no topic, so one cached analysis of a
/// page answers every topic's request for it.
#[test]
fn the_preprocessor_stands_for_no_topic() {
    let registry = shipped();

    for topic in registry.topics() {
        let pre = registry
            .get_preprocessor(&topic.name)
            .expect("a name the registry just handed out");

        assert!(pre.topic_span_classes().is_empty(), "{}", topic.name);
    }
}
