//! What the exercise page shows when the page it was asked for could not be
//! read: the panel bad news is told in, naming the site, the failure's code,
//! and the way back to choosing another page.
//!
//! The panel carries a `Link`, so it is rendered inside a router of this
//! file's own, as `tests/uploads.rs` renders its panels.

use std::rc::Rc;

use dioxus::history::{History, MemoryHistory};
use dioxus::prelude::dioxus_router::components::HistoryProvider;
use dioxus::prelude::*;

use teaksta_web::api::{ApiError, PageFailure, failed};
use teaksta_web::ui::exercise::Unread;

/// The page that was reported unreadable, on a site that answered 502 for
/// every page it holds.
const REPORTED: &str = "https://kursa.oahpa.no/2013/12/30/ruovttus-leat-ollu-barggut/";

#[derive(Clone, PartialEq)]
struct Shown {
    failure: PageFailure,
    url: String,
}

#[derive(Routable, Clone, Debug, PartialEq)]
enum Harness {
    #[route("/")]
    Panel {},
}

#[component]
fn Panel() -> Element {
    let shown = use_context::<Shown>();
    rsx! {
        Unread { failure: shown.failure, url: shown.url.clone() }
    }
}

#[component]
fn Harnessed(shown: Shown) -> Element {
    use_context_provider(|| shown.clone());

    rsx! {
        HistoryProvider {
            history: move |_| Rc::new(MemoryHistory::with_initial_path("/")) as Rc<dyn History>,
            Router::<Harness> {}
        }
    }
}

fn render(failure: PageFailure, url: &str) -> String {
    let mut dom = VirtualDom::new_with_props(
        Harnessed,
        HarnessedProps {
            shown: Shown {
                failure,
                url: url.to_string(),
            },
        },
    );
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// The reported case, from the reply the backend sends to the panel the
/// learner reads: the site is named with what it answered, the failure is
/// said to be the site's, and the way forward is another page.
#[test]
fn a_site_that_is_down_is_named() {
    let ApiError::Page(failure) = failed(502, r#"{"error":"site-error","status":502}"#) else {
        panic!("a coded reply is read as the failure it names");
    };
    let html = render(failure, REPORTED);

    assert!(html.contains("class=\"refusal\""));
    assert!(html.contains("role=\"alert\""));
    assert!(html.contains("kursa.oahpa.no answered with an error (502)."));
    assert!(html.contains("not teaksta"));
    assert!(html.contains("class=\"code\">site-error<"));
    assert!(html.contains("href=\"/\""));
    assert!(html.contains("Vállje neahttasiiddu"));
    assert!(!html.contains("backend answered"));
}

#[test]
fn a_stored_text_names_no_site() {
    let html = render(
        PageFailure::Unreachable,
        "/api/texts/00000000000000000000000000000000",
    );

    assert!(html.contains("The text could not be read."));
    assert!(html.contains("class=\"code\">page-unreachable<"));
}

/// A reply naming nothing the app knows still says what the backend
/// answered, rather than inventing a reason.
#[test]
fn an_uncoded_reply_keeps_its_status() {
    assert_eq!(failed(500, "the analyser fell over"), ApiError::Status(500));
    assert_eq!(
        failed(400, "mode must be colorize, click, mc or cloze"),
        ApiError::Status(400)
    );
}
