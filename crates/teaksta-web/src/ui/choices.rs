//! The two lists an entry form is built from: the grammar topics on offer, and
//! the exercises any of them can be read with.
//!
//! Naming a page and offering a text of your own are two ways to the same
//! exercise, so both forms ask for the same two things and differ only in
//! where the page comes from. The lists stand apart from either.

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

/// The radio group the exercise modes are asked as.
const MODE_GROUP: &str = "mode";

/// The topics on offer, over the one the form is set to.
#[component]
pub fn Topics(registry: Rc<Registry>, chosen: String, onpick: EventHandler<String>) -> Element {
    rsx! {
        section { class: "picker",
            h2 {
                "Fáddá"
                span { class: "gloss", "Topic" }
            }
            ul { class: "topics", {topic_items(&registry, &chosen, onpick)} }
        }
    }
}

/// The exercises on offer, over the one the form is set to. Modes are declared
/// once for the whole registry, so every topic is offered every one of them.
#[component]
pub fn Modes(registry: Rc<Registry>, chosen: String, onpick: EventHandler<String>) -> Element {
    rsx! {
        fieldset { class: "exercises",
            legend {
                "Hárjehus"
                span { class: "gloss", "Exercise type" }
            }
            {mode_items(&registry, &chosen, onpick)}
        }
    }
}

/// A topic is picked by pressing it, so each is a button of its own.
fn topic_items(registry: &Registry, chosen: &str, onpick: EventHandler<String>) -> Element {
    rsx! {
        for topic in named_topics(registry) {
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

/// An exercise is one answer to a single question, so the modes are radios.
fn mode_items(registry: &Registry, chosen: &str, onpick: EventHandler<String>) -> Element {
    rsx! {
        for mode in named_modes(registry) {
            Choice {
                key: "{mode.name}",
                group: MODE_GROUP.to_string(),
                chosen: mode.name == chosen,
                value: mode.name.clone(),
                label: mode.label.clone(),
                gloss: mode.name.clone(),
                onpick: move |value| onpick.call(value),
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
            "aria-pressed": "{pressed}",
            disabled: !topic.enabled,
            onclick: move |_| onpick.call(picked.clone()),
            span { class: "topic-sme", "{topic.label}" }
            span { class: "gloss", "{topic.name}" }
        }
    }
}

/// One radio in a group: what it stands for, what the learner reads, and the
/// English gloss beside it. Every either-or on a form is asked this way, so
/// the exercise modes and what becomes of an uploaded file are one control
/// answered twice.
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
        label { class: "exercise",
            input {
                r#type: "radio",
                name: "{group}",
                value: "{value}",
                checked: chosen,
                onchange: move |_| onpick.call(picked.clone()),
            }
            span { class: "exercise-sme", "{label}" }
            span { class: "gloss", "{gloss}" }
        }
    }
}
