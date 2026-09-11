//! The site chrome every view sits inside, and the one registry fetch the
//! views beneath it share.
//!
//! A quiet bar over a centred column, and nothing else. The wordmark holds
//! the only weight up there and the two ways into an exercise sit beside it
//! as plain links, so the analysed text below is the heaviest thing on any
//! page of the app.

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
        header { class: "bar",
            Link { to: Route::Home {}, class: "tk-wordmark", "teaksta" }
            nav { class: "bar-right",
                Link { to: Route::Home {}, class: "bar-link", lang: "se", "Vállje neahttasiiddu" }
                Link { to: Route::Upload {}, class: "bar-link", lang: "se", "Vállje fiilla" }
            }
        }
        main { class: "col chrome-main", Outlet::<Route> {} }
        p { class: "foot", lang: "se", "Divvun · UiT Norgga árktalaš universitehta" }
    }
}
