//! The upload form: offer a text of your own and practise on that, rather than
//! on a page already on the web.
//!
//! The backend stores what passes its gates and answers with the URL the
//! stored copy is reached at, which is a page source like any other. So an
//! accepted text leads into exactly the exercise the entry form leads into,
//! and everything but where the page came from is the same question twice.

use std::rc::Rc;

use dioxus::html::FileData;
use dioxus::prelude::*;

use crate::api::{ApiError, Backend, DEFAULT_MODE, Registry, UploadFile, upload};
use crate::route::{ExerciseQuery, Route};
use crate::ui::SharedRegistry;
use crate::ui::choices::{Choice, Modes, Topics, first_topic};
use crate::ui::home::upload_icon;

/// The radio group asking what becomes of an accepted text, which is the one
/// question this form asks that the backend does not answer for itself.
const KEEP_GROUP: &str = "keep";

/// The file field, named here because the drop target, the label above it and
/// a refusal's way forward all point at the same input.
const FILE_FIELD: &str = "page-file";

#[component]
pub fn Upload() -> Element {
    let navigator = use_navigator();
    let backend = use_context::<Backend>();
    let registry = use_context::<SharedRegistry>();
    let value = registry.value();

    rsx! {
        match &*value.read_unchecked() {
            None => rsx! {
                p { class: "state state-pending", lang: "se",
                    "Vuorddát…"
                    span { class: "tk-gloss", "Asking the backend which topics it offers" }
                }
            },
            Some(Ok(offered)) => rsx! {
                div { class: "start",
                    div { class: "start-head",
                        h2 { "Start an exercise" }
                        p {
                            "Send a page of your own and the text is analysed once, just as a "
                            "page already on the web is — every mode then plays over it."
                        }
                    }
                    UploadPicker {
                        registry: Rc::new(offered.clone()),
                        backend: backend.clone(),
                        onstart: move |params| {
                            navigator.push(Route::Exercise { params });
                        },
                    }
                    div { class: "or", lang: "se", "dahje / or" }
                    label { class: "field", lang: "se",
                        "Vállje neahttasiiddu"
                        span { class: "tk-gloss", "Choose a web page" }
                    }
                    Link { to: Route::Home {}, class: "drop",
                        span { class: "drop-icon", {upload_icon()} }
                        span { class: "drop-body",
                            span { class: "drop-label", lang: "se", "Vállje neahttasiiddu" }
                            span { class: "drop-sub", "Practise on a page already on the web" }
                        }
                    }
                }
            },
            Some(Err(error)) => rsx! {
                p { class: "state state-error", "{error}" }
            },
        }
    }
}

/// How far an offered text has got. One that passes is left behind at once, so
/// there is no state here for a text that did.
#[derive(Clone, Debug, PartialEq)]
enum Offer {
    /// Nothing has been offered yet, or the picker was answered afresh.
    Waiting,
    /// The text is with the backend, which reads every word of it before it
    /// answers.
    Sending,
    /// A gate closed, or the request never arrived.
    Refused(ApiError),
}

/// The picker over one registry, sourced from a file rather than an address.
/// Where an accepted text leads is the caller's business, so this stands on
/// its own without the router; the backend it offers the text to is handed in
/// for the same reason.
#[component]
pub fn UploadPicker(
    registry: Rc<Registry>,
    backend: Backend,
    onstart: EventHandler<ExerciseQuery>,
) -> Element {
    let mut chosen_topic = use_signal(|| first_topic(&registry));
    let mut chosen_mode = use_signal(|| DEFAULT_MODE.to_string());
    let mut chosen_file = use_signal(|| None::<FileData>);
    let mut keep = use_signal(|| true);
    let mut offer = use_signal(|| Offer::Waiting);

    let current_topic = chosen_topic();
    let current_mode = chosen_mode();
    let current_offer = offer();
    let sending = current_offer == Offer::Sending;
    let chosen_name = chosen_file().map(|file| file.name()).unwrap_or_default();
    let ready = !chosen_name.is_empty() && !current_topic.is_empty() && !sending;
    let topic_label = registry.activity_label(&current_topic).to_string();
    let mode_label = registry.mode_label(&current_mode).to_string();
    let drop_label = if chosen_name.is_empty() {
        "Vállje fiilla".to_string()
    } else {
        chosen_name
    };

    rsx! {
        Topics {
            registry: registry.clone(),
            chosen: current_topic.clone(),
            onpick: move |name| chosen_topic.set(name),
        }

        Modes {
            registry,
            chosen: current_mode.clone(),
            onpick: move |name| chosen_mode.set(name),
        }

        div { class: "chosen-ctx",
            span { class: "tk-chip tk-chip--topic",
                span { class: "tk-sme", lang: "se", "{topic_label}" }
                span { class: "tk-gloss", "{current_topic}" }
            }
            span { class: "tk-chip",
                span { class: "tk-sme", lang: "se", "{mode_label}" }
                span { class: "tk-gloss", "{current_mode}" }
            }
        }

        form {
            class: "entry",
            onsubmit: move |event| {
                event.prevent_default();
                let Some(file) = chosen_file() else { return };
                let backend = backend.clone();
                let (topic, mode, keep) = (chosen_topic(), chosen_mode(), keep());
                spawn(async move {
                    offer.set(Offer::Sending);
                    let name = file.name();
                    let read = file
                        .read_bytes()
                        .await
                        .map_err(|error| ApiError::Network(error.to_string()));
                    let sent = match read {
                        Ok(content) => {
                            let text = UploadFile::new(name, content.to_vec()).keeping(keep);
                            upload(&backend, &text).await
                        }
                        Err(error) => Err(error),
                    };
                    match sent {
                        Ok(url) => onstart.call(ExerciseQuery::new(topic, mode, url)),
                        Err(error) => offer.set(Offer::Refused(error)),
                    }
                });
            },

            label { class: "field", r#for: FILE_FIELD, lang: "se",
                "Vállje fiilla maid háliidat geavahit"
                span { class: "tk-gloss", "Choose a file to use" }
            }
            label { class: "drop", r#for: FILE_FIELD,
                span { class: "drop-icon", {upload_icon()} }
                span { class: "drop-body",
                    span { class: "drop-label", lang: "se", "{drop_label}" }
                    span { class: "drop-sub", lang: "se",
                        "Teaksta ferte leat .html formáhtas. "
                        span { class: "tk-gloss", "· max 5 MB" }
                    }
                }
                input {
                    id: FILE_FIELD,
                    r#type: "file",
                    name: "file",
                    accept: ".html,.htm,.xhtml,text/html,application/xhtml+xml",
                    hidden: true,
                    disabled: sending,
                    onchange: move |event| {
                        chosen_file.set(event.files().into_iter().next());
                        offer.set(Offer::Waiting);
                    },
                }
            }

            div { class: "keep",
                p { class: "keep-lead", lang: "se",
                    "Sáhtát addit hárjehusa liŋkka ohppiide. Vállje nuppi molssaeavttu:"
                    span { class: "tk-gloss",
                        "The exercise keeps a link you can hand out"
                    }
                }
                div { class: "opts",
                    Choice {
                        group: KEEP_GROUP.to_string(),
                        value: true.to_string(),
                        chosen: keep(),
                        label: "Fiila vurkejuvvo nu ahte sáhtát geavahit liŋkka ođđasit eará háve."
                            .to_string(),
                        gloss: "The file is kept, so the link works again later.".to_string(),
                        onpick: move |wanted: String| keep.set(wanted == true.to_string()),
                    }
                    Choice {
                        group: KEEP_GROUP.to_string(),
                        value: false.to_string(),
                        chosen: !keep(),
                        label: "Fiila sihkkojuvvo gaskaija áigge.".to_string(),
                        gloss: "The file is deleted overnight.".to_string(),
                        onpick: move |wanted: String| keep.set(wanted == true.to_string()),
                    }
                }
            }

            div { class: "actions",
                button {
                    r#type: "submit",
                    class: "tk-btn tk-btn--primary",
                    lang: "se",
                    disabled: !ready,
                    "Sádde"
                }
            }
        }

        match current_offer {
            Offer::Waiting => rsx! {},
            Offer::Sending => rsx! {
                p { class: "state state-pending", lang: "se",
                    "Vuorddát…"
                    span { class: "tk-gloss", "The backend is reading the text you sent" }
                }
            },
            Offer::Refused(error) => rsx! {
                Refusal { error }
            },
        }
    }
}

/// Why a text was turned away.
///
/// A closed gate is the teacher's own business: it is named in North Sámi in
/// the wording the webapp this one replaces used, glossed in English, given
/// the code the endpoint wrote so a deployment can be asked about it, and
/// handed a way forward. Red never speaks alone. Anything else that went
/// wrong is not the teacher's business and reads as every other failure in
/// the app does.
#[component]
fn Refusal(error: ApiError) -> Element {
    let ApiError::Rejected(gate) = error else {
        return rsx! {
            p { class: "state state-error", "{error}" }
        };
    };

    rsx! {
        div { class: "refusal", role: "alert",
            span { class: "refusal-icon", {alert_icon()} }
            div { class: "refusal-body",
                p { class: "refusal-sme", lang: "se", "{gate.message()}" }
                p { class: "refusal-en", "{gate.gloss()}" }
                div { class: "refusal-act",
                    label { class: "tk-btn", r#for: FILE_FIELD, lang: "se", "Vállje fiilla" }
                    span { class: "code", "{gate.code()}" }
                }
            }
        }
    }
}

/// The mark a refusal carries, so the panel is not red alone.
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

#[cfg(test)]
mod tests {
    use crate::api::Rejection;

    /// Every gate the endpoint can close names itself in the panel, so a
    /// teacher reads a sentence, a code and a way forward rather than a
    /// colour.
    #[test]
    fn every_gate_is_named_in_both_languages() {
        for gate in Rejection::ALL {
            assert!(!gate.message().is_empty(), "{:?}", gate.code());
            assert!(!gate.gloss().is_empty(), "{:?}", gate.code());
            assert!(gate.code().is_ascii());
        }
    }
}
