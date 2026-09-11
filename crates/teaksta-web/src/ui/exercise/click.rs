//! The click exercise: nothing is coloured in, and the learner picks out the
//! word forms they believe belong to the topic. A pick is judged at once and
//! stands, so the page fills in with the learner's own reading of it.
//!
//! Every word of the text is a target, and a word that has not been pressed
//! carries no state at all — not in what it looks like and not in what it is
//! made of. The topic's class stays off the page here, unlike every other
//! mode, because on this page it would be the answer key.

use std::collections::HashMap;
use std::rc::Rc;

use dioxus::prelude::*;

use super::markup::{Markup, TokenSpan};
use super::score::{Dots, RESULT_LABEL, Review, Score, ScoreChip, review_of};
use super::{Setting, enhanced_text};
use crate::ui::choices::mode_gloss;

/// The mode this file is, as the backend and the mode switch name it.
const MODE: &str = "click";

/// How one pick was judged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Right,
    Wrong,
}

/// Judge one pick: a token the topic marked is right, any other token is a
/// word the learner mistook for one.
pub fn judge(token: &TokenSpan, topic: &str) -> Verdict {
    if token.is_hit(topic) {
        Verdict::Right
    } else {
        Verdict::Wrong
    }
}

/// The classes a word carries, before and after it has been judged.
pub fn token_class(verdict: Option<Verdict>) -> &'static str {
    match verdict {
        None => "teaksta-token tk-pick",
        Some(Verdict::Right) => "teaksta-token tk-pick tk-correct",
        Some(Verdict::Wrong) => "teaksta-token tk-pick tk-wrong",
    }
}

/// The mark that rides along with a verdict, so state is never colour alone.
pub fn verdict_mark(verdict: Verdict) -> &'static str {
    match verdict {
        Verdict::Right => "✓",
        Verdict::Wrong => "✕",
    }
}

/// One word the learner may pick, which stops answering once judged.
#[component]
pub fn ClickToken(
    text: String,
    verdict: Option<Verdict>,
    onchoose: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        button {
            r#type: "button",
            class: token_class(verdict),
            lang: "se",
            disabled: verdict.is_some(),
            onclick: move |event| onchoose.call(event),
            "{text}"
            if let Some(verdict) = verdict {
                span { class: "tk-mark", "aria-hidden": "true", "{verdict_mark(verdict)}" }
            }
        }
    }
}

#[component]
pub fn ClickMode(markup: Rc<Markup>, topic: String, prompt: String) -> Element {
    let mut picks = use_signal(HashMap::<String, Verdict>::new);
    let mut showing = use_signal(|| false);

    let wanted = markup.hits(&topic);
    let right = picks
        .read()
        .values()
        .filter(|verdict| **verdict == Verdict::Right)
        .count();
    let wrong = picks.read().len() - right;
    let rows: Vec<Review> = review_of(&markup, &topic, |token| {
        (picks.read().contains_key(&token.id), None)
    });

    rsx! {
        section { class: "mode mode-click",
            div { class: "tk-exhead",
                p { class: "tk-prompt", lang: "se",
                    "{prompt}"
                    span { class: "tk-gloss", "{mode_gloss(MODE)}" }
                }
                ScoreChip {
                    label: "Rivttes".to_string(),
                    count: "{right} / {wanted}",
                    gloss: format!("{wrong} words picked that the topic does not mark"),
                    dots: Dots::Settled { right, total: wanted },
                }
            }
            {
                enhanced_text(
                    &markup,
                    Setting::Reading,
                    |_, token| {
                        let verdict = picks.read().get(&token.id).copied();
                        let id = token.id.clone();
                        let judged = judge(token, &topic);
                        rsx! {
                            ClickToken {
                                text: token.text.clone(),
                                verdict,
                                onchoose: move |_| {
                                    picks.write().insert(id.clone(), judged);
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
                    gloss: format!("{right} of the {wanted} marked words found"),
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
    use crate::ui::exercise::markup::parse;

    fn token(class: &str) -> TokenSpan {
        let block = format!("<p><span class=\"{class}\">Viesut</span></p>");
        parse(std::slice::from_ref(&block)).tokens()[0].clone()
    }

    #[test]
    fn a_topic_word_is_judged_right() {
        let hit = token("teaksta-token teaksta-Substantive");

        assert_eq!(judge(&hit, "Substantive"), Verdict::Right);
    }

    #[test]
    fn a_word_of_another_topic_is_wrong() {
        let hit = token("teaksta-token teaksta-Substantive");

        assert_eq!(judge(&hit, "VerbConjugation"), Verdict::Wrong);
    }

    #[test]
    fn a_word_the_topic_left_alone_is_wrong() {
        let plain = token("teaksta-token");

        assert_eq!(judge(&plain, "Substantive"), Verdict::Wrong);
    }

    /// Every word on the page is made of the same thing until it is pressed:
    /// no state class, and no class of the topic's either, which on this page
    /// would name the answers in the markup.
    #[test]
    fn an_unpicked_word_gives_nothing_away() {
        let unpicked = token_class(None);

        assert_eq!(unpicked, "teaksta-token tk-pick");
        assert!(!unpicked.contains("tk-correct"));
        assert!(!unpicked.contains("tk-wrong"));
    }

    #[test]
    fn each_verdict_is_told_apart() {
        let right = token_class(Some(Verdict::Right));
        let wrong = token_class(Some(Verdict::Wrong));

        assert_ne!(right, wrong);
        assert!(right.contains("tk-correct"));
        assert!(wrong.contains("tk-wrong"));
        assert!(right.starts_with(token_class(None)));
        assert_ne!(verdict_mark(Verdict::Right), verdict_mark(Verdict::Wrong));
    }
}
