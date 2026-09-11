//! The multiple-choice exercise: every word form the topic marked becomes a
//! list of forms of the same lemma, one of which stood in the page.
//!
//! The chooser is built here rather than taken from the browser. A native
//! `<select>` cannot be styled into the reading — it brings its own type, its
//! own metrics and its own menu — and inside the paragraph it belongs to, the
//! obvious markup for a menu is not available either: a `<ul>` is flow
//! content and would close the `<p>` out from under it. So the menu is a run
//! of spans carrying listbox roles, and everything a native control would
//! have given for free — the open and shut, the cursor, the keyboard, the
//! announcement — is written out below.
//!
//! The state machine is kept apart from the rendering. `key_move` says what a
//! press asks for and `step` says what the chooser does about it; neither
//! touches the DOM, so both can be read straight through and tested as the
//! plain functions they are.

use std::collections::HashMap;
use std::rc::Rc;

use dioxus::prelude::*;

use super::markup::{Markup, TokenSpan, accepts};
use super::score::{Dots, RESULT_LABEL, Review, Score, ScoreChip, review_of};
use super::{Setting, enhanced_text};
use crate::ui::choices::mode_gloss;

/// The mode this file is, as the backend and the mode switch name it.
const MODE: &str = "mc";

/// How many forms one slot offers, which is what the legacy engine offered.
pub const MAX_CHOICES: usize = 5;

/// The character a chooser holds while nothing has been taken, so that an
/// empty slot still has the height of a full one.
const EMPTY_VALUE: &str = "\u{a0}";

/// How far one slot has got.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Slot {
    /// The form last taken, which stays on show while a learner tries again.
    pub chosen: String,
    /// Set once a taken form was accepted, after which the slot is fixed.
    pub settled: bool,
    /// How many forms were taken before the slot settled.
    pub tries: u32,
}

/// Where the keyboard is in one chooser's menu: nowhere while the menu is
/// shut, and on one of the forms while it is open.
pub type Cursor = Option<usize>;

/// What a press asks a chooser to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    /// Open the menu at its first form.
    Open,
    /// Open the menu at its last form, which is where a learner arriving from
    /// below expects to land.
    OpenLast,
    /// Shut the menu, taking nothing.
    Shut,
    /// Toggle: shut an open menu, open a shut one.
    Press,
    Prev,
    Next,
    First,
    Last,
    /// Take the form under the cursor.
    Take,
    /// Take the form at this place in the menu, which is what pressing one
    /// asks for — a pointer names the form it landed on rather than walking
    /// the cursor over to it.
    TakeAt(usize),
    /// A key this chooser does not answer.
    Ignore,
}

/// What a move leaves behind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// The menu is shut and nothing was taken.
    Shut,
    /// The menu is open with this form under the cursor.
    Open(usize),
    /// This form was taken, and the menu is shut behind it.
    Take(usize),
    /// Nothing changes.
    Stay,
}

/// What a key asks of a chooser. The map is the one a native select has, so a
/// learner who knows one knows this: Enter, Space or Down open it, the arrows
/// walk it, Enter takes the form under the cursor, and Escape leaves without
/// taking anything.
pub fn key_move(key: &str, open: bool) -> Move {
    match key {
        "Escape" | "Tab" => Move::Shut,
        "Enter" | " " | "Spacebar" if open => Move::Take,
        "Enter" | " " | "Spacebar" => Move::Open,
        "ArrowDown" if open => Move::Next,
        "ArrowDown" => Move::Open,
        "ArrowUp" if open => Move::Prev,
        "ArrowUp" => Move::OpenLast,
        "Home" | "PageUp" => Move::First,
        "End" | "PageDown" => Move::Last,
        _ => Move::Ignore,
    }
}

/// Whether the chooser answers a key itself, so the browser should not act on
/// it as well. Tab is the exception: it shuts the menu and then has to be let
/// through, or focus would be trapped in a word.
pub fn swallows(key: &str, open: bool) -> bool {
    key != "Tab" && key_move(key, open) != Move::Ignore
}

/// What one move does to a chooser standing at `cursor` over `count` forms.
///
/// The cursor wraps at both ends, which is what a learner walking a list of
/// five forms expects and what saves a keypress at either end.
pub fn step(cursor: Cursor, asked: Move, count: usize) -> Step {
    // A slot the enhancer generated no paradigm for offers nothing to open on.
    if count == 0 {
        return match asked {
            Move::Ignore | Move::TakeAt(_) => Step::Stay,
            _ => Step::Shut,
        };
    }

    let last = count - 1;
    match (cursor, asked) {
        (_, Move::Ignore) => Step::Stay,
        (_, Move::TakeAt(at)) if at <= last => Step::Take(at),
        (_, Move::TakeAt(_)) => Step::Stay,
        (_, Move::Shut) => Step::Shut,
        (Some(_), Move::Press) => Step::Shut,
        (None, Move::Press | Move::Open | Move::Next | Move::First) => Step::Open(0),
        (None, Move::OpenLast | Move::Prev | Move::Last) => Step::Open(last),
        // Enter on a shut chooser opens it rather than taking a form nothing
        // is standing on.
        (None, Move::Take) => Step::Open(0),
        (Some(_), Move::Open) => Step::Open(0),
        (Some(_), Move::OpenLast | Move::Last) => Step::Open(last),
        (Some(_), Move::First) => Step::Open(0),
        (Some(at), Move::Take) => Step::Take(at),
        (Some(at), Move::Next) => Step::Open(if at >= last { 0 } else { at + 1 }),
        (Some(at), Move::Prev) => Step::Open(if at == 0 { last } else { at - 1 }),
    }
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
///
/// The chooser keeps focus the whole time the menu is open — there is nothing
/// inside the menu to move focus to, and moving it there would lose the
/// paragraph. The option the keyboard is on is named by `aria-activedescendant`
/// instead, which is how a listbox says where it is without moving focus.
#[component]
pub fn McToken(
    id: String,
    choices: Vec<String>,
    slot: Slot,
    cursor: Cursor,
    onmove: EventHandler<Move>,
) -> Element {
    if slot.settled {
        return rsx! {
            span { class: "teaksta-token tk-slot tk-correct", id: "{id}", lang: "se",
                "{slot.chosen}"
                span { class: "tk-mark", "aria-hidden": "true", "✓" }
            }
        };
    }

    let open = cursor.is_some();
    let expanded = if open { "true" } else { "false" };
    let class = if slot.tries > 0 {
        "tk-mc tk-wrong"
    } else {
        "tk-mc"
    };
    let value_class = if slot.chosen.is_empty() {
        "tk-mc-value tk-mc-value--empty"
    } else {
        "tk-mc-value"
    };
    let value = if slot.chosen.is_empty() {
        EMPTY_VALUE.to_string()
    } else {
        slot.chosen.clone()
    };
    let menu = format!("{id}-menu");
    // A shut chooser has no option under the cursor, so it names none rather
    // than pointing at an id that is not on the page.
    let active = cursor.map(|at| format!("{id}-opt-{at}"));

    rsx! {
        span { class: "tk-mc-wrap",
            button {
                r#type: "button",
                class: "{class}",
                id: "{id}",
                role: "combobox",
                "aria-haspopup": "listbox",
                "aria-expanded": "{expanded}",
                "aria-controls": "{menu}",
                "aria-activedescendant": active,
                "aria-label": "Choose the missing form",
                onclick: move |_| onmove.call(Move::Press),
                onblur: move |_| onmove.call(Move::Shut),
                onkeydown: move |event| {
                    let key = event.key().to_string();
                    if swallows(&key, open) {
                        event.prevent_default();
                    }
                    onmove.call(key_move(&key, open));
                },
                span { class: "{value_class}", lang: "se", "{value}" }
                {caret()}
            }
            if open {
                span {
                    class: "tk-mc-menu",
                    id: "{menu}",
                    role: "listbox",
                    "aria-label": "Forms of the same word",
                    for (at , choice) in choices.iter().enumerate() {
                        span {
                            key: "{at}",
                            class: "tk-mc-option",
                            id: "{id}-opt-{at}",
                            role: "option",
                            lang: "se",
                            "aria-selected": if cursor == Some(at) { "true" } else { "false" },
                            // The chooser has to keep focus while a form is
                            // taken, or its own blur would shut the menu out
                            // from under the press.
                            onmousedown: move |event| event.prevent_default(),
                            onclick: move |_| onmove.call(Move::TakeAt(at)),
                            "{choice}"
                        }
                    }
                }
            }
        }
    }
}

fn caret() -> Element {
    rsx! {
        svg {
            class: "tk-caret",
            width: "10",
            height: "6",
            view_box: "0 0 10 6",
            "aria-hidden": "true",
            path {
                d: "M1 1l4 4 4-4",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.6",
                stroke_linecap: "round",
                stroke_linejoin: "round",
            }
        }
    }
}

#[component]
pub fn McMode(markup: Rc<Markup>, topic: String, prompt: String) -> Element {
    let mut slots = use_signal(HashMap::<String, Slot>::new);
    let mut showing = use_signal(|| false);
    // One menu at a time: the open chooser is held for the whole text rather
    // than by each chooser for itself, so opening one shuts any other.
    let mut open = use_signal(|| None::<(String, usize)>);

    let wanted = markup.hits(&topic);
    let settled = slots.read().values().filter(|slot| slot.settled).count();
    let tries: u32 = slots.read().values().map(|slot| slot.tries).sum();
    let rows: Vec<Review> = review_of(&markup, &topic, |token| {
        let slot = slots.read().get(&token.id).cloned().unwrap_or_default();
        (slot.settled, (!slot.settled).then_some(slot.chosen))
    });

    rsx! {
        section { class: "mode mode-mc",
            div { class: "tk-exhead",
                p { class: "tk-prompt", lang: "se",
                    "{prompt}"
                    span { class: "tk-gloss", "{mode_gloss(MODE)}" }
                }
                ScoreChip {
                    label: "Rivttes".to_string(),
                    count: "{settled} / {wanted}",
                    gloss: format!("{tries} forms chosen so far"),
                    dots: Dots::Settled { right: settled, total: wanted },
                }
            }
            {
                enhanced_text(
                    &markup,
                    Setting::Menus,
                    |at, token| {
                        if !token.is_hit(&topic) {
                            return rsx! {
                                span { class: "teaksta-token", lang: "se", "{token.text}" }
                            };
                        }
                        let offered = choices(token);
                        let count = offered.len();
                        let slot = slots.read().get(&token.id).cloned().unwrap_or_default();
                        let id = token.id.clone();
                        let accepted = token.accepted_forms();
                        let taken = offered.clone();
                        let cursor = open()
                            .filter(|(slot, _)| *slot == token.id)
                            .map(|(_, at)| at);
                        // A slot's control has to name itself for the menu it
                        // opens and the option the keyboard is on. The
                        // enhancer's own id carries the lemma in it, so the
                        // number the token was read at is used instead.
                        let dom_id = format!("teaksta-slot-{at}");
                        rsx! {
                            McToken {
                                id: dom_id,
                                choices: offered,
                                slot,
                                cursor,
                                onmove: move |asked| {
                                    match step(cursor, asked, count) {
                                        Step::Stay => {}
                                        Step::Shut => open.set(None),
                                        Step::Open(to) => open.set(Some((id.clone(), to))),
                                        Step::Take(to) => {
                                            open.set(None);
                                            let Some(form) = taken.get(to).cloned() else {
                                                return;
                                            };
                                            let mut all = slots.write();
                                            let slot = all.entry(id.clone()).or_default();
                                            slot.settled = accepts(&accepted, &form);
                                            slot.chosen = form;
                                            slot.tries += 1;
                                        }
                                    }
                                },
                            }
                        }
                    },
                )
            }
            if showing() {
                Score {
                    right: settled,
                    total: wanted,
                    gloss: format!("{settled} of the {wanted} slots settled"),
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

    fn token(attributes: &str) -> TokenSpan {
        let block = format!("<p><span class=\"teaksta-token\" {attributes}>Viesut</span></p>");
        parse(std::slice::from_ref(&block)).tokens()[0].clone()
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

    /// Every way into the menu, which is every way a native select opens.
    #[test]
    fn enter_space_or_down_opens_a_menu() {
        for key in ["Enter", " ", "ArrowDown"] {
            assert_eq!(key_move(key, false), Move::Open, "{key}");
            assert_eq!(step(None, key_move(key, false), 5), Step::Open(0), "{key}");
        }
    }

    /// Arriving from below lands on the last form rather than walking the
    /// whole list to reach it.
    #[test]
    fn up_opens_the_menu_at_its_end() {
        assert_eq!(key_move("ArrowUp", false), Move::OpenLast);
        assert_eq!(step(None, Move::OpenLast, 5), Step::Open(4));
    }

    #[test]
    fn the_arrows_walk_the_open_menu() {
        assert_eq!(step(Some(0), key_move("ArrowDown", true), 5), Step::Open(1));
        assert_eq!(step(Some(1), key_move("ArrowUp", true), 5), Step::Open(0));
        assert_eq!(step(Some(0), Move::Last, 5), Step::Open(4));
        assert_eq!(step(Some(4), Move::First, 5), Step::Open(0));
    }

    /// The cursor wraps at both ends, which saves a keypress at either.
    #[test]
    fn the_cursor_wraps_at_both_ends() {
        assert_eq!(step(Some(4), Move::Next, 5), Step::Open(0));
        assert_eq!(step(Some(0), Move::Prev, 5), Step::Open(4));
    }

    #[test]
    fn enter_takes_the_form_under_the_cursor() {
        assert_eq!(key_move("Enter", true), Move::Take);
        assert_eq!(step(Some(2), Move::Take, 5), Step::Take(2));
        assert_eq!(step(Some(0), key_move(" ", true), 5), Step::Take(0));
    }

    /// Escape leaves without taking anything, and so does tabbing away —
    /// which must not be swallowed, or focus would be trapped in a word.
    #[test]
    fn escape_and_tab_leave_the_menu_shut() {
        assert_eq!(step(Some(3), key_move("Escape", true), 5), Step::Shut);
        assert_eq!(step(Some(3), key_move("Tab", true), 5), Step::Shut);
        assert!(swallows("Escape", true));
        assert!(!swallows("Tab", true));
        assert!(!swallows("a", true));
    }

    /// Pressing the chooser toggles it, so a second press puts the text back.
    #[test]
    fn pressing_an_open_chooser_shuts_it() {
        assert_eq!(step(None, Move::Press, 5), Step::Open(0));
        assert_eq!(step(Some(2), Move::Press, 5), Step::Shut);
    }

    #[test]
    fn a_key_the_chooser_ignores_changes_nothing() {
        assert_eq!(key_move("a", true), Move::Ignore);
        assert_eq!(step(Some(1), Move::Ignore, 5), Step::Stay);
        assert_eq!(step(None, Move::Ignore, 5), Step::Stay);
    }

    /// A slot the enhancer generated no paradigm for has nothing to open on,
    /// and must not be walked into a form that is not there.
    #[test]
    fn a_menu_with_no_forms_never_opens() {
        for asked in [Move::Open, Move::Press, Move::Next, Move::Take, Move::Last] {
            assert_eq!(step(None, asked, 0), Step::Shut, "{asked:?}");
        }
        assert_eq!(step(None, Move::Ignore, 0), Step::Stay);
    }

    /// One form is the whole menu, and walking it stays where it is.
    #[test]
    fn a_single_form_is_a_menu_of_one() {
        assert_eq!(step(None, Move::Open, 1), Step::Open(0));
        assert_eq!(step(Some(0), Move::Next, 1), Step::Open(0));
        assert_eq!(step(Some(0), Move::Prev, 1), Step::Open(0));
        assert_eq!(step(Some(0), Move::Take, 1), Step::Take(0));
    }
}
