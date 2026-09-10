//! Teaksta's web client: pick a North Sámi grammar topic and an exercise
//! type, name a page, and read that page with the exercise woven into it.

pub mod activities;
pub mod api;
pub mod route;
pub mod ui;

use dioxus::prelude::*;

use crate::api::Backend;
use crate::route::Route;

pub const STYLE: Asset = asset!("/assets/main.css");

/// The app root: site-wide state, the stylesheet and the router.
#[component]
pub fn App() -> Element {
    use_context_provider(Backend::default);

    rsx! {
        document::Stylesheet { href: STYLE }
        document::Title { "Teaksta" }
        Router::<Route> {}
    }
}
