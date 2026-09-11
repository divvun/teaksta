//! The two lists an entry form is built from: the grammar topics on offer, and
//! the exercises any of them can be read with.
//!
//! Naming a page and offering a text of your own are two ways to the same
//! exercise, so both forms ask for the same two things and differ only in
//! where the page comes from. The lists stand apart from either.
//!
//! What a topic or a mode is *called* always comes from the registry, so a
//! deployment names its own topics. What one *is* cannot: the backend serves
//! no description, and ten North Sámi names over ten directory names tell a
//! learner who has met neither vocabulary nothing. So the short English line
//! under each name is this app's own, keyed by the backend's name and falling
//! back to that name for anything a deployment adds.

use std::rc::Rc;

use dioxus::prelude::*;

use crate::api::Registry;

/// One registry entry as a form shows it: what the backend calls it, what the
/// learner reads, and whether it can be chosen at all.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Named {
    pub name: String,
    pub label: String,
    pub enabled: bool,
}

/// The topic a form starts on: the first the deployment has switched on, since
/// a topic switched off cannot be chosen at all.
pub fn first_topic(registry: &Registry) -> String {
    registry
        .activities
        .iter()
        .find(|activity| activity.enabled)
        .map(|activity| activity.name.clone())
        .unwrap_or_default()
}

/// The names the two lists answer under. Each control carries the backend's
/// own name for what it stands for as its value, so the registry's vocabulary
/// is in the markup as well as on the page — the English line beside a topic
/// is for reading, not for addressing it.
const TOPIC_GROUP: &str = "topic";
const MODE_GROUP: &str = "mode";

/// What each shipped topic marks, in English, under the North Sámi name.
const TOPIC_SUBTITLES: &[(&str, &str)] = &[
    ("Substantive", "Nouns in any form"),
    ("SubstantiveSingular", "Singular nouns only"),
    ("SubstantivePlural", "Plural nouns only"),
    ("Subject", "The subject of each clause"),
    ("Object", "The object of each clause"),
    ("Adverbial", "Adverbial phrases"),
    ("Conjunctions", "Coordinating conjunctions"),
    ("NegVerbs", "Negation verbs"),
    ("InfiniteVerbs", "Non-finite verb forms"),
    ("VerbConjugation", "Finite verbs and their conjugation"),
];

/// What each mode asks for, in three or four words, so a segment says more
/// than the backend's parameter value does on its own.
const MODE_HINTS: &[(&str, &str)] = &[
    ("colorize", "Read & notice"),
    ("click", "Find them"),
    ("mc", "Pick a form"),
    ("cloze", "Type it"),
];

/// The English reading of each mode's North Sámi instruction.
const MODE_GLOSSES: &[(&str, &str)] = &[
    ("colorize", "Look at the coloured words."),
    ("click", "Click the right words!"),
    ("mc", "Choose the right words!"),
    ("cloze", "Write the right words!"),
];

fn looked_up<'a>(table: &'a [(&'a str, &'a str)], name: &'a str) -> &'a str {
    table
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, text)| *text)
        .unwrap_or(name)
}

/// What a topic marks, in English. A topic this app has never met reads as the
/// name the backend serves it under, which is all anyone knows about it.
pub fn topic_subtitle(name: &str) -> &str {
    looked_up(TOPIC_SUBTITLES, name)
}

/// What a mode asks for, in a few English words.
pub fn mode_hint(name: &str) -> &str {
    looked_up(MODE_HINTS, name)
}

/// The English reading of a mode's North Sámi instruction.
pub fn mode_gloss(name: &str) -> &str {
    looked_up(MODE_GLOSSES, name)
}

/// The topics on offer, over the one the form is set to.
#[component]
pub fn Topics(registry: Rc<Registry>, chosen: String, onpick: EventHandler<String>) -> Element {
    let offered = named_topics(&registry);
    let picked = usize::from(offered.iter().any(|topic| topic.name == chosen));
    let total = offered.len();

    rsx! {
        section { class: "picker",
            div { class: "tk-fieldhead",
                h2 { lang: "se",
                    "Fáddá "
                    span { class: "tk-gloss", "Topic" }
                }
                span { class: "tk-gloss", "{picked} of {total}" }
            }
            ul { class: "topics",
                for topic in offered {
                    li { key: "{topic.name}",
                        TopicButton {
                            chosen: topic.name == chosen,
                            topic,
                            onpick: move |name| onpick.call(name),
                        }
                    }
                }
            }
        }
    }
}

/// The exercises on offer, over the one the form is set to, and the sentence
/// the chosen one will put in front of the learner. Modes are declared once
/// for the whole registry, so every topic is offered every one of them.
#[component]
pub fn Modes(registry: Rc<Registry>, chosen: String, onpick: EventHandler<String>) -> Element {
    let offered = named_modes(&registry);
    let ways = offered.len();
    let saying = registry.mode_label(&chosen).to_string();

    rsx! {
        section { class: "exercises",
            div { class: "tk-fieldhead",
                h2 { lang: "se",
                    "Hárjehus "
                    span { class: "tk-gloss", "Exercise" }
                }
                span { class: "tk-gloss", "Same text, {ways} ways in" }
            }
            div { class: "seg", role: "tablist",
                for mode in offered {
                    ModeTab {
                        key: "{mode.name}",
                        chosen: mode.name == chosen,
                        mode,
                        onpick: move |name| onpick.call(name),
                    }
                }
            }
            div { class: "seg-say",
                span { class: "seg-say-sme", lang: "se", "{saying}" }
                span { class: "tk-gloss", "{mode_gloss(&chosen)}" }
            }
        }
    }
}

fn named_topics(registry: &Registry) -> Vec<Named> {
    registry
        .activities
        .iter()
        .map(|activity| Named {
            name: activity.name.clone(),
            label: registry.activity_label(&activity.name).to_string(),
            enabled: activity.enabled,
        })
        .collect()
}

fn named_modes(registry: &Registry) -> Vec<Named> {
    registry
        .modes
        .iter()
        .map(|mode| Named {
            name: mode.name.clone(),
            label: registry.mode_label(&mode.name).to_string(),
            enabled: true,
        })
        .collect()
}

/// A topic is picked by pressing it, so each is a button of its own. The tick
/// stands in every row and only fills in on the chosen one, so the list never
/// changes shape as a learner moves down it.
#[component]
fn TopicButton(topic: Named, chosen: bool, onpick: EventHandler<String>) -> Element {
    let class = if chosen {
        "topic topic-chosen"
    } else {
        "topic"
    };
    let pressed = if chosen { "true" } else { "false" };
    let picked = topic.name.clone();

    rsx! {
        button {
            r#type: "button",
            class: "{class}",
            name: TOPIC_GROUP,
            value: "{topic.name}",
            "aria-pressed": "{pressed}",
            disabled: !topic.enabled,
            onclick: move |_| onpick.call(picked.clone()),
            span { class: "topic-body",
                span { class: "topic-sme", lang: "se", "{topic.label}" }
                span { class: "topic-en", "{topic_subtitle(&topic.name)}" }
            }
            span { class: "topic-tick", {tick()} }
        }
    }
}

/// One segment of the mode switch. It carries the backend's own mode name, so
/// that the app and the deployment keep one vocabulary between them, with the
/// English hint underneath saying what that name asks for.
#[component]
fn ModeTab(mode: Named, chosen: bool, onpick: EventHandler<String>) -> Element {
    let selected = if chosen { "true" } else { "false" };
    let picked = mode.name.clone();

    rsx! {
        button {
            r#type: "button",
            role: "tab",
            class: "seg-item",
            name: MODE_GROUP,
            value: "{mode.name}",
            "aria-selected": "{selected}",
            onclick: move |_| onpick.call(picked.clone()),
            span { class: "seg-name", "{mode.name}" }
            span { class: "seg-hint", "{mode_hint(&mode.name)}" }
        }
    }
}

fn tick() -> Element {
    rsx! {
        svg {
            class: "tk-tick",
            width: "14",
            height: "11",
            view_box: "0 0 14 11",
            "aria-hidden": "true",
            path {
                d: "M1 5.6l4 4L13 1.4",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2.1",
                stroke_linecap: "round",
                stroke_linejoin: "round",
            }
        }
    }
}

/// One radio in a group: what it stands for, what the learner reads, and the
/// English gloss beneath it. The one either-or a form asks that the backend
/// does not answer for itself is put this way, so the answer reads as a choice
/// between two futures for the file rather than as a switch to overlook.
#[component]
pub fn Choice(
    group: String,
    value: String,
    label: String,
    gloss: String,
    chosen: bool,
    onpick: EventHandler<String>,
) -> Element {
    let picked = value.clone();

    rsx! {
        label { class: "opt",
            input {
                r#type: "radio",
                name: "{group}",
                value: "{value}",
                checked: chosen,
                onchange: move |_| onpick.call(picked.clone()),
            }
            span { class: "opt-text",
                span { lang: "se", "{label}" }
                br {}
                span { class: "tk-gloss", "{gloss}" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_shipped_topic_reads_in_english() {
        assert_eq!(topic_subtitle("Substantive"), "Nouns in any form");
        assert_eq!(topic_subtitle("NegVerbs"), "Negation verbs");
        assert_eq!(TOPIC_SUBTITLES.len(), 10);
    }

    /// A deployment may offer a topic this app has never met, and the list has
    /// to read the same when it does.
    #[test]
    fn an_unknown_topic_reads_as_its_name() {
        assert_eq!(topic_subtitle("Preps"), "Preps");
        assert_eq!(mode_hint("nonesuch"), "nonesuch");
        assert_eq!(mode_gloss("nonesuch"), "nonesuch");
    }

    #[test]
    fn every_mode_says_what_it_asks_for() {
        assert_eq!(mode_hint("cloze"), "Type it");
        assert_eq!(mode_gloss("cloze"), "Write the right words!");
        assert_eq!(MODE_HINTS.len(), MODE_GLOSSES.len());
    }
}
