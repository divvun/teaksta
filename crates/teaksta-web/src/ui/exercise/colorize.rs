//! The colorize exercise: the topic's word forms stand out of the page so a
//! learner reads them in context without being asked anything.

use std::rc::Rc;

use dioxus::prelude::*;

use super::enhanced_text;
use super::markup::Markup;

/// The class carrying the activity's `colorizeStyle`: coloured and bold.
pub const HIT_CLASS: &str = "token token-hit";

/// The class a token outside the topic carries, which is no style at all.
pub const PLAIN_CLASS: &str = "token";

#[component]
pub fn ColorizeMode(markup: Rc<Markup>, topic: String) -> Element {
    let found = markup.hits(&topic);

    rsx! {
        section { class: "mode mode-colorize",
            p { class: "score",
                "Ivdnejuvvon sánit: {found}"
                span { class: "gloss", "{found} highlighted word forms on the page" }
            }
            {
                enhanced_text(
                    &markup,
                    |token| {
                        let class = if token.is_hit(&topic) { HIT_CLASS } else { PLAIN_CLASS };
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

    #[test]
    fn only_a_hit_is_given_a_colour() {
        assert!(HIT_CLASS.starts_with(PLAIN_CLASS));
        assert_ne!(HIT_CLASS, PLAIN_CLASS);
        assert!(HIT_CLASS.contains("token-hit"));
    }
}
