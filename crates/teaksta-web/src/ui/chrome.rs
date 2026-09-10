//! The site chrome every view sits inside, and the one registry fetch the
//! views beneath it share.

use dioxus::prelude::*;

use crate::api::{Backend, fetch_registry};
use crate::route::Route;

#[component]
pub fn Chrome() -> Element {
    let backend = use_context::<Backend>();
    let registry = use_resource(move || {
        let backend = backend.clone();
        async move { fetch_registry(&backend).await }
    });
    use_context_provider(|| registry);

    rsx! {
        div { class: "shell",
            header { class: "chrome",
                Link { to: Route::Home {}, class: "wordmark", "Teaksta" }
                p { class: "chrome-lead",
                    "Hárjehala sámegiela neahttasiidduin maid ieš válljet."
                    span { class: "gloss",
                        "Practise North Sámi grammar on web pages you choose yourself."
                    }
                }
            }
            main { class: "chrome-main", Outlet::<Route> {} }
            footer { class: "chrome-foot", "Teaksta · Divvun · Giellatekno" }
        }
    }
}
