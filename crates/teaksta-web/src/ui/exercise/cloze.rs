//! The cloze exercise: every word form the topic marked is taken out of the
//! page and the learner writes it back from the lemma.
//!
//! A topic can generate several forms a learner could not tell apart from the
//! text that is left, so a slot accepts any of them. That is the rule the
//! legacy engine was fixed to follow and it is what makes the exercise fair.

use std::collections::HashMap;
use std::rc::Rc;

use dioxus::prelude::*;

use super::enhanced_text;
use super::markup::Markup;

/// How far one slot has got.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum State {
    #[default]
    Open,
    Wrong,
    Right,
    /// The learner asked for the answer, so the slot is filled but not won.
    Shown,
}

/// What a learner has written into one slot.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Slot {
    pub written: String,
    pub state: State,
    pub tries: u32,
}

/// The classes the box carries while it is still being answered.
pub fn box_class(state: State) -> &'static str {
    match state {
        State::Wrong => "cloze-box cloze-miss",
        _ => "cloze-box",
    }
}

/// One slot: a box to write in, the lemma to write it from, and a hint that
/// gives up on the slot and shows every form it would have taken.
#[component]
pub fn ClozeToken(
    lemma: Option<String>,
    slot: Slot,
    onwrite: EventHandler<String>,
    onhint: EventHandler<()>,
) -> Element {
    match slot.state {
        State::Right => rsx! {
            span { class: "token pick-right", "{slot.written}" }
        },
        State::Shown => rsx! {
            span { class: "token cloze-shown", "{slot.written}" }
        },
        _ => rsx! {
            span { class: "cloze",
                input {
                    r#type: "text",
                    class: box_class(slot.state),
                    value: "{slot.written}",
                    onchange: move |event| onwrite.call(event.value()),
                }
                button {
                    r#type: "button",
                    class: "cloze-hint",
                    title: "Čájet vástádusa",
                    onclick: move |_| onhint.call(()),
                    "?"
                }
                if let Some(lemma) = lemma.as_ref() {
                    span { class: "cloze-lemma", " ({lemma})" }
                }
            }
        },
    }
}

#[component]
pub fn ClozeMode(markup: Rc<Markup>, topic: String) -> Element {
    let mut slots = use_signal(HashMap::<String, Slot>::new);

    let wanted = markup.hits(&topic);
    let right = slots
        .read()
        .values()
        .filter(|slot| slot.state == State::Right)
        .count();
    let tries: u32 = slots.read().values().map(|slot| slot.tries).sum();

    rsx! {
        section { class: "mode mode-cloze",
            p { class: "score",
                "Rivttes: {right} / {wanted}"
                span { class: "gloss", "{tries} forms written so far" }
            }
            {
                enhanced_text(
                    &markup,
                    |token| {
                        if !token.is_hit(&topic) {
                            return rsx! {
                                span { class: "token", "{token.text}" }
                            };
                        }
                        let slot = slots.read().get(&token.id).cloned().unwrap_or_default();
                        let id = token.id.clone();
                        let judged = token.clone();
                        let shown = token.hint();
                        let hinted = token.id.clone();
                        rsx! {
                            ClozeToken {
                                lemma: token.lemma.clone(),
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
                                    slot
                                        .state = if judged.accepts(&written) {
                                        State::Right
                                    } else {
                                        State::Wrong
                                    };
                                },
                                onhint: move |()| {
                                    let mut all = slots.write();
                                    let slot = all.entry(hinted.clone()).or_default();
                                    slot.written = shown.clone();
                                    slot.state = State::Shown;
                                },
                            }
                        }
                    },
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_slot_is_open_and_empty() {
        let slot = Slot::default();

        assert_eq!(slot.state, State::Open);
        assert!(slot.written.is_empty());
        assert_eq!(slot.tries, 0);
    }

    #[test]
    fn only_a_missed_box_is_marked() {
        assert_eq!(box_class(State::Open), "cloze-box");
        assert_eq!(box_class(State::Wrong), "cloze-box cloze-miss");
        assert!(box_class(State::Wrong).starts_with(box_class(State::Open)));
    }
}
