//! The topic picker, rendered from a registry the backend really answered with.
//!
//! The fixture is a `GET /api/activities` reply saved as it arrived, so the
//! North Sámi names shown here are the deployment's own.

use std::rc::Rc;

use dioxus::prelude::*;

use teaksta_web::api::{Registry, parse_registry};
use teaksta_web::ui::Picker;

const ACTIVITIES: &str = include_str!("fixtures/activities.json");

fn offered() -> Registry {
    parse_registry(ACTIVITIES).expect("the backend's registry parses")
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

fn picker_page() -> String {
    render(offered())
}

#[test]
fn the_registry_names_every_topic_and_mode() {
    let registry = offered();

    assert_eq!(registry.activities.len(), 10);
    assert_eq!(registry.modes.len(), 4);
    assert!(registry.activities.iter().all(|topic| topic.enabled));
    assert_eq!(
        registry.activity_label("SubstantivePlural"),
        "Substantiivvat máŋggaidlogus"
    );
    assert_eq!(registry.mode_label("mc"), "Vállje rivttes sániid!");
}

#[test]
fn the_picker_lists_every_topic_in_sami() {
    let html = picker_page();

    for topic in offered().activities {
        let label = topic.label.expect("every shipped topic is named");
        assert!(html.contains(&label), "missing {label}");
        assert!(html.contains(&topic.name), "missing {}", topic.name);
    }
}

/// The four modes are a segmented switch rather than a radio list: they are
/// four ways into one text, and the switch says so. Each segment carries the
/// backend's own name for the mode, and the chosen one's North Sámi
/// instruction is spelled out under the row in both languages.
#[test]
fn the_form_offers_a_url_field_and_modes() {
    let html = picker_page();

    assert!(html.contains("name=\"url\""));
    assert_eq!(html.matches("role=\"tab\"").count(), 4);
    assert_eq!(html.matches("aria-selected=\"true\"").count(), 1);
    assert!(!html.contains("type=\"radio\""));
    for mode in offered().modes {
        assert!(
            html.contains(&format!("class=\"seg-name\">{}<", mode.name)),
            "missing {}",
            mode.name
        );
    }
    assert!(html.contains("Geahča ivdnejuvvon sániid."));
    assert!(html.contains("Look at the coloured words."));
}

#[test]
fn the_first_topic_starts_out_chosen() {
    let html = picker_page();

    assert_eq!(html.matches("topic-chosen").count(), 1);
    assert!(html.contains("aria-pressed=\"true\""));
    assert!(html.contains("Adverbiála"));
}

/// A topic a deployment has switched off stays in the list and sunken rather
/// than disappearing, so the list never changes shape.
#[test]
fn a_topic_the_backend_switched_off_is_dead() {
    let mut registry = offered();
    registry.activities[0].enabled = false;

    let html = render(registry);

    // Ten topics, four mode segments, and the button that sends the form.
    assert_eq!(html.matches("<button").count(), 15);
    assert_eq!(html.matches("class=\"topic\"").count(), 9);
    assert_eq!(html.matches("disabled").count(), 2);
    assert!(html.contains("Adverbiála"));
    assert!(html.contains("Konjunkšuvnnat"));
}
