//! The entry form: pick a grammar topic, name a page, pick an exercise mode.
//!
//! Both lists come from the backend's registry rather than from a table of
//! this app's own, so a topic is named here exactly as the deployment names it.

use std::rc::Rc;

use dioxus::prelude::*;

use crate::api::{DEFAULT_MODE, Registry};
use crate::route::{ExerciseQuery, Route};
use crate::ui::SharedRegistry;
use crate::ui::choices::{Modes, Topics, first_topic};

#[component]
pub fn Home() -> Element {
    let navigator = use_navigator();
    let registry = use_context::<SharedRegistry>();
    let value = registry.value();

    rsx! {
        match &*value.read_unchecked() {
            None => rsx! {
                p { class: "state state-pending",
                    "Vuorddát…"
                    span { class: "gloss", "Asking the backend which topics it offers" }
                }
            },
            Some(Ok(offered)) => rsx! {
                Picker {
                    registry: Rc::new(offered.clone()),
                    onstart: move |params| {
                        navigator.push(Route::Exercise { params });
                    },
                }
                Link { to: Route::Upload {}, class: "swap",
                    "Vállje fiilla maid háliidat geavahit"
                    span { class: "gloss", "Practise on a text of your own instead" }
                }
            },
            Some(Err(error)) => rsx! {
                p { class: "state state-error", "{error}" }
            },
        }
    }
}

/// The picker over one registry: the topics as buttons, the page address, and
/// the exercise modes as radios. Where a filled-in form leads is the caller's
/// business, so the picker stands on its own without the router.
#[component]
pub fn Picker(registry: Rc<Registry>, onstart: EventHandler<ExerciseQuery>) -> Element {
    let mut chosen_topic = use_signal(|| first_topic(&registry));
    let mut chosen_mode = use_signal(|| DEFAULT_MODE.to_string());
    let mut page_url = use_signal(String::new);

    let current_topic = chosen_topic();
    let ready = !page_url().trim().is_empty() && !current_topic.is_empty();
    let chosen_label = registry.activity_label(&current_topic).to_string();
    let offered_modes = registry.modes.len();

    rsx! {
        Topics {
            registry: registry.clone(),
            chosen: current_topic,
            onpick: move |name| chosen_topic.set(name),
        }

        form {
            class: "entry",
            onsubmit: move |event| {
                event.prevent_default();
                let target = page_url();
                let params = ExerciseQuery::new(chosen_topic(), chosen_mode(), target.trim());
                if params.is_complete() {
                    onstart.call(params);
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

            Modes {
                registry,
                chosen: chosen_mode(),
                onpick: move |name| chosen_mode.set(name),
            }

            button { r#type: "submit", class: "go", disabled: !ready, "Mana!" }
        }

        p { class: "chosen-note",
            "{chosen_label}"
            span { class: "gloss", "{offered_modes} exercise types available" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::api::{Activity, Mode};

    fn topic(name: &str, label: Option<&str>, enabled: bool) -> Activity {
        Activity {
            name: name.to_string(),
            label: label.map(str::to_string),
            enabled,
        }
    }

    /// A registry shaped like the backend's, with the first topic switched off
    /// and the second carrying no North Sámi name.
    fn offered() -> Registry {
        Registry {
            activities: vec![
                topic("Adverbial", Some("Adverbiála"), false),
                topic("Preps", None, true),
                topic("Subject", Some("Subjeakta"), true),
            ],
            modes: vec![
                Mode {
                    name: "colorize".to_string(),
                    label: Some("Geahča ivdnejuvvon sániid.".to_string()),
                },
                Mode {
                    name: "cloze".to_string(),
                    label: None,
                },
            ],
        }
    }

    /// The picker on its own, with nowhere for a filled-in form to lead.
    #[component]
    fn Offered(registry: Rc<Registry>) -> Element {
        rsx! {
            Picker { registry, onstart: move |_| {} }
        }
    }

    fn render(registry: Registry) -> String {
        let mut dom = VirtualDom::new_with_props(
            Offered,
            OfferedProps {
                registry: Rc::new(registry),
            },
        );
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn every_offered_topic_reaches_the_page() {
        let html = render(offered());

        assert_eq!(html.matches("aria-pressed").count(), 3);
        for name in ["Adverbial", "Preps", "Subject"] {
            assert!(html.contains(name), "missing {name}");
        }
    }

    #[test]
    fn a_topic_with_no_label_shows_its_name() {
        let html = render(offered());

        assert!(html.contains("class=\"topic-sme\">Preps<"));
        assert!(html.contains("class=\"topic-sme\">Adverbiála<"));
    }

    #[test]
    fn a_topic_switched_off_cannot_be_chosen() {
        let html = render(offered());

        assert!(html.contains("disabled"));
        assert_eq!(html.matches("topic-chosen").count(), 1);
        assert!(!html.contains(
            "topic topic-chosen\" aria-pressed=\"true\"><span class=\"topic-sme\">Adverbiála"
        ));
    }

    #[test]
    fn the_first_live_topic_starts_out_chosen() {
        let html = render(offered());
        let chosen = html
            .find("topic-chosen")
            .expect("one topic starts out chosen");

        assert!(html[chosen..].contains("Preps"));
        assert_eq!(html.matches("aria-pressed=\"true\"").count(), 1);
    }

    #[test]
    fn every_mode_is_offered_for_every_topic() {
        let html = render(offered());

        assert_eq!(html.matches("type=\"radio\"").count(), 2);
        assert!(html.contains("Geahča ivdnejuvvon sániid."));
        assert!(html.contains("class=\"exercise-sme\">cloze<"));
    }

    #[test]
    fn a_registry_with_no_topics_still_renders() {
        let html = render(Registry::default());

        assert!(html.contains("class=\"topics\""));
        assert!(!html.contains("topic-chosen"));
        assert!(html.contains("0 exercise types available"));
    }
}
