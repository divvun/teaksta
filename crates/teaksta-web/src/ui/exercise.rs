//! The exercise view: the chosen parameters, the backend request they map to,
//! and the state of that request, rendered in the panel that holds the
//! enhanced page.

use dioxus::prelude::*;

use crate::activities::{exercise_type, topic};
use crate::api::{ApiError, Backend, EnhanceRequest, fetch_enhanced};
use crate::route::{ExerciseQuery, Route};

#[component]
pub fn Exercise(params: ExerciseQuery) -> Element {
    let backend = use_context::<Backend>();
    let request = EnhanceRequest::new(params.url.clone(), params.topic.clone(), &params.exercise);
    let target = backend.enhance_url(&request);

    let page = use_resource(use_reactive!(|request| {
        let backend = backend.clone();
        async move {
            if request.url.is_empty() || request.activity.is_empty() {
                return Err(ApiError::Incomplete);
            }
            fetch_enhanced(&backend, &request).await
        }
    }));

    let topic_sme = topic(&params.topic).map(|item| item.sme);
    let exercise_sme = exercise_type(&params.exercise).map(|kind| kind.sme);
    let value = page.value();

    rsx! {
        section { class: "exercise",
            h2 {
                {topic_sme.unwrap_or("—")}
                span { class: "gloss", "{params.topic}" }
            }
            p { class: "instruction", {exercise_sme.unwrap_or("—")} }

            dl { class: "params",
                dt { "Neahttasiidu" }
                dd { class: "param-url", "{params.url}" }
                dt { "Hárjehus" }
                dd { "{params.exercise}" }
                dt { "Bálvá" }
                dd { class: "param-url", "{target}" }
            }

            div { class: "panel",
                match &*value.read_unchecked() {
                    None => rsx! {
                        p { class: "state state-pending",
                            "Vuorddát…"
                            span { class: "gloss", "Asking the backend for the enhanced page" }
                        }
                    },
                    Some(Ok(html)) => rsx! {
                        p { class: "state state-ready",
                            "Gárvvis"
                            span { class: "gloss", "{html.len()} bytes of enhanced page received" }
                        }
                    },
                    Some(Err(error)) => rsx! {
                        p { class: "state state-error", "{error}" }
                    },
                }
            }

            Link { to: Route::Home {}, class: "back",
                "Ruovttoluotta"
                span { class: "gloss", "Back to the form" }
            }
        }
    }
}
