//! What the two entry panels look like when the backend takes a teacher's own
//! text, and what they look like when it does not.
//!
//! The registry is the same `GET /api/activities` reply the picker tests read,
//! so the deployment that says yes here is a real one; the deployment that
//! says no is that reply with the one field turned over, which is exactly what
//! a backend with nowhere to keep a text answers.
//!
//! Both panels carry `Link`s, which need a router over them, so they are
//! rendered inside a router of this file's own rather than the app's — the
//! app's would route to `Home`, which is still waiting on a registry no
//! browser is here to fetch. What a `Link` writes is an address, so a router
//! with one route renders them the same way the app's does.

use std::rc::Rc;

use dioxus::history::{History, MemoryHistory};
use dioxus::prelude::dioxus_router::components::HistoryProvider;
use dioxus::prelude::*;

use teaksta_web::api::{Backend, Registry, parse_registry};
use teaksta_web::ui::{Start, UploadStart};

const ACTIVITIES: &str = include_str!("fixtures/activities.json");

/// The registry the backend answered, which is a deployment that takes
/// uploads — and the same one with that turned off.
fn offered(uploads: bool) -> Registry {
    let mut registry = parse_registry(ACTIVITIES).expect("the backend's registry parses");
    assert!(
        registry.uploads,
        "the saved reply is a deployment that does"
    );
    registry.uploads = uploads;
    registry
}

/// Which of the two panels is under test, with the registry it reads.
#[derive(Clone, PartialEq)]
struct Panelled {
    registry: Rc<Registry>,
    file_form: bool,
}

#[derive(Routable, Clone, Debug, PartialEq)]
enum Harness {
    #[route("/")]
    Panel {},
}

#[component]
fn Panel() -> Element {
    let panelled = use_context::<Panelled>();

    if panelled.file_form {
        rsx! {
            UploadStart {
                registry: panelled.registry.clone(),
                backend: Backend::default(),
                onstart: move |_| {},
            }
        }
    } else {
        rsx! {
            Start { registry: panelled.registry.clone(), onstart: move |_| {} }
        }
    }
}

#[component]
fn Harnessed(panelled: Panelled) -> Element {
    use_context_provider(|| panelled.clone());

    rsx! {
        HistoryProvider {
            history: move |_| Rc::new(MemoryHistory::with_initial_path("/")) as Rc<dyn History>,
            Router::<Harness> {}
        }
    }
}

fn render(registry: Registry, file_form: bool) -> String {
    let mut dom = VirtualDom::new_with_props(
        Harnessed,
        HarnessedProps {
            panelled: Panelled {
                registry: Rc::new(registry),
                file_form,
            },
        },
    );
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// A deployment that keeps texts offers the second way in: the seam under the
/// picker, and the file route it leads to.
#[test]
fn the_file_route_is_offered_where_it_answers() {
    let html = render(offered(true), false);

    assert!(html.contains("class=\"or\""));
    assert!(html.contains("dahje / or"));
    assert!(html.contains("href=\"/upload\""));
    assert!(html.contains("Vállje fiilla"));
    // The page form is still the panel's own business and is untouched.
    assert!(html.contains("name=\"url\""));
}

/// One that does not keeps the page form and loses the seam entirely: no way
/// across, and no mention of sending a file to a backend that would refuse it.
#[test]
fn no_store_hides_the_file_route() {
    let html = render(offered(false), false);

    assert!(!html.contains("class=\"or\""));
    assert!(!html.contains("dahje / or"));
    assert!(!html.contains("/upload"));
    assert!(!html.contains("Vállje fiilla"));
    // The one way in that works is offered exactly as it always was.
    assert!(html.contains("name=\"url\""));
    assert!(html.contains("Vállje neahttasiiddu"));
    assert_eq!(html.matches("role=\"tab\"").count(), 4);
}

/// Nothing leads to the file route on such a deployment, but the address is
/// an address and somebody arrives at it anyway. They are told what this
/// deployment does not do, and pointed at what it does.
#[test]
fn the_file_route_says_so_instead() {
    let html = render(offered(false), true);

    assert!(html.contains("class=\"refusal\""));
    assert!(html.contains("role=\"alert\""));
    assert!(html.contains("This deployment does not accept uploads."));
    // Nothing to fill in and nothing to send: no file field, no keep
    // question, no *Sádde*.
    assert!(!html.contains("type=\"file\""));
    assert!(!html.contains("type=\"radio\""));
    assert!(!html.contains("Sádde"));
    // and the way forward is the route that works.
    assert!(html.contains("href=\"/\""));
    assert!(html.contains("Vállje neahttasiiddu"));
}

/// Where it does answer, the file route is the form it always was.
#[test]
fn the_file_route_is_the_form_otherwise() {
    let html = render(offered(true), true);

    assert!(html.contains("type=\"file\""));
    assert!(html.contains("Sádde"));
    assert!(!html.contains("This deployment does not accept uploads."));
    // and its own seam leads back across to naming a page.
    assert!(html.contains("dahje / or"));
    assert!(html.contains("href=\"/\""));
}
