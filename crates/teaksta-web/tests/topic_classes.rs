//! A topic whose class is not its own name spelled twice.
//!
//! The client decides what is a hit by building the class from the topic name
//! the registry served it, so a fixture for `Substantive` — where the topic's
//! class happens to read like its name however the scheme is spelled — cannot
//! tell a working derivation from a broken one. `Conjunctions` can: its
//! enhancer once marked hits `teaksta-conjunction`, and under that class the
//! whole topic was inert. The fixtures here are `POST /api/enhance/blocks`
//! and `GET /api/activities` replies saved as they arrived.

use std::rc::Rc;

use dioxus::prelude::*;

use teaksta_web::api::{parse_blocks, parse_registry};
use teaksta_web::ui::exercise::colorize::{ColorizeMode, ColorizeModeProps, HIT_CLASS};
use teaksta_web::ui::exercise::markup::{Markup, parse};

const ACTIVITIES: &str = include_str!("fixtures/activities.json");
const COLORIZE: &str = include_str!("fixtures/conjunctions-colorize.json");

/// One saved reply, read as the exercises read it.
fn read(reply: &str) -> Markup {
    let blocks: Vec<String> = parse_blocks(reply)
        .expect("the backend's reply parses")
        .into_iter()
        .map(|block| block.html)
        .collect();

    parse(&blocks)
}

/// The name the backend serves this topic under, taken from the registry
/// rather than written out here: it is the name the client builds the class
/// from, so reading it from anywhere else would test a different thing.
fn topic() -> String {
    parse_registry(ACTIVITIES)
        .expect("the backend's registry parses")
        .activities
        .into_iter()
        .find(|topic| topic.label.as_deref() == Some("Konjunkšuvnnat"))
        .expect("the registry offers the conjunctions topic")
        .name
}

fn colorize_page() -> String {
    let mut dom = VirtualDom::new_with_props(
        ColorizeMode,
        ColorizeModeProps {
            markup: Rc::new(read(COLORIZE)),
            topic: topic(),
            prompt: "Geahča ivdnejuvvon sániid.".to_string(),
        },
    );
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn the_topic_is_named_by_the_registry() {
    assert_eq!(topic(), "Conjunctions");
}

#[test]
fn the_enhancer_marks_the_conjunctions_it_found() {
    let markup = read(COLORIZE);
    let topic = topic();

    assert_eq!(markup.hits(&topic), 3);
    assert!(markup.tokens().iter().all(|token| token.is_hit(&topic)));
    for form in ["ja", "muhto", "go"] {
        assert!(
            markup.tokens().iter().any(|token| token.text == form),
            "missing {form}"
        );
    }
}

#[test]
fn per_tag_classes_ride_alongside_the_topic_class() {
    let markup = read(COLORIZE);
    let carried = |class: &str| {
        markup
            .tokens()
            .iter()
            .filter(|token| token.classes.iter().any(|it| it == class))
            .count()
    };

    // The coordinators and the subordinator are told apart by a class of their
    // own, which the topic class is additional to rather than replaced by.
    assert_eq!(carried("teaksta-CC"), 2);
    assert_eq!(carried("teaksta-CS"), 1);
    assert_eq!(carried("teaksta-Conjunctions"), 3);
}

/// The highlight is written on the state class, not on one topic's own, so
/// this topic wears the same band the noun topics wear — which is the whole
/// point of a class the enhancer never styles by itself.
#[test]
fn colorize_styles_every_conjunction() {
    let markup = read(COLORIZE);
    let html = colorize_page();

    assert_eq!(html.matches(HIT_CLASS).count(), markup.hits(&topic()));
    // The per-tag class the enhancer wrote alongside the topic's own rides
    // along untouched, with the highlight added after both.
    assert!(html.contains(&format!(
        "teaksta-token teaksta-Conjunctions teaksta-CC {HIT_CLASS}"
    )));
    for form in ["ja", "muhto", "go"] {
        assert!(html.contains(&format!(">{form}</span>")), "missing {form}");
    }
    assert!(html.contains("Ivdnejuvvon sánit"));
    assert!(html.contains("class=\"tk-score-count\">3<"));
    assert!(html.contains("beatnaga ikte."));
}

#[test]
fn another_topic_finds_nothing_on_the_page() {
    let markup = read(COLORIZE);

    assert_eq!(markup.hits("Substantive"), 0);
    assert_eq!(markup.hits("NegVerbs"), 0);
    // The class is the name, not a prefix of it: the old scheme's spelling is
    // no longer a hit, and neither is a name the class merely starts with.
    assert_eq!(markup.hits("conjunction"), 0);
    assert_eq!(markup.hits("Conjunction"), 0);
}
