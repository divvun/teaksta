use super::*;

use crate::types::PIPELINE_LANGUAGE;

/// The ten topics the compiled-in registry declares, in the order it serves
/// them, each with the North Sámi label the learner is shown.
const SHIPPED: &[(&str, &str)] = &[
    ("Adverbial", "Adverbiála"),
    ("Conjunctions", "Konjunkšuvnnat"),
    ("InfiniteVerbs", "Infinihtta vearbbat"),
    ("NegVerbs", "Biehttalanvearbbat"),
    ("Object", "Objeakta"),
    ("Subject", "Subjeakta"),
    ("Substantive", "Substantiivvat"),
    ("SubstantivePlural", "Substantiivvat máŋggaidlogus"),
    ("SubstantiveSingular", "Substantiivvat ovttaidlogus"),
    ("VerbConjugation", "Finihtta vearbbat"),
];

/// One well-formed topic, for the tests that vary one thing about a file.
fn one_topic(name: &str) -> String {
    format!(
        "[[topic]]\n\
         name = \"{name}\"\n\
         label = \"Fáddá\"\n\
         enabled = true\n\
         enhancer = \"noun\"\n\
         tags = \"Sg Nom\"\n\
         token_tags = \"N\"\n"
    )
}

#[test]
fn the_compiled_in_registry_offers_every_shipped_topic() {
    let registry = Registry::from_config(None).expect("the compiled-in topics parse");

    let served: Vec<(&str, &str)> = registry
        .topics()
        .iter()
        .map(|topic| (topic.name.as_str(), topic.label.as_str()))
        .collect();
    assert_eq!(served, SHIPPED);
    assert!(registry.topics().iter().all(|topic| topic.enabled));
}

/// The scheme the client depends on: it derives the class it looks for from
/// the name the registry served it, so a topic whose enhancer marks anything
/// else is inert in every exercise.
#[test]
fn every_topic_marks_its_registry_name() {
    let registry = Registry::from_config(None).expect("the compiled-in topics parse");

    for topic in registry.topics() {
        let post = registry
            .get_postprocessor(PIPELINE_LANGUAGE, &topic.name)
            .expect("a name the registry just handed out");

        assert_eq!(
            post.topic_span_classes(),
            vec![format!("teaksta-{}", topic.name).as_str()],
            "the {} topic does not mark its hits under its own name",
            topic.name
        );
    }
}

#[test]
fn every_topic_carries_a_pipeline_pair() {
    let registry = Registry::from_config(None).expect("the compiled-in topics parse");

    for (name, _) in SHIPPED {
        assert!(registry.knows(name));
        assert_eq!(
            registry
                .get_preprocessor(PIPELINE_LANGUAGE, name)
                .expect("a preprocessor")
                .len(),
            5
        );
        assert_eq!(
            registry
                .get_postprocessor(PIPELINE_LANGUAGE, name)
                .expect("a postprocessor")
                .len(),
            2
        );
    }
}

#[test]
fn a_lookup_misses_unknown_topic_or_language() {
    let registry = Registry::from_config(None).expect("the compiled-in topics parse");

    assert!(
        registry
            .get_preprocessor(PIPELINE_LANGUAGE, "Kitchens")
            .is_none()
    );
    assert!(
        registry
            .get_postprocessor(PIPELINE_LANGUAGE, "Kitchens")
            .is_none()
    );
    assert!(!registry.knows("Kitchens"));
    // The name is matched as it is served, not case-folded.
    assert!(
        registry
            .get_preprocessor(PIPELINE_LANGUAGE, "substantive")
            .is_none()
    );
    // Every topic is registered under the one language this deployment has a
    // pipeline for, so any other misses.
    assert!(
        registry
            .get_preprocessor("klingon", "Substantive")
            .is_none()
    );
    assert!(
        registry
            .get_postprocessor("klingon", "Substantive")
            .is_none()
    );
}

#[test]
fn a_named_file_replaces_the_registry_whole() {
    let dir = tempfile::tempdir().expect("temp dir");
    let file = dir.path().join("topics.toml");
    std::fs::write(&file, one_topic("Fáddá")).expect("the override is written");

    let registry = Registry::from_config(Some(&file)).expect("the override parses");

    // Replaced, not merged: what the file says is the whole registry.
    assert_eq!(registry.topics().len(), 1);
    assert_eq!(registry.topics()[0].name, "Fáddá");
    assert!(!registry.knows("Substantive"));
}

#[test]
fn the_registry_is_ordered_by_name() {
    let mut written = one_topic("Zebra");
    written.push_str(&one_topic("Apple"));

    let registry = Registry::parse(&written).expect("both topics parse");

    let names: Vec<&str> = registry
        .topics()
        .iter()
        .map(|topic| topic.name.as_str())
        .collect();
    assert_eq!(names, vec!["Apple", "Zebra"]);
}

#[test]
fn a_file_that_will_not_parse_names_itself() {
    let dir = tempfile::tempdir().expect("temp dir");
    let file = dir.path().join("broken.toml");
    std::fs::write(&file, "[[topic]]\nname = \"Half\"\n").expect("the override is written");

    let Err(error) = Registry::from_config(Some(&file)) else {
        panic!("a topics file missing half its fields must fail the boot");
    };

    let reported = format!("{error:#}");
    assert!(reported.contains(&file.display().to_string()), "{reported}");
    assert!(reported.contains("label"), "{reported}");
}

#[test]
fn a_file_that_is_not_there_names_itself() {
    let dir = tempfile::tempdir().expect("temp dir");
    let absent = dir.path().join("absent.toml");

    let Err(error) = Registry::from_config(Some(&absent)) else {
        panic!("a topics file that is not there must fail the boot");
    };

    assert!(
        format!("{error:#}").contains(&absent.display().to_string()),
        "{error:#}"
    );
}

#[test]
fn an_unread_key_is_refused() {
    let written = format!("{}colour = \"blue\"\n", one_topic("Fáddá"));

    let Err(error) = Registry::parse(&written) else {
        panic!("a key the server ignores must fail the boot rather than be ignored");
    };

    assert!(format!("{error:#}").contains("colour"), "{error:#}");
}

#[test]
fn an_enhancer_no_build_carries_is_refused() {
    let written = one_topic("Fáddá").replace("\"noun\"", "\"maltparser\"");

    let Err(error) = Registry::parse(&written) else {
        panic!("an enhancer with no counterpart must fail the boot");
    };

    assert!(format!("{error:#}").contains("maltparser"), "{error:#}");
}

#[test]
fn a_topic_declared_twice_is_refused() {
    let written = format!("{}{}", one_topic("Fáddá"), one_topic("Fáddá"));

    let Err(error) = Registry::parse(&written) else {
        panic!("two topics under one name must fail the boot");
    };

    assert!(format!("{error:#}").contains("declared twice"), "{error:#}");
}

#[test]
fn a_registry_offering_nothing_is_refused() {
    let Err(error) = Registry::parse("") else {
        panic!("a registry with no topics must fail the boot");
    };

    assert!(format!("{error:#}").contains("no topics"), "{error:#}");
}

/// An empty tag list splits to one empty tag, and every reading contains the
/// empty string — so such a topic would mark the whole page as a hit rather
/// than nothing. No enhancer refuses it on its own, so the registry does,
/// naming the topic.
#[test]
fn a_topic_declaring_no_tags_is_refused() {
    for empty in ["\"\"", "\"   \""] {
        let written = one_topic("Fáddá").replace("\"Sg Nom\"", empty);

        let Err(error) = Registry::parse(&written) else {
            panic!("a topic that would mark every word must fail the boot");
        };

        let reported = format!("{error:#}");
        assert!(reported.contains("Fáddá"), "{reported}");
        assert!(reported.contains("no tags"), "{reported}");
    }
}

/// The lemma filter is the one field a topic may leave out. What the two
/// values do to the flow is [`crate::pipeline::flow`]'s to prove; that a file
/// may omit it, and that declaring it is not an unknown key, is this one's.
#[test]
fn the_lemma_filter_is_the_only_optional_field() {
    assert!(!one_topic("Fáddá").contains("use_lemma_filter"));
    Registry::parse(&one_topic("Fáddá")).expect("a topic may leave the lemma filter out");

    let written = format!("{}use_lemma_filter = true\n", one_topic("Fáddá"));
    Registry::parse(&written).expect("and may declare it");

    // Every other field is required.
    for field in ["name", "label", "enabled", "enhancer", "tags", "token_tags"] {
        let without: String = one_topic("Fáddá")
            .lines()
            .filter(|line| !line.starts_with(&format!("{field} =")))
            .map(|line| format!("{line}\n"))
            .collect();
        assert!(
            Registry::parse(&without).is_err(),
            "a topic without {field} was accepted"
        );
    }
}
