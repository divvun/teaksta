//! The site chrome every view sits inside, and the one registry fetch the
//! views beneath it share.
//!
//! A quiet bar over a centred column, and nothing else. The wordmark holds
//! the only weight up there and the ways into an exercise sit beside it as
//! plain links, so the analysed text below is the heaviest thing on any page
//! of the app.
//!
//! How many ways there are is the deployment's answer rather than this bar's.
//! The file link stands beside the page one once the registry has said the
//! backend takes a teacher's own text; until it has, and on a deployment that
//! never will, the bar carries the one way in that works. A link arriving with
//! the panel it belongs beside is better than a link that was always there and
//! leads somewhere refusing.

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

    let answered = registry.value();
    let uploads = matches!(&*answered.read_unchecked(), Some(Ok(offered)) if offered.uploads);

    rsx! {
        header { class: "bar",
            Link { to: Route::Home {}, class: "tk-wordmark", "teaksta" }
            nav { class: "bar-right",
                Link { to: Route::Home {}, class: "bar-link", lang: "se", "Vállje neahttasiiddu" }
                if uploads {
                    Link { to: Route::Upload {}, class: "bar-link", lang: "se", "Vállje fiilla" }
                }
            }
        }
        main { class: "col chrome-main", Outlet::<Route> {} }
        p { class: "foot", lang: "se", "Divvun · UiT Norgga árktalaš universitehta" }
    }
}
