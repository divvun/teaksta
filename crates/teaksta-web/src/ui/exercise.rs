//! The exercise view: the chosen parameters, the backend request they map to,
//! and the enhanced page itself, woven into whichever exercise was asked for.

pub mod click;
pub mod cloze;
pub mod colorize;
pub mod markup;
pub mod mc;

use std::rc::Rc;

use dioxus::prelude::*;

use crate::api::{Backend, EnhanceRequest, fetch_enhanced};
use crate::route::{ExerciseQuery, Route};
use crate::ui::SharedRegistry;

use click::ClickMode;
use cloze::ClozeMode;
use colorize::ColorizeMode;
use markup::{Block, BlockKind, Markup, Piece, TokenSpan};
use mc::McMode;

#[component]
pub fn Exercise(params: ExerciseQuery) -> Element {
    let backend = use_context::<Backend>();
    let request = EnhanceRequest::new(params.url.clone(), params.topic.clone(), &params.mode);
    let target = backend.enhance_url(&request);

    let page = use_resource(use_reactive!(|request| {
        let backend = backend.clone();
        async move { fetch_enhanced(&backend, &request).await }
    }));

    let registry = use_context::<SharedRegistry>();
    let named = registry.value();
    let (topic_label, mode_label) = match &*named.read_unchecked() {
        Some(Ok(offered)) => (
            offered.activity_label(&params.topic).to_string(),
            offered.mode_label(&params.mode).to_string(),
        ),
        _ => (params.topic.clone(), params.mode.clone()),
    };
    let value = page.value();

    rsx! {
        section { class: "exercise",
            h2 {
                "{topic_label}"
                span { class: "gloss", "{params.topic}" }
            }
            p { class: "instruction", "{mode_label}" }

            dl { class: "params",
                dt { "Neahttasiidu" }
                dd { class: "param-url", "{params.url}" }
                dt { "Hárjehus" }
                dd { "{params.mode}" }
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
                        EnhancedPage {
                            html: html.clone(),
                            topic: params.topic.clone(),
                            mode: params.mode.clone(),
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

/// One enhanced page, read once and handed to the exercise that was asked
/// for. An unknown mode reads as colorize, which every topic offers.
#[component]
pub fn EnhancedPage(html: String, topic: String, mode: String) -> Element {
    let parsed = use_memo(use_reactive!(|html| Rc::new(markup::parse(&html))));
    let markup = parsed();

    match mode.as_str() {
        "click" => rsx! {
            ClickMode { markup, topic }
        },
        "mc" => rsx! {
            McMode { markup, topic }
        },
        "cloze" => rsx! {
            ClozeMode { markup, topic }
        },
        _ => rsx! {
            ColorizeMode { markup, topic }
        },
    }
}

/// Render the enhanced page, letting the caller put its own control where each
/// token stands. The page's own markup around the tokens is kept as the
/// enhancer wrote it.
pub fn enhanced_text(markup: &Markup, control: impl Fn(&TokenSpan) -> Element) -> Element {
    rsx! {
        div { class: "enhanced",
            for block in markup.blocks().iter() {
                {block_view(markup, block, &control)}
            }
        }
    }
}

fn block_view(markup: &Markup, block: &Block, control: &impl Fn(&TokenSpan) -> Element) -> Element {
    let body = rsx! {
        for piece in block.pieces.iter() {
            match piece {
                Piece::Html(html) => rsx! {
                    span { class: "flow", dangerous_inner_html: "{html}" }
                },
                Piece::Token(at) => match markup.token(*at) {
                    Some(token) => control(token),
                    None => rsx! {},
                },
            }
        }
    };

    match block.kind {
        BlockKind::Heading => rsx! {
            h3 { class: "enhanced-head", {body} }
        },
        BlockKind::Item => rsx! {
            li { class: "enhanced-item", {body} }
        },
        BlockKind::Quote => rsx! {
            blockquote { class: "enhanced-quote", {body} }
        },
        BlockKind::Paragraph => rsx! {
            p { class: "enhanced-line", {body} }
        },
    }
}
