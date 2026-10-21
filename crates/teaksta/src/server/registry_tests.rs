use super::*;

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
            .get_postprocessor(&topic.name)
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
                .get_preprocessor(name)
                .expect("a preprocessor")
                .len(),
            5
        );
        assert_eq!(
            registry
                .get_postprocessor(name)
                .expect("a postprocessor")
                .len(),
            2
        );
    }
}

#[test]
fn a_lookup_misses_an_unknown_topic() {
    let registry = Registry::from_config(None).expect("the compiled-in topics parse");

    assert!(registry.get_preprocessor("Kitchens").is_none());
    assert!(registry.get_postprocessor("Kitchens").is_none());
    assert!(!registry.knows("Kitchens"));
    // The name is matched as it is served, not case-folded.
    assert!(registry.get_preprocessor("substantive").is_none());
    assert!(registry.get_postprocessor("substantive").is_none());
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

/// The lemma filter and the texts are the fields a topic may leave out. What
/// the filter's two values do to the flow is [`crate::pipeline::flow`]'s to
/// prove; that a file may omit either, and that declaring them is not an
/// unknown key, is this one's.
#[test]
fn only_the_lemma_filter_and_texts_are_optional() {
    assert!(!one_topic("Fáddá").contains("use_lemma_filter"));
    let registry =
        Registry::parse(&one_topic("Fáddá")).expect("a topic may leave the lemma filter out");
    assert!(registry.topics()[0].texts.is_empty());

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

/// One text, as a topic declares it under its own keys.
fn with_text(text: &str) -> String {
    format!("{}\n[[topic.text]]\n{text}", one_topic("Fáddá"))
}

/// The six topics the webapp recommended pages for offer them again, in its
/// order, and the four it recommended none for offer none.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.activities-fn+3/test]
#[test]
fn six_topics_offer_their_old_texts() {
    let registry = Registry::from_config(None).expect("the compiled-in topics parse");
    let offered: Vec<(&str, usize)> = registry
        .topics()
        .iter()
        .map(|topic| (topic.name.as_str(), topic.texts.len()))
        .collect();

    assert_eq!(
        offered,
        [
            ("Adverbial", 0),
            ("Conjunctions", 0),
            ("InfiniteVerbs", 4),
            ("NegVerbs", 3),
            ("Object", 0),
            ("Subject", 0),
            ("Substantive", 8),
            ("SubstantivePlural", 4),
            ("SubstantiveSingular", 5),
            ("VerbConjugation", 5),
        ]
    );

    let texts = registry.topics().iter().flat_map(|topic| &topic.texts);
    for text in texts {
        assert!(text.url.starts_with("https://"), "{}", text.url);
        assert!(text.words > 0, "{}", text.title);
    }

    let conjugation = &registry.topics()[9];
    assert_eq!(conjugation.texts[0].title, "Mearragáttis");
    assert_eq!(conjugation.texts[0].tense, Some(Tense::Present));
    assert_eq!(conjugation.texts[1].tense, None);
    assert_eq!(conjugation.texts[2].tense, Some(Tense::Past));
}

/// A text is served as written, and one that names no web address, carries
/// no title, or names a kind of text this build does not know is refused when
/// the file is read rather than offered to a learner.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.activities-fn+3/test]
#[test]
fn a_text_is_read_as_written_or_refused() {
    let registry = Registry::parse(&with_text(
        "title = \"Mearragáttis\"\nurl = \"https://example.org/a\"\ngenre = \"story\"\nwords = 190\n",
    ))
    .expect("a well-formed text");
    assert_eq!(
        registry.topics()[0].texts,
        [SampleText {
            title: "Mearragáttis".to_string(),
            url: "https://example.org/a".to_string(),
            genre: Genre::Story,
            words: 190,
            tense: None,
        }]
    );

    for refused in [
        "title = \"A\"\nurl = \"ftp://example.org/a\"\ngenre = \"story\"\nwords = 1\n",
        "title = \"A\"\nurl = \"/api/texts/a\"\ngenre = \"story\"\nwords = 1\n",
        "title = \" \"\nurl = \"https://example.org/a\"\ngenre = \"story\"\nwords = 1\n",
        "title = \"A\"\nurl = \"https://example.org/a\"\ngenre = \"poem\"\nwords = 1\n",
        "title = \"A\"\nurl = \"https://example.org/a\"\ngenre = \"story\"\nwords = 1\nlevel = 2\n",
    ] {
        assert!(Registry::parse(&with_text(refused)).is_err(), "{refused}");
    }
}
