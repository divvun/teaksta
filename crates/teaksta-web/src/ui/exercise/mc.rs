//! The multiple-choice exercise: every word form the topic marked becomes a
//! list of forms of the same lemma, one of which stood in the page.

use std::collections::HashMap;
use std::rc::Rc;

use dioxus::prelude::*;

use super::enhanced_text;
use super::markup::{Markup, TokenSpan};

/// How many forms one slot offers, which is what the legacy engine offered.
pub const MAX_CHOICES: usize = 5;

/// How far one slot has got.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Slot {
    /// The form last chosen, which stays on show while a learner tries again.
    pub chosen: String,
    /// Set once a chosen form was accepted, after which the slot is fixed.
    pub settled: bool,
    /// How many forms were chosen before the slot settled.
    pub tries: u32,
}

/// How a word form is capitalised where it stands in the page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Plain,
    Upper,
    Initial,
}

/// Read the capitalisation of a word form.
pub fn shape_of(word: &str) -> Shape {
    if word.is_empty() {
        return Shape::Plain;
    }
    if word == word.to_uppercase() {
        return Shape::Upper;
    }

    let mut letters = word.chars();
    let first: String = letters
        .next()
        .map(|letter| letter.to_uppercase().collect())
        .unwrap_or_default();
    if word == format!("{first}{}", letters.as_str()) {
        Shape::Initial
    } else {
        Shape::Plain
    }
}

/// Recase a form to read like the one standing in the page, so that no option
/// gives itself away by its capitals.
pub fn cased(word: &str, shape: Shape) -> String {
    match shape {
        Shape::Plain => word.to_string(),
        Shape::Upper => word.to_uppercase(),
        Shape::Initial => {
            let mut letters = word.chars();
            match letters.next() {
                Some(first) => format!(
                    "{}{}",
                    first.to_uppercase().collect::<String>(),
                    letters.as_str()
                ),
                None => String::new(),
            }
        }
    }
}

/// The forms one slot offers.
///
/// The enhancer generates the paradigm and writes the form that stood in the
/// page last, so the attribute's order cannot be shown as it arrived — it
/// would answer the question. The forms are ordered by themselves instead,
/// which tells a learner nothing and reads the same on every render.
pub fn choices(token: &TokenSpan) -> Vec<String> {
    let shape = shape_of(&token.text);
    let surface = token.text.to_lowercase();
    let mut forms = vec![surface.clone()];

    for distractor in &token.distractors {
        let form = distractor.to_lowercase();
        if form.is_empty() || forms.contains(&form) || forms.len() >= MAX_CHOICES {
            continue;
        }
        forms.push(form);
    }

    forms.sort();
    forms.iter().map(|form| cased(form, shape)).collect()
}

/// One slot, which turns into the form it was answered with.
#[component]
pub fn McToken(choices: Vec<String>, slot: Slot, onchoose: EventHandler<String>) -> Element {
    if slot.settled {
        return rsx! {
            span { class: "token pick-right", "{slot.chosen}" }
        };
    }

    let class = if slot.tries > 0 {
        "mc-pick mc-wrong"
    } else {
        "mc-pick"
    };

    rsx! {
        select {
            class: "{class}",
            onchange: move |event| onchoose.call(event.value()),
            option { value: "", selected: slot.chosen.is_empty(), " " }
            for choice in choices.iter() {
                option { value: "{choice}", selected: choice == &slot.chosen, "{choice}" }
            }
        }
    }
}

#[component]
pub fn McMode(markup: Rc<Markup>, topic: String) -> Element {
    let mut slots = use_signal(HashMap::<String, Slot>::new);

    let wanted = markup.hits(&topic);
    let settled = slots.read().values().filter(|slot| slot.settled).count();
    let tries: u32 = slots.read().values().map(|slot| slot.tries).sum();

    rsx! {
        section { class: "mode mode-mc",
            p { class: "score",
                "Rivttes: {settled} / {wanted}"
                span { class: "gloss", "{tries} forms chosen so far" }
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
                        rsx! {
                            McToken {
                                choices: choices(token),
                                slot,
                                onchoose: move |form: String| {
                                    let mut all = slots.write();
                                    let slot = all.entry(id.clone()).or_default();
                                    slot.chosen = form.clone();
                                    if form.trim().is_empty() {
                                        return;
                                    }
                                    slot.tries += 1;
                                    slot.settled = judged.accepts(&form);
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

    fn token(attributes: &str) -> TokenSpan {
        let page = format!("<p><span class=\"teaksta-token\" {attributes}>Viesut</span></p>");
        parse(&page).tokens()[0].clone()
    }

    #[test]
    fn capitalisation_is_read_off_the_form() {
        assert_eq!(shape_of("viesu"), Shape::Plain);
        assert_eq!(shape_of("Viesut"), Shape::Initial);
        assert_eq!(shape_of("VIESUT"), Shape::Upper);
        assert_eq!(shape_of("Á"), Shape::Upper);
        assert_eq!(shape_of(""), Shape::Plain);
    }

    #[test]
    fn a_form_is_recased_to_match() {
        assert_eq!(cased("viesu", Shape::Plain), "viesu");
        assert_eq!(cased("viesu", Shape::Initial), "Viesu");
        assert_eq!(cased("viesu", Shape::Upper), "VIESU");
        assert_eq!(cased("álbmot", Shape::Initial), "Álbmot");
        assert_eq!(cased("", Shape::Initial), "");
    }

    #[test]
    fn every_option_reads_like_the_page() {
        let offered = choices(&token("distractors=\"viesuid viesuide viesut\""));

        assert!(offered.iter().all(|form| form.starts_with('V')));
        assert!(offered.contains(&"Viesut".to_string()));
    }

    #[test]
    fn the_page_form_is_always_offered() {
        let offered = choices(&token("distractors=\"viesuid viesuide\""));

        assert_eq!(offered, ["Viesuid", "Viesuide", "Viesut"]);
    }

    #[test]
    fn a_form_is_never_offered_twice() {
        let offered = choices(&token("distractors=\"viesuid viesuid viesut Viesut\""));

        assert_eq!(offered, ["Viesuid", "Viesut"]);
    }

    #[test]
    fn no_slot_offers_more_than_five() {
        let offered = choices(&token("distractors=\"a b c d e f g h\""));

        assert_eq!(offered.len(), MAX_CHOICES);
        assert!(offered.contains(&"Viesut".to_string()));
    }

    #[test]
    fn a_token_without_distractors_still_reads() {
        assert_eq!(choices(&token("lemma=\"viessu\"")), ["Viesut"]);
    }

    #[test]
    fn a_fresh_slot_has_been_tried_never() {
        let slot = Slot::default();

        assert!(slot.chosen.is_empty());
        assert!(!slot.settled);
        assert_eq!(slot.tries, 0);
    }
}
