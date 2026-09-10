//! The upload form, over a registry the backend really answered with.
//!
//! The fixture is the same `GET /api/activities` reply the picker tests read,
//! so the topics and exercises offered beside the file field are the
//! deployment's own. Nothing here reaches the network: the form is read as it
//! stands before a text has been offered, and the replies to one are read back
//! from the shapes the endpoint answers in.

use std::rc::Rc;

use dioxus::prelude::*;

use teaksta_web::api::{ApiError, Backend, Registry, Rejection, parse_registry, parse_upload};
use teaksta_web::ui::UploadPicker;

const ACTIVITIES: &str = include_str!("fixtures/activities.json");

fn offered() -> Registry {
    parse_registry(ACTIVITIES).expect("the backend's registry parses")
}

/// The upload form on its own, with nowhere for an accepted text to lead and a
/// backend nothing is ever sent to.
#[component]
fn Offered(registry: Rc<Registry>) -> Element {
    rsx! {
        UploadPicker { registry, backend: Backend::default(), onstart: move |_| {} }
    }
}

fn upload_page() -> String {
    let mut dom = VirtualDom::new_with_props(
        Offered,
        OfferedProps {
            registry: Rc::new(offered()),
        },
    );
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn the_form_asks_for_a_file() {
    let html = upload_page();

    assert!(html.contains("type=\"file\""));
    assert!(html.contains("name=\"file\""));
    assert!(!html.contains("class=\"url-input\""));
    assert!(html.contains("accept=\".html,.htm,.xhtml,text/html,application/xhtml+xml\""));
}

#[test]
fn the_usual_topics_and_exercises_are_offered() {
    let html = upload_page();

    for topic in offered().activities {
        let label = topic.label.expect("every shipped topic is named");
        assert!(html.contains(&label), "missing {label}");
    }
    for mode in offered().modes {
        let label = mode.label.expect("every shipped mode is named");
        assert!(html.contains(&label), "missing {label}");
    }
    assert_eq!(html.matches("topic-chosen").count(), 1);
    assert!(html.contains("Adverbiála"));
}

#[test]
fn nothing_is_sent_without_a_file() {
    let html = upload_page();

    // The topics are all live in this registry, so the only dead control is
    // the submit button standing over an empty picker.
    assert_eq!(html.matches("disabled").count(), 1);
    assert!(html.contains("class=\"go\""));
    assert!(html.contains("Sádde"));
    assert!(html.contains("No text has been sent yet"));
}

#[test]
fn a_file_is_kept_unless_refused() {
    let html = upload_page();

    // Four exercise modes and the two answers to what becomes of the file.
    assert_eq!(html.matches("type=\"radio\"").count(), 6);
    assert!(html.contains("Fiila vurkejuvvo nu ahte sáhtát geavahit liŋkka ođđasit eará háve."));
    assert!(html.contains("Fiila sihkkojuvvo gaskaija áigge."));
    assert_eq!(html.matches("name=\"keep\"").count(), 2);
}

#[test]
fn a_teacher_is_told_what_to_offer() {
    let html = upload_page();

    assert!(html.contains("Vállje fiilla"));
    assert!(html.contains("Teaksta ferte leat .html formáhtas."));
}

/// The four gates read to the teacher in the wording the webapp this one
/// replaces turned the same four away with.
#[test]
fn every_gate_keeps_the_old_wording() {
    assert_eq!(Rejection::NoFile.message(), "Vajálduhttet sáddet fiilla!");
    assert_eq!(
        Rejection::TooLarge.message(),
        "Fiila lea menddo stuoris! Lobálaš sturrodat: 5MB."
    );
    assert_eq!(
        Rejection::NotAPage.message(),
        "Fiilla formáhta ii leat html! Lobálaš formáhta: html."
    );
    assert_eq!(
        Rejection::NotNorthSami.message(),
        "Fiila ii sisttisdoala davvisámegiela!"
    );
}

/// The replies the endpoint answers an offered text with, as it writes them.
#[test]
fn an_accepted_text_answers_a_url() {
    let accepted = parse_upload(200, r#"{"url":"file:///srv/teaksta/upload/aB3xY9zQ1w"}"#);
    let refused = parse_upload(400, r#"{"error":"not-north-sami"}"#);

    assert_eq!(accepted.unwrap(), "file:///srv/teaksta/upload/aB3xY9zQ1w");
    assert_eq!(
        refused.unwrap_err(),
        ApiError::Rejected(Rejection::NotNorthSami)
    );
}
