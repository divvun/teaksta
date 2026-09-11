//! The colorize exercise: the topic's word forms stand out of the page so a
//! learner reads them in context without being asked anything.
//!
//! Passive noticing. Nothing is asked, the topic simply shows itself, and the
//! leading stays at the reading default because nothing here changes the line
//! box.

use std::rc::Rc;

use dioxus::prelude::*;

use super::markup::{Markup, TokenSpan};
use super::score::{Dots, ScoreChip};
use super::{Setting, enhanced_text};
use crate::ui::choices::mode_gloss;

/// The mode this file is, as the backend and the mode switch name it.
const MODE: &str = "colorize";

/// The state class a word the topic matched wears. Written on the state alone
/// rather than on a topic's own class, so every topic highlights the same way.
pub const HIT_CLASS: &str = "tk-hit";

/// What one word carries: whatever the enhancer wrote on it, and the
/// highlight when it belongs to the topic. A word outside the topic is left
/// with the enhancer's classes alone, which style nothing.
pub fn token_class(token: &TokenSpan, topic: &str) -> String {
    let mut class = token.classes.join(" ");
    if token.is_hit(topic) {
        if !class.is_empty() {
            class.push(' ');
        }
        class.push_str(HIT_CLASS);
    }

    class
}

#[component]
pub fn ColorizeMode(markup: Rc<Markup>, topic: String, prompt: String) -> Element {
    let found = markup.hits(&topic);

    rsx! {
        section { class: "mode mode-colorize",
            div { class: "tk-exhead",
                p { class: "tk-prompt", lang: "se",
                    "{prompt}"
                    span { class: "tk-gloss", "{mode_gloss(MODE)}" }
                }
                ScoreChip {
                    label: "Ivdnejuvvon sánit".to_string(),
                    count: found.to_string(),
                    gloss: format!("{found} highlighted word forms"),
                    dots: Dots::Marked(found),
                }
            }
            {
                enhanced_text(
                    &markup,
                    Setting::Reading,
                    |_, token| {
                        let class = token_class(token, &topic);
                        rsx! {
                            span { class: "{class}", "{token.text}" }
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
    fn only_a_hit_is_given_a_colour() {
        let hit = token("teaksta-token teaksta-Substantive");
        let plain = token("teaksta-token");

        assert_eq!(
            token_class(&hit, "Substantive"),
            "teaksta-token teaksta-Substantive tk-hit"
        );
        assert_eq!(token_class(&plain, "Substantive"), "teaksta-token");
        assert!(!token_class(&hit, "VerbConjugation").contains(HIT_CLASS));
    }

    /// The enhancer's own classes reach the page untouched, so a per-tag class
    /// a topic writes alongside its own is still there to be styled.
    #[test]
    fn the_enhancers_classes_are_kept() {
        let tagged = token("teaksta-token teaksta-CC teaksta-Conjunctions");

        assert!(token_class(&tagged, "Conjunctions").starts_with(&tagged.classes.join(" ")));
    }
}
