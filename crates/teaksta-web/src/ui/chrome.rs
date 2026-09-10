//! The site chrome every view sits inside.

use dioxus::prelude::*;

use crate::route::Route;

#[component]
pub fn Chrome() -> Element {
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
