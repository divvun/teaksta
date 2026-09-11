//! How an exercise is going, and how it went.
//!
//! Two presentations of one count. While the text is being answered a chip
//! rides beside the instruction: the number, and a dot for every word so the
//! shape of the work is visible at a glance. When the learner asks to see the
//! result, the same count opens out into a figure, a meter and the words one
//! by one, each with the lemma it came from — which is the thing worth taking
//! away from the exercise.
//!
//! The number is never a grade. It counts right answers, there is no pass
//! mark, and nothing turns red for being low.

use dioxus::prelude::*;

use super::markup::{Markup, TokenSpan};

/// Above this many words the dots stop being a glance and turn into counting,
/// so a long text carries the number alone.
const MAX_DOTS: usize = 20;

/// What the learner reads once the exercise is over.
pub const RESULT_LABEL: &str = "Geahča bohtosa";

/// What the dots in a score chip stand for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dots {
    /// One dot per word the topic marked, every one of them lit. Colorize
    /// asks nothing, so a marked word is simply there to be read.
    Marked(usize),
    /// One dot per word there is to answer, filling in as they settle.
    Settled { right: usize, total: usize },
}

impl Dots {
    /// How many dots this would draw.
    pub fn total(self) -> usize {
        match self {
            Dots::Marked(found) => found,
            Dots::Settled { total, .. } => total,
        }
    }

    /// The class of the dot standing at `at`, counting from zero.
    pub fn dot_class(self, at: usize) -> &'static str {
        match self {
            Dots::Marked(_) => "tk-dot tk-dot--topic",
            Dots::Settled { right, .. } if at < right => "tk-dot tk-dot--on",
            Dots::Settled { .. } => "tk-dot",
        }
    }

    /// Whether the dots are worth drawing at all. A text with nothing in it
    /// draws none, and a text with more words than the eye reads at a glance
    /// draws none either.
    pub fn worth_drawing(self) -> bool {
        (1..=MAX_DOTS).contains(&self.total())
    }
}

/// One word the learner met, as the review lists it afterwards.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Review {
    /// Whether the word was answered as the text had it.
    pub right: bool,
    /// The form standing in the text.
    pub word: String,
    /// The base form the enhancer read it from.
    pub lemma: Option<String>,
    /// What the learner answered, when that was not the form wanted.
    pub gave: Option<String>,
}

/// The words the topic marked, in the order they stand in the text, each as
/// the review lists it.
///
/// Only the topic's own words are listed, whatever else an exercise put on
/// the page: a word the learner mistook for one belongs to the count of
/// mistakes beside the instruction, not to the list of what the text was
/// about.
pub fn review_of(
    markup: &Markup,
    topic: &str,
    answered: impl Fn(&TokenSpan) -> (bool, Option<String>),
) -> Vec<Review> {
    markup
        .tokens()
        .iter()
        .filter(|token| token.is_hit(topic))
        .map(|token| {
            let (right, gave) = answered(token);
            Review {
                right,
                word: token.text.clone(),
                lemma: token.lemma.clone(),
                gave: gave.filter(|form| !form.trim().is_empty()),
            }
        })
        .collect()
}

/// The live count, beside the instruction.
#[component]
pub fn ScoreChip(label: String, count: String, gloss: String, dots: Dots) -> Element {
    rsx! {
        div { class: "tk-score",
            span { class: "tk-score-label", lang: "se",
                "{label}: "
                span { class: "tk-score-count", "{count}" }
            }
            if dots.worth_drawing() {
                span { class: "tk-dots", "aria-hidden": "true",
                    for at in 0..dots.total() {
                        span { key: "{at}", class: dots.dot_class(at) }
                    }
                }
            }
            span { class: "tk-gloss", "{gloss}" }
        }
    }
}

/// How it went: the count, the meter, and the words one by one.
#[component]
pub fn Score(right: usize, total: usize, gloss: String, rows: Vec<Review>) -> Element {
    rsx! {
        div { class: "result",
            div { class: "res",
                div {
                    div { class: "res-figure",
                        span { class: "res-num", "{right}" }
                        span { class: "res-of", "/ {total}" }
                    }
                    p { class: "res-label", lang: "se",
                        "Rivttes"
                        span { class: "tk-gloss", "{gloss}" }
                    }
                }
            }

            div { class: "meter", "aria-hidden": "true",
                for (at , row) in rows.iter().enumerate() {
                    span { key: "{at}", class: if row.right { "on" } else { "miss" } }
                }
            }

            p { class: "tk-eyebrow", "Word by word" }
            ul { class: "review",
                for (at , row) in rows.iter().enumerate() {
                    li { key: "{at}", class: "rev", {row_view(row)} }
                }
            }
        }
    }
}

fn row_view(row: &Review) -> Element {
    let mark = if row.right {
        "rev-mark ok"
    } else {
        "rev-mark no"
    };
    let word = if row.right { "rev-word" } else { "rev-word no" };

    rsx! {
        span { class: "{mark}", "aria-hidden": "true", if row.right { "✓" } else { "✕" } }
        span { class: "{word}", lang: "se", "{row.word}" }
        span { class: "rev-meta",
            if let Some(gave) = row.gave.as_ref() {
                "wrote "
                em { lang: "se", "{gave}" }
                if row.lemma.is_some() {
                    " · "
                }
            }
            if let Some(lemma) = row.lemma.as_ref() {
                "← "
                em { lang: "se", "{lemma}" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_marked_word_is_a_topic_dot() {
        let dots = Dots::Marked(3);

        assert_eq!(dots.total(), 3);
        assert_eq!(dots.dot_class(0), "tk-dot tk-dot--topic");
        assert_eq!(dots.dot_class(2), "tk-dot tk-dot--topic");
    }

    #[test]
    fn a_settled_word_lights_its_dot() {
        let dots = Dots::Settled { right: 2, total: 5 };

        assert_eq!(dots.total(), 5);
        assert!(dots.dot_class(0).contains("tk-dot--on"));
        assert!(dots.dot_class(1).contains("tk-dot--on"));
        assert_eq!(dots.dot_class(2), "tk-dot");
        assert_eq!(dots.dot_class(4), "tk-dot");
    }

    /// The dots are a glance at the shape of the work, so a page with more
    /// words than the eye takes in at once carries the count alone.
    #[test]
    fn a_long_text_draws_no_dots() {
        assert!(!Dots::Marked(0).worth_drawing());
        assert!(Dots::Marked(1).worth_drawing());
        assert!(Dots::Marked(MAX_DOTS).worth_drawing());
        assert!(!Dots::Marked(MAX_DOTS + 1).worth_drawing());
    }
}
