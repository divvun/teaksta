//! The bilingual lines more than one view shows, each read from the message
//! bundles under one id: the North Sámi, and the English glossing it.

use dioxus::prelude::*;
use teaksta_l10n::{en, se_or_en};

/// The line shown while the backend is asked for something.
#[component]
pub fn Pending(id: String) -> Element {
    rsx! {
        p { class: "state state-pending", lang: "se",
            {se_or_en(&id)}
            span { class: "tk-gloss", {en(&id)} }
        }
    }
}

/// The seam between the two ways in.
#[component]
pub fn Or() -> Element {
    let or = format!("{} / {}", se_or_en("or"), en("or"));
    rsx! {
        div { class: "or", lang: "se", "{or}" }
    }
}

/// What a file must be, under the file route's label.
#[component]
pub fn FileLimits() -> Element {
    let format = se_or_en("file-format");
    let limit = en("file-limit");
    rsx! {
        span { class: "drop-sub", lang: "se",
            "{format} "
            span { class: "tk-gloss", "· {limit}" }
        }
    }
}
