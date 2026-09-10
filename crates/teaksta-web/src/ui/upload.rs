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

/// The radio group asking what becomes of an accepted text, which is the one
/// question this form asks that the backend does not answer for itself.
const KEEP_GROUP: &str = "keep";

#[component]
pub fn Upload() -> Element {
    let navigator = use_navigator();
    let backend = use_context::<Backend>();
    let registry = use_context::<SharedRegistry>();
    let value = registry.value();

    rsx! {
        match &*value.read_unchecked() {
            None => rsx! {
                p { class: "state state-pending",
                    "Vuorddát…"
                    span { class: "gloss", "Asking the backend which topics it offers" }
                }
            },
            Some(Ok(offered)) => rsx! {
                UploadPicker {
                    registry: Rc::new(offered.clone()),
                    backend: backend.clone(),
                    onstart: move |params| {
                        navigator.push(Route::Exercise { params });
                    },
                }
                Link { to: Route::Home {}, class: "swap",
                    "Vállje neahttasiiddu"
                    span { class: "gloss", "Practise on a web page instead" }
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
    let current_offer = offer();
    let sending = current_offer == Offer::Sending;
    let chosen_name = chosen_file().map(|file| file.name()).unwrap_or_default();
    let ready = !chosen_name.is_empty() && !current_topic.is_empty() && !sending;

    rsx! {
        Topics {
            registry: registry.clone(),
            chosen: current_topic,
            onpick: move |name| chosen_topic.set(name),
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

            label { class: "field", r#for: "page-file",
                "Vállje fiilla"
                span { class: "gloss", "Choose a file" }
            }
            input {
                id: "page-file",
                class: "file-input",
                r#type: "file",
                name: "file",
                accept: ".html,.htm,.xhtml,text/html,application/xhtml+xml",
                disabled: sending,
                onchange: move |event| {
                    chosen_file.set(event.files().into_iter().next());
                    offer.set(Offer::Waiting);
                },
            }
            p { class: "file-note",
                "Teaksta ferte leat .html formáhtas."
                span { class: "gloss", "The text has to be an HTML page, at most 5 MB" }
            }

            fieldset { class: "keeping",
                legend {
                    "Fiila"
                    span { class: "gloss", "What becomes of the file" }
                }
                p { class: "keep-lead",
                    "Sáhtát addit hárjehusa liŋkka ohppiide. Vállje nuppi molssaeavttu:"
                    span { class: "gloss", "The exercise keeps a link you can hand out" }
                }
                Choice {
                    group: KEEP_GROUP.to_string(),
                    value: true.to_string(),
                    chosen: keep(),
                    label: "Fiila vurkejuvvo nu ahte sáhtát geavahit liŋkka ođđasit eará háve."
                        .to_string(),
                    gloss: "Keep the file, so the link works another day".to_string(),
                    onpick: move |wanted: String| keep.set(wanted == true.to_string()),
                }
                Choice {
                    group: KEEP_GROUP.to_string(),
                    value: false.to_string(),
                    chosen: !keep(),
                    label: "Fiila sihkkojuvvo gaskaija áigge.".to_string(),
                    gloss: "Sweep the file away at midnight".to_string(),
                    onpick: move |wanted: String| keep.set(wanted == true.to_string()),
                }
            }

            Modes {
                registry,
                chosen: chosen_mode(),
                onpick: move |name| chosen_mode.set(name),
            }

            button { r#type: "submit", class: "go", disabled: !ready, "Sádde" }
        }

        match current_offer {
            Offer::Waiting => rsx! {
                p { class: "chosen-note",
                    "{chosen_name}"
                    span { class: "gloss", "No text has been sent yet" }
                }
            },
            Offer::Sending => rsx! {
                p { class: "state state-pending",
                    "Vuorddát…"
                    span { class: "gloss", "The backend is reading the text you sent" }
                }
            },
            Offer::Refused(error) => rsx! {
                Refusal { error }
            },
        }
    }
}

/// Why a text was turned away. A closed gate is the teacher's own business and
/// is named in North Sámi; anything else that went wrong is not, and reads as
/// every other failure in the app does.
#[component]
fn Refusal(error: ApiError) -> Element {
    match error {
        ApiError::Rejected(rejection) => rsx! {
            p { class: "state state-error",
                "{rejection.message()}"
                span { class: "gloss", "{rejection.gloss()}" }
            }
        },
        error => rsx! {
            p { class: "state state-error", "{error}" }
        },
    }
}
