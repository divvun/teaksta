//! The cloze exercise: every word form the topic marked is taken out of the
//! page and the learner writes it back.
//!
//! A topic can generate several forms a learner could not tell apart from the
//! text that is left, so a slot accepts any of them. That is the rule the
//! legacy engine was fixed to follow and it is what makes the exercise fair.
//!
//! What the learner is given to write from is the lemma, and only when they
//! ask for it. Yellow means help everywhere in the app, and what this help
//! gives back is the base form — never one of the forms that would have been
//! accepted, because a hint that answered the question would end the exercise
//! instead of carrying it. A slot stays editable through all of it, so a form
//! written wrong can be corrected where it stands.

use std::collections::HashMap;
use std::rc::Rc;

use dioxus::prelude::*;

use super::markup::{Markup, accepts};
use super::score::{Dots, RESULT_LABEL, Review, Score, ScoreChip, review_of};
use super::{Setting, enhanced_text};
use crate::ui::choices::mode_gloss;

/// The mode this file is, as the backend and the mode switch name it.
const MODE: &str = "cloze";

/// The narrowest a slot is ever drawn, in characters of the reading serif. A
/// two-letter form still needs room to be written in and corrected.
const MIN_WIDTH: usize = 4;

/// How much room a form is given beyond its own length, so that writing it
/// never runs up against the edge of the box.
const WIDTH_SLACK: usize = 2;

/// How far one slot has got.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum State {
    #[default]
    Open,
    Wrong,
    Right,
}

/// What a learner has written into one slot.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Slot {
    pub written: String,
    pub state: State,
    pub tries: u32,
    /// Whether the learner has asked for the base form.
    pub hinted: bool,
}

/// The classes the box carries, which say how the form in it was judged.
pub fn box_class(state: State) -> &'static str {
    match state {
        State::Open => "tk-cloze-input",
        State::Wrong => "tk-cloze-input tk-wrong",
        State::Right => "tk-cloze-input tk-correct",
    }
}

/// How wide to draw the slot for one word, in characters of the reading
/// serif. A gap as wide as the form it expects is a hint about length, which
/// is fair; a box of a fixed few pixels the word cannot fit in is not.
pub fn slot_width(form: &str) -> usize {
    (form.chars().count() + WIDTH_SLACK).max(MIN_WIDTH)
}

/// One slot: a box to write in, how the last form written was judged, and a
/// hint that gives up the base form the word came from.
#[component]
pub fn ClozeToken(
    lemma: Option<String>,
    width: usize,
    slot: Slot,
    onwrite: EventHandler<String>,
    onhint: EventHandler<()>,
) -> Element {
    let settled = slot.state == State::Right;
    let mark = match slot.state {
        State::Right => Some(("tk-mark tk-mark--ok", "✓")),
        State::Wrong => Some(("tk-mark tk-mark--no", "✕")),
        State::Open => None,
    };

    rsx! {
        span { class: "tk-cloze",
            input {
                r#type: "text",
                class: box_class(slot.state),
                lang: "se",
                "aria-label": "Write the missing form",
                style: "width: {width}ch",
                readonly: settled,
                value: "{slot.written}",
                onchange: move |event| onwrite.call(event.value()),
            }
            if let Some((class, glyph)) = mark {
                span { class, "aria-hidden": "true", "{glyph}" }
            }
            if let Some(lemma) = lemma.as_ref().filter(|_| !settled) {
                if slot.hinted {
                    span { class: "tk-hint",
                        span { class: "tk-hint-mark", "aria-hidden": "true", "?" }
                        em { lang: "se", "{lemma}" }
                    }
                } else {
                    button {
                        r#type: "button",
                        class: "tk-hint-btn",
                        "aria-label": "Show the base form",
                        onclick: move |_| onhint.call(()),
                        "?"
                    }
                }
            }
        }
    }
}

#[component]
pub fn ClozeMode(markup: Rc<Markup>, topic: String, prompt: String) -> Element {
    let mut slots = use_signal(HashMap::<String, Slot>::new);
    let mut showing = use_signal(|| false);

    let wanted = markup.hits(&topic);
    let right = slots
        .read()
        .values()
        .filter(|slot| slot.state == State::Right)
        .count();
    let tries: u32 = slots.read().values().map(|slot| slot.tries).sum();
    let rows: Vec<Review> = review_of(&markup, &topic, |token| {
        let slot = slots.read().get(&token.id).cloned().unwrap_or_default();
        let done = slot.state == State::Right;
        (done, (!done).then_some(slot.written))
    });

    rsx! {
        section { class: "mode mode-cloze",
            div { class: "tk-exhead",
                p { class: "tk-prompt", lang: "se",
                    "{prompt}"
                    span { class: "tk-gloss", "{mode_gloss(MODE)}" }
                }
                ScoreChip {
                    label: "Rivttes".to_string(),
                    count: "{right} / {wanted}",
                    gloss: format!("{tries} forms written so far"),
                    dots: Dots::Settled { right, total: wanted },
                }
            }
            {
                enhanced_text(
                    &markup,
                    Setting::Inline,
                    |_, token| {
                        if !token.is_hit(&topic) {
                            return rsx! {
                                span { class: "teaksta-token", lang: "se", "{token.text}" }
                            };
                        }
                        let slot = slots.read().get(&token.id).cloned().unwrap_or_default();
                        let id = token.id.clone();
                        let accepted = token.accepted_forms();
                        let hinted = token.id.clone();
                        rsx! {
                            ClozeToken {
                                lemma: token.hint().map(str::to_string),
                                width: slot_width(&token.text),
                                slot,
                                onwrite: move |written: String| {
                                    let mut all = slots.write();
                                    let slot = all.entry(id.clone()).or_default();
                                    slot.written = written.clone();
                                    if written.trim().is_empty() {
                                        slot.state = State::Open;
                                        return;
                                    }
                                    slot.tries += 1;
                                    slot.state = if accepts(&accepted, &written) {
                                        State::Right
                                    } else {
                                        State::Wrong
                                    };
                                },
                                onhint: move |()| {
                                    slots.write().entry(hinted.clone()).or_default().hinted = true;
                                },
                            }
                        }
                    },
                )
            }
            if showing() {
                Score {
                    right,
                    total: wanted,
                    gloss: format!("{right} of the {wanted} forms written right"),
                    rows,
                }
            } else {
                div { class: "actions",
                    button {
                        r#type: "button",
                        class: "tk-btn tk-btn--primary",
                        lang: "se",
                        onclick: move |_| showing.set(true),
                        "{RESULT_LABEL}"
                        span { class: "tk-gloss", "See how it went" }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::exercise::markup::{TokenSpan, parse};

    fn token(attributes: &str) -> TokenSpan {
        let block = format!("<p><span class=\"teaksta-token\" {attributes}>viesu</span></p>");
        parse(std::slice::from_ref(&block)).tokens()[0].clone()
    }

    #[test]
    fn a_fresh_slot_is_open_and_empty() {
        let slot = Slot::default();

        assert_eq!(slot.state, State::Open);
        assert!(slot.written.is_empty());
        assert_eq!(slot.tries, 0);
        assert!(!slot.hinted);
    }

    #[test]
    fn a_judged_box_says_how_it_went() {
        assert_eq!(box_class(State::Open), "tk-cloze-input");
        assert!(box_class(State::Wrong).contains("tk-wrong"));
        assert!(box_class(State::Right).contains("tk-correct"));
        assert!(box_class(State::Wrong).starts_with(box_class(State::Open)));
    }

    /// The gap is as wide as the form the text had, plus room to write in.
    #[test]
    fn a_slot_is_as_wide_as_its_form() {
        assert_eq!(slot_width("viesu"), 7);
        assert_eq!(slot_width("skuvllas"), 10);
        assert_eq!(slot_width("ja"), MIN_WIDTH);
        assert_eq!(slot_width(""), MIN_WIDTH);
        // Counted in characters, not bytes: a diacritic is one column wide.
        assert_eq!(slot_width("Bárdni"), slot_width("Bardni"));
    }

    /// What the hint gives up is the base form the word was read from. Every
    /// form that would have been accepted stays behind it.
    #[test]
    fn the_hint_offers_the_lemma_alone() {
        let token = token("lemma=\"viessu\" possibleforms=\"viesu viesuid\"");

        assert_eq!(token.hint(), Some("viessu"));
        assert!(!token.accepts("viessu"));
        assert!(token.accepts("viesu"));
    }
}
