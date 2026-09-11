//! The exercise view: the chosen parameters, the backend request they map to,
//! and the analysed text itself, woven into whichever exercise was asked for.
//!
//! The screen is one column over one stage. The topic names the screen, the
//! provenance of the text sits under it in a quiet line, and everything below
//! that belongs to the reading: the instruction, the score, and the sentences
//! the learner came for.

pub mod click;
pub mod cloze;
pub mod colorize;
pub mod markup;
pub mod mc;
pub mod score;

use std::rc::Rc;

use dioxus::prelude::*;

use crate::api::{Backend, BlockRequest, fetch_blocks};
use crate::route::{ExerciseQuery, Route};
use crate::ui::SharedRegistry;
use crate::ui::choices::topic_subtitle;

use click::ClickMode;
use cloze::ClozeMode;
use colorize::ColorizeMode;
use markup::{Block, BlockKind, Markup, Piece, TokenSpan};
use mc::McMode;

#[component]
pub fn Exercise(params: ExerciseQuery) -> Element {
    let backend = use_context::<Backend>();
    let request = BlockRequest::fetched(params.url.clone(), params.topic.clone(), &params.mode);
    let target = backend.blocks_url();

    let text = use_resource(use_reactive!(|request| {
        let backend = backend.clone();
        async move { fetch_blocks(&backend, &request).await }
    }));

    let registry = use_context::<SharedRegistry>();
    let named = registry.value();
    let (topic_label, mode_label, others) = match &*named.read_unchecked() {
        Some(Ok(offered)) => (
            offered.activity_label(&params.topic).to_string(),
            offered.mode_label(&params.mode).to_string(),
            offered
                .modes
                .iter()
                .filter(|mode| mode.name != params.mode)
                .map(|mode| {
                    (
                        mode.name.clone(),
                        offered.mode_label(&mode.name).to_string(),
                    )
                })
                .collect::<Vec<(String, String)>>(),
        ),
        _ => (
            params.topic.clone(),
            params.mode.clone(),
            Vec::<(String, String)>::new(),
        ),
    };
    let value = text.value();

    rsx! {
        div { class: "ctx",
            div {
                h1 { lang: "se",
                    "{topic_label}"
                    span { class: "ctx-gloss", "{topic_subtitle(&params.topic)}" }
                }
            }
            div { class: "ctx-chips",
                span { class: "tk-chip",
                    span { class: "tk-sme", lang: "se", "{mode_label}" }
                    span { class: "tk-gloss", "{params.mode}" }
                }
            }
        }

        p { class: "src",
            span { class: "src-title", "{params.topic}" }
            span { class: "src-sep", "·" }
            span { class: "src-url", "{params.url}" }
            span { class: "src-sep", "·" }
            span { class: "src-url", "{target}" }
        }

        section { class: "stage",
            match &*value.read_unchecked() {
                None => rsx! {
                    p { class: "state state-pending", lang: "se",
                        "Vuorddát…"
                        span { class: "tk-gloss", "Asking the backend for the analysed text" }
                    }
                },
                Some(Ok(text)) => rsx! {
                    EnhancedText {
                        blocks: text.iter().map(|block| block.html.clone()).collect::<Vec<String>>(),
                        topic: params.topic.clone(),
                        mode: params.mode.clone(),
                        prompt: mode_label.clone(),
                    }
                },
                Some(Err(error)) => rsx! {
                    p { class: "state state-error", "{error}" }
                },
            }

            div { class: "stage-foot",
                span { class: "tk-gloss",
                    "The same text is still loaded — switch mode to read it again."
                }
                span { class: "stage-swaps",
                    for (name , label) in others {
                        Link {
                            key: "{name}",
                            to: Route::Exercise {
                                params: ExerciseQuery::new(
                                    params.topic.clone(),
                                    name.clone(),
                                    params.url.clone(),
                                ),
                            },
                            class: "tk-btn tk-btn--quiet",
                            title: "{label}",
                            "{name}"
                        }
                    }
                }
            }
        }
    }
}

/// The analysed text, read once and handed to the exercise that was asked
/// for. An unknown mode reads as colorize, which every topic offers.
#[component]
pub fn EnhancedText(blocks: Vec<String>, topic: String, mode: String, prompt: String) -> Element {
    let parsed = use_memo(use_reactive!(|blocks| Rc::new(markup::parse(&blocks))));
    let markup = parsed();

    match mode.as_str() {
        "click" => rsx! {
            ClickMode { markup, topic, prompt }
        },
        "mc" => rsx! {
            McMode { markup, topic, prompt }
        },
        "cloze" => rsx! {
            ClozeMode { markup, topic, prompt }
        },
        _ => rsx! {
            ColorizeMode { markup, topic, prompt }
        },
    }
}

/// How an exercise sets the text it weaves its controls into.
///
/// Inline controls add to the line box, so the modes that put one in every
/// slot ride a looser leading and the rhythm of the prose holds. A menu goes
/// further: it is an overlay inside the reading surface, so the surface keeps
/// room for it rather than letting it spill past its own edge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Setting {
    /// Nothing is added to the line box: the text keeps its reading leading.
    #[default]
    Reading,
    /// A control stands in every slot.
    Inline,
    /// A control stands in every slot and one of them can open over the text.
    Menus,
}

impl Setting {
    fn prose(self) -> &'static str {
        match self {
            Setting::Reading => "tk-prose",
            Setting::Inline | Setting::Menus => "tk-prose tk-prose--inline",
        }
    }

    fn reading(self) -> &'static str {
        match self {
            Setting::Reading | Setting::Inline => "enhanced tk-reading",
            Setting::Menus => "enhanced tk-reading tk-reading--menus",
        }
    }
}

/// Render the analysed text, letting the caller put its own control where each
/// token stands. The page's own markup around the tokens is kept as the
/// enhancer wrote it.
///
/// The control is handed the token's number as well as the token, because a
/// control that needs an id of its own — a menu to point at, a slot to label —
/// has to name itself something stable that gives nothing away, and the
/// enhancer's own ids carry the lemma in them.
pub fn enhanced_text(
    markup: &Markup,
    setting: Setting,
    control: impl Fn(usize, &TokenSpan) -> Element,
) -> Element {
    rsx! {
        div { class: setting.reading(),
            for block in markup.blocks().iter() {
                {block_view(markup, block, setting, &control)}
            }
        }
    }
}

fn block_view(
    markup: &Markup,
    block: &Block,
    setting: Setting,
    control: &impl Fn(usize, &TokenSpan) -> Element,
) -> Element {
    let prose = setting.prose();
    let body = rsx! {
        for piece in block.pieces.iter() {
            match piece {
                Piece::Html(html) => rsx! {
                    span { class: "flow", dangerous_inner_html: "{html}" }
                },
                Piece::Token(at) => match markup.token(*at) {
                    Some(token) => control(*at, token),
                    None => rsx! {},
                },
            }
        }
    };

    match block.kind {
        BlockKind::Heading => rsx! {
            h3 { class: "enhanced-head {prose}", lang: "se", {body} }
        },
        BlockKind::Item => rsx! {
            li { class: "enhanced-item {prose}", lang: "se", {body} }
        },
        BlockKind::Quote => rsx! {
            blockquote { class: "enhanced-quote {prose}", lang: "se", {body} }
        },
        BlockKind::Paragraph => rsx! {
            p { class: "enhanced-line {prose}", lang: "se", {body} }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Only the modes that put a control in the line loosen the leading, and
    /// only the one whose control opens a menu keeps room under the text.
    #[test]
    fn a_setting_says_how_the_text_is_set() {
        assert_eq!(Setting::Reading.prose(), "tk-prose");
        assert!(Setting::Inline.prose().contains("tk-prose--inline"));
        assert!(Setting::Menus.prose().contains("tk-prose--inline"));

        assert!(!Setting::Inline.reading().contains("--menus"));
        assert!(Setting::Menus.reading().contains("tk-reading--menus"));
    }
}
