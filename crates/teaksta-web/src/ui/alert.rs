//! The panel bad news is told in: something the learner or teacher came to do
//! is not going to happen, why, and the way forward. Red never speaks alone —
//! the panel carries a mark and a sentence, and the code the backend named the
//! failure by when it named one, so a deployment can be asked about it.

use dioxus::prelude::*;

/// One piece of bad news. `sme` is the North Sámi line where there is a
/// translation; `en` is always shown; the children are the way forward.
#[component]
pub fn Alert(sme: Option<String>, en: String, code: Option<String>, children: Element) -> Element {
    let sme = sme.map(|sme| rsx! { p { class: "refusal-sme", lang: "se", "{sme}" } });
    let code = code.map(|code| rsx! { span { class: "code", "{code}" } });

    rsx! {
        div { class: "refusal", role: "alert",
            span { class: "refusal-icon", {alert_icon()} }
            div { class: "refusal-body",
                {sme}
                p { class: "refusal-en", "{en}" }
                div { class: "refusal-act", {children} {code} }
            }
        }
    }
}

/// The mark the panel carries, so it is not red alone.
fn alert_icon() -> Element {
    rsx! {
        svg {
            width: "16",
            height: "16",
            view_box: "0 0 16 16",
            "aria-hidden": "true",
            path {
                d: "M8 4v5M8 11.6v.1",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
            }
        }
    }
}
