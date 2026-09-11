//! The entry form: pick a grammar topic, pick an exercise mode, name a page.
//!
//! Which topics and which modes are on offer comes from the backend's
//! registry rather than from a table of this app's own, so a topic is named
//! here exactly as the deployment names it.
//!
//! One panel, two ways in. Naming a page and sending a text of your own ask
//! the same two questions and differ only in where the page comes from, so
//! the topic and the mode are chosen once at the top and the seam below them
//! leads across to the other route. They really are two forms, and each ends
//! in its own *Sádde*.

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
                p { class: "state state-pending", lang: "se",
                    "Vuorddát…"
                    span { class: "tk-gloss", "Asking the backend which topics it offers" }
                }
            },
            Some(Ok(offered)) => rsx! {
                div { class: "start",
                    div { class: "start-head",
                        h2 { "Start an exercise" }
                        p {
                            "Any North Sámi page will do. Name one, or send a file of your own — "
                            "either way the text is analysed once and every mode plays over it."
                        }
                    }
                    Picker {
                        registry: Rc::new(offered.clone()),
                        onstart: move |params| {
                            navigator.push(Route::Exercise { params });
                        },
                    }
                    div { class: "or", lang: "se", "dahje / or" }
                    label { class: "field", lang: "se",
                        "Vállje fiilla maid háliidat geavahit"
                        span { class: "tk-gloss", "Choose a file to use" }
                    }
                    Link { to: Route::Upload {}, class: "drop",
                        span { class: "drop-icon", {upload_icon()} }
                        span { class: "drop-body",
                            span { class: "drop-label", lang: "se", "Vállje fiilla" }
                            span { class: "drop-sub", lang: "se",
                                "Teaksta ferte leat .html formáhtas. "
                                span { class: "tk-gloss", "· max 5 MB" }
                            }
                        }
                    }
                }
            },
            Some(Err(error)) => rsx! {
                p { class: "state state-error", "{error}" }
            },
        }
    }
}

/// The picker over one registry: the topics, the exercise modes, and the page
/// address. Where a filled-in form leads is the caller's business, so the
/// picker stands on its own without the router.
#[component]
pub fn Picker(registry: Rc<Registry>, onstart: EventHandler<ExerciseQuery>) -> Element {
    let mut chosen_topic = use_signal(|| first_topic(&registry));
    let mut chosen_mode = use_signal(|| DEFAULT_MODE.to_string());
    let mut page_url = use_signal(String::new);

    let current_topic = chosen_topic();
    let current_mode = chosen_mode();
    let ready = !page_url().trim().is_empty() && !current_topic.is_empty();
    let topic_label = registry.activity_label(&current_topic).to_string();
    let mode_label = registry.mode_label(&current_mode).to_string();

    rsx! {
        Topics {
            registry: registry.clone(),
            chosen: current_topic.clone(),
            onpick: move |name| chosen_topic.set(name),
        }

        Modes {
            registry,
            chosen: current_mode.clone(),
            onpick: move |name| chosen_mode.set(name),
        }

        div { class: "chosen-ctx",
            span { class: "tk-chip tk-chip--topic",
                span { class: "tk-sme", lang: "se", "{topic_label}" }
                span { class: "tk-gloss", "{current_topic}" }
            }
            span { class: "tk-chip",
                span { class: "tk-sme", lang: "se", "{mode_label}" }
                span { class: "tk-gloss", "{current_mode}" }
            }
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

            label { class: "field", r#for: "page-url", lang: "se",
                "Vállje neahttasiiddu"
                span { class: "tk-gloss", "Choose a web page" }
            }
            div { class: "row",
                input {
                    id: "page-url",
                    class: "input",
                    r#type: "url",
                    name: "url",
                    inputmode: "url",
                    placeholder: "https://oahpa.no/…",
                    autocomplete: "url",
                    value: "{page_url}",
                    oninput: move |event| page_url.set(event.value()),
                }
                button {
                    r#type: "submit",
                    class: "tk-btn tk-btn--primary",
                    lang: "se",
                    disabled: !ready,
                    "Sádde"
                }
            }
        }
    }
}

/// The mark on the file route: a page leaving the teacher's own machine.
pub fn upload_icon() -> Element {
    rsx! {
        svg {
            width: "20",
            height: "20",
            view_box: "0 0 20 20",
            "aria-hidden": "true",
            path {
                d: "M10 14V3.5M10 3.5L6.2 7.3M10 3.5l3.8 3.8M3 13v2.5A1.5 1.5 0 004.5 17h11a1.5 1.5 0 001.5-1.5V13",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.6",
                stroke_linecap: "round",
                stroke_linejoin: "round",
            }
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

        assert!(html.contains(">Preps<"));
        assert!(html.contains(">Adverbiála<"));
    }

    #[test]
    fn a_topic_switched_off_cannot_be_chosen() {
        let html = render(offered());

        assert!(html.contains("disabled"));
        assert_eq!(html.matches("topic-chosen").count(), 1);
        assert!(!html.contains("topic topic-chosen\" aria-pressed=\"true\" disabled"));
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

    /// Every topic is offered every mode, so the switch is built from the
    /// registry's mode list alone and reads the same whichever topic is on.
    #[test]
    fn every_mode_is_offered_for_every_topic() {
        let html = render(offered());

        assert_eq!(html.matches("role=\"tab\"").count(), 2);
        assert_eq!(html.matches("aria-selected=\"true\"").count(), 1);
        assert!(html.contains("Geahča ivdnejuvvon sániid."));
        assert!(html.contains("class=\"seg-name\">cloze<"));
        assert!(html.contains("class=\"seg-hint\">Type it<"));
    }

    /// The chosen topic and mode are shown as chips as well, because they
    /// travel with whichever of the two routes the teacher takes.
    #[test]
    fn the_chosen_pair_is_summed_up() {
        let html = render(offered());

        assert!(html.contains("class=\"chosen-ctx\""));
        assert!(html.contains("tk-chip tk-chip--topic"));
        assert!(html.contains("Geahča ivdnejuvvon sániid."));
    }

    #[test]
    fn a_registry_with_no_topics_still_renders() {
        let html = render(Registry::default());

        assert!(html.contains("class=\"topics\""));
        assert!(!html.contains("topic-chosen"));
        assert!(html.contains("0 of 0"));
    }
}
