//! The click exercise: nothing is coloured in, and the learner picks out the
//! word forms they believe belong to the topic. A pick is judged at once and
//! stands, so the page fills in with the learner's own reading of it.

use std::collections::HashMap;
use std::rc::Rc;

use dioxus::prelude::*;

use super::enhanced_text;
use super::markup::{Markup, TokenSpan};

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

/// The classes a token carries, before and after it has been judged.
pub fn token_class(verdict: Option<Verdict>) -> &'static str {
    match verdict {
        None => "token token-pick",
        Some(Verdict::Right) => "token token-pick pick-right",
        Some(Verdict::Wrong) => "token token-pick pick-wrong",
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
            disabled: verdict.is_some(),
            onclick: move |event| onchoose.call(event),
            "{text}"
        }
    }
}

#[component]
pub fn ClickMode(markup: Rc<Markup>, topic: String) -> Element {
    let mut picks = use_signal(HashMap::<String, Verdict>::new);

    let wanted = markup.hits(&topic);
    let right = picks
        .read()
        .values()
        .filter(|verdict| **verdict == Verdict::Right)
        .count();
    let wrong = picks.read().len() - right;

    rsx! {
        section { class: "mode mode-click",
            p { class: "score",
                "Rivttes: {right} / {wanted}"
                span { class: "gloss", "{wrong} words picked that the topic does not mark" }
            }
            {
                enhanced_text(
                    &markup,
                    |token| {
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

    #[test]
    fn an_unpicked_word_gives_nothing_away() {
        let unpicked = token_class(None);

        assert!(!unpicked.contains("right"));
        assert!(!unpicked.contains("wrong"));
    }

    #[test]
    fn each_verdict_is_told_apart() {
        let right = token_class(Some(Verdict::Right));
        let wrong = token_class(Some(Verdict::Wrong));

        assert_ne!(right, wrong);
        assert!(right.contains("pick-right"));
        assert!(wrong.contains("pick-wrong"));
        assert!(right.starts_with(token_class(None)));
    }
}
