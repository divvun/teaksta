//! The entry form: pick a grammar topic, name a page, pick an exercise type.

use dioxus::prelude::*;

use crate::activities::{DEFAULT_EXERCISE, TOPICS, exercises_for, offers, topic};
use crate::route::{ExerciseQuery, Route};

#[component]
pub fn Home() -> Element {
    let navigator = use_navigator();
    let mut chosen_topic = use_signal(|| TOPICS[0].id.to_string());
    let mut chosen_exercise = use_signal(|| DEFAULT_EXERCISE.to_string());
    let mut page_url = use_signal(String::new);

    let current_topic = chosen_topic();
    let available = exercises_for(&current_topic);
    let ready = !page_url().trim().is_empty();

    rsx! {
        section { class: "picker",
            h2 {
                "Fáddá"
                span { class: "gloss", "Topic" }
            }
            ul { class: "topics",
                for item in TOPICS {
                    li { key: "{item.id}",
                        button {
                            r#type: "button",
                            class: if item.id == current_topic { "topic topic-chosen" } else { "topic" },
                            "aria-pressed": if item.id == current_topic { "true" } else { "false" },
                            onclick: move |_| {
                                chosen_topic.set(item.id.to_string());
                                if !offers(item.id, &chosen_exercise()) {
                                    chosen_exercise.set(DEFAULT_EXERCISE.to_string());
                                }
                            },
                            span { class: "topic-sme", "{item.sme}" }
                            span { class: "gloss", "{item.id}" }
                        }
                    }
                }
            }
        }

        form {
            class: "entry",
            onsubmit: move |event| {
                event.prevent_default();
                let target = page_url();
                let params = ExerciseQuery::new(chosen_topic(), chosen_exercise(), target.trim());
                if params.is_complete() {
                    navigator.push(Route::Exercise { params });
                }
            },

            label { class: "field", r#for: "page-url",
                "Neahttasiidu"
                span { class: "gloss", "Web page address" }
            }
            input {
                id: "page-url",
                class: "url-input",
                r#type: "url",
                name: "url",
                placeholder: "https://",
                autocomplete: "url",
                value: "{page_url}",
                oninput: move |event| page_url.set(event.value()),
            }

            fieldset { class: "exercises",
                legend {
                    "Hárjehus"
                    span { class: "gloss", "Exercise type" }
                }
                for kind in available.iter().copied() {
                    label { key: "{kind.id}", class: "exercise",
                        input {
                            r#type: "radio",
                            name: "client.enhancement",
                            value: kind.id,
                            checked: kind.id == chosen_exercise(),
                            onchange: move |_| chosen_exercise.set(kind.id.to_string()),
                        }
                        span { class: "exercise-sme", "{kind.sme}" }
                        span { class: "gloss", "{kind.id}" }
                    }
                }
            }

            button { r#type: "submit", class: "go", disabled: !ready, "Mana!" }
        }

        if let Some(item) = topic(&current_topic) {
            p { class: "chosen-note",
                "{item.sme}"
                span { class: "gloss", "{available.len()} exercise types available" }
            }
        }
    }
}
