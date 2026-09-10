//! The four exercises, rendered from pages the backend really answered with.
//!
//! The fixtures are `GET /api/enhance` replies for the `Substantive` topic over
//! one North Sámi page, saved as they arrived. The colorize reply carries the
//! markup the click exercise reads as well: the enhancer writes the same spans
//! for both and only the instruction differs.

use std::rc::Rc;

use dioxus::prelude::*;

use teaksta_web::ui::exercise::click::{ClickMode, ClickModeProps, ClickToken, Verdict};
use teaksta_web::ui::exercise::cloze::{ClozeMode, ClozeModeProps, ClozeToken, Slot as Written};
use teaksta_web::ui::exercise::colorize::{ColorizeMode, ColorizeModeProps};
use teaksta_web::ui::exercise::markup::{Markup, TokenSpan, parse};
use teaksta_web::ui::exercise::mc::{self, McMode, McModeProps, McToken, Slot as Chosen};
use teaksta_web::ui::exercise::{EnhancedPage, EnhancedPageProps};

const COLORIZE: &str = include_str!("fixtures/substantive-colorize.html");
const MC: &str = include_str!("fixtures/substantive-mc.html");
const CLOZE: &str = include_str!("fixtures/substantive-cloze.html");

const TOPIC: &str = "Substantive";

fn render<P: Clone + 'static>(component: fn(P) -> Element, props: P) -> String {
    let mut dom = VirtualDom::new_with_props(component, props);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn token_reading(markup: &Markup, form: &str) -> TokenSpan {
    markup
        .tokens()
        .iter()
        .find(|token| token.text == form)
        .unwrap_or_else(|| panic!("the fixture carries a token for {form}"))
        .clone()
}

fn token_of_lemma(markup: &Markup, lemma: &str) -> TokenSpan {
    markup
        .tokens()
        .iter()
        .find(|token| token.lemma.as_deref() == Some(lemma))
        .unwrap_or_else(|| panic!("the fixture carries a token for {lemma}"))
        .clone()
}

fn colorize_page() -> String {
    render(
        ColorizeMode,
        ColorizeModeProps {
            markup: Rc::new(parse(COLORIZE)),
            topic: TOPIC.to_string(),
        },
    )
}

fn click_page() -> String {
    render(
        ClickMode,
        ClickModeProps {
            markup: Rc::new(parse(COLORIZE)),
            topic: TOPIC.to_string(),
        },
    )
}

fn mc_page() -> String {
    render(
        McMode,
        McModeProps {
            markup: Rc::new(parse(MC)),
            topic: TOPIC.to_string(),
        },
    )
}

fn cloze_page() -> String {
    render(
        ClozeMode,
        ClozeModeProps {
            markup: Rc::new(parse(CLOZE)),
            topic: TOPIC.to_string(),
        },
    )
}

#[test]
fn the_mode_parameter_picks_the_exercise() {
    for (mode, rendered) in [
        ("colorize", "mode-colorize"),
        ("click", "mode-click"),
        ("mc", "mode-mc"),
        ("cloze", "mode-cloze"),
        ("nonesuch", "mode-colorize"),
    ] {
        let html = render(
            EnhancedPage,
            EnhancedPageProps {
                html: COLORIZE.to_string(),
                topic: TOPIC.to_string(),
                mode: mode.to_string(),
            },
        );

        assert!(html.contains(rendered), "{mode} did not render {rendered}");
    }
}

#[test]
fn the_enhancer_marks_the_nouns_it_found() {
    let markup = parse(COLORIZE);

    assert_eq!(markup.hits(TOPIC), 8);
    assert_eq!(
        token_reading(&markup, "viesu").lemma.as_deref(),
        Some("viessu")
    );
    assert!(markup.tokens().iter().all(|token| token.is_hit(TOPIC)));
}

#[test]
fn the_page_survives_being_taken_apart() {
    let html = colorize_page();

    assert!(html.contains("Mun oidnen"));
    assert!(html.contains("leat stuorrát"));
    assert!(html.contains("viehká olgun, ja mii boahtit ruoktot"));
    assert!(!html.contains("<script"));
    assert!(!html.contains("wertiview"));
}

#[test]
fn colorize_styles_every_topic_word() {
    let markup = parse(COLORIZE);
    let hits = markup.hits(TOPIC);
    let html = colorize_page();

    assert_eq!(html.matches("class=\"token token-hit\"").count(), hits);
    for form in ["Teakstabihttá", "viesu", "Viesut", "skuvllas", "Beana"] {
        assert!(html.contains(&format!(">{form}</span>")), "missing {form}");
    }
}

#[test]
fn colorize_asks_the_learner_nothing() {
    let html = colorize_page();

    assert!(!html.contains("<select"));
    assert!(!html.contains("<input"));
    assert!(!html.contains("<button"));
}

#[test]
fn click_leaves_every_word_unmarked() {
    let markup = parse(COLORIZE);
    let html = click_page();

    assert_eq!(
        html.matches("class=\"token token-pick\"").count(),
        markup.tokens().len()
    );
    assert!(!html.contains("pick-right"));
    assert!(!html.contains("pick-wrong"));
    assert!(html.contains("Rivttes: 0 / 8"));
}

#[test]
fn click_marks_a_topic_word_right() {
    let markup = parse(COLORIZE);
    let token = token_reading(&markup, "Viesut");

    assert_eq!(
        teaksta_web::ui::exercise::click::judge(&token, TOPIC),
        Verdict::Right
    );
}

#[test]
fn click_marks_any_other_word_wrong() {
    let markup = parse(COLORIZE);
    let noun = token_reading(&markup, "Viesut");
    // The shape the generic token enhancer writes for a word outside the topic.
    let plain = parse("<p><span class=\"teaksta-token\" id=\"x\">ikte</span></p>");

    assert_eq!(
        teaksta_web::ui::exercise::click::judge(&noun, "VerbConjugation"),
        Verdict::Wrong
    );
    assert_eq!(
        teaksta_web::ui::exercise::click::judge(&plain.tokens()[0], TOPIC),
        Verdict::Wrong
    );
}

#[component]
fn Picked(verdict: Option<Verdict>) -> Element {
    rsx! {
        ClickToken { text: "Viesut".to_string(), verdict, onchoose: move |_| {} }
    }
}

#[test]
fn a_judged_click_shows_its_verdict() {
    let right = render(
        Picked,
        PickedProps {
            verdict: Some(Verdict::Right),
        },
    );
    let wrong = render(
        Picked,
        PickedProps {
            verdict: Some(Verdict::Wrong),
        },
    );

    assert!(right.contains("pick-right"));
    assert!(right.contains("disabled"));
    assert!(wrong.contains("pick-wrong"));
    assert!(!wrong.contains("pick-right"));
}

#[test]
fn mc_offers_the_servers_distractor_forms() {
    let markup = parse(MC);
    let token = token_reading(&markup, "viesu");

    assert_eq!(
        token.distractors,
        ["viessu", "vissui", "viesus", "viesuin", "viesu"]
    );
    let mut offered = mc::choices(&token);
    let mut sent = token.distractors.clone();
    offered.sort();
    sent.sort();
    assert_eq!(offered, sent);
}

#[test]
fn mc_renders_a_select_per_hit() {
    let markup = parse(MC);
    let html = mc_page();

    assert_eq!(html.matches("<select").count(), markup.hits(TOPIC));
    for form in ["viessu", "vissui", "viesus", "viesuin"] {
        assert!(
            html.contains(&format!(">{form}</option>")),
            "missing {form}"
        );
    }
    assert!(html.contains("Rivttes: 0 / 8"));
}

#[test]
fn mc_hides_the_answer_behind_the_capitals() {
    let markup = parse(MC);
    let offered = mc::choices(&token_reading(&markup, "Viesut"));

    assert_eq!(offered.len(), mc::MAX_CHOICES);
    assert!(offered.iter().all(|form| form.starts_with('V')));
    assert!(offered.contains(&"Viesut".to_string()));
}

#[test]
fn mc_accepts_only_the_form_read() {
    let markup = parse(MC);
    let token = token_reading(&markup, "viesu");

    assert!(token.accepts("viesu"));
    assert!(token.accepts("Viesu"));
    assert!(!token.accepts("viessu"));
    assert!(!token.accepts("viesuin"));
}

#[test]
fn an_ungenerable_form_stays_answerable() {
    let markup = parse(MC);
    let token = token_reading(&markup, "Teakstabihttá");
    let offered = mc::choices(&token);

    // The enhancer could not generate this compound and wrote the reading it
    // analysed instead, so the offered forms come from the page and the
    // distractors, never from the answer attribute.
    assert!(token.answer.iter().any(|form| form.contains('+')));
    assert!(offered.iter().all(|form| !form.contains('+')));
    assert!(offered.contains(&"Teakstabihttá".to_string()));
    assert!(token.accepts("teakstabihttá"));
}

#[component]
fn Chose(slot: Chosen) -> Element {
    rsx! {
        McToken {
            choices: vec!["Viesut".to_string(), "Viesuid".to_string()],
            slot,
            onchoose: move |_| {},
        }
    }
}

#[test]
fn a_right_choice_fixes_the_slot() {
    let right = render(
        Chose,
        ChoseProps {
            slot: Chosen {
                chosen: "Viesut".to_string(),
                settled: true,
                tries: 1,
            },
        },
    );
    let wrong = render(
        Chose,
        ChoseProps {
            slot: Chosen {
                chosen: "Viesuid".to_string(),
                settled: false,
                tries: 1,
            },
        },
    );

    assert!(right.contains("pick-right"));
    assert!(!right.contains("<select"));
    assert!(wrong.contains("mc-wrong"));
    assert!(wrong.contains("<select"));
}

#[test]
fn the_cloze_page_carries_parallel_forms() {
    let markup = parse(CLOZE);
    let parallel = markup
        .tokens()
        .iter()
        .filter(|token| token.possible_forms.len() > 1)
        .count();

    assert!(parallel >= 3, "{parallel} tokens with parallel forms");
    for token in markup.tokens() {
        assert!(token.accepts(&token.text), "{} rejected", token.text);
    }
}

#[test]
fn cloze_accepts_every_parallel_form() {
    let markup = parse(CLOZE);
    let token = token_of_lemma(&markup, "skuvla");

    assert_eq!(token.possible_forms, ["skuvllas", "skuvllain"]);
    assert!(token.accepts("skuvllas"));
    assert!(token.accepts("skuvllain"));
    assert!(token.accepts(" SKUVLLAIN "));
}

#[test]
fn cloze_rejects_a_form_off_the_paradigm() {
    let markup = parse(CLOZE);
    let token = token_of_lemma(&markup, "skuvla");

    assert!(!token.accepts("skuvla"));
    assert!(!token.accepts("skuvllii"));
    assert!(!token.accepts(""));
    assert!(!token_of_lemma(&markup, "girji").accepts("girji"));
}

#[test]
fn the_hint_shows_the_parallel_forms() {
    let markup = parse(CLOZE);

    assert_eq!(
        token_of_lemma(&markup, "skuvla").hint(),
        "skuvllas/skuvllain"
    );
    assert_eq!(token_of_lemma(&markup, "viessu").hint(), "viesu/viesuid");
    assert_eq!(token_of_lemma(&markup, "beana").hint(), "beana");
}

#[test]
fn cloze_renders_a_box_per_hit() {
    let markup = parse(CLOZE);
    let html = cloze_page();
    let hits = markup.hits(TOPIC);

    assert_eq!(hits, 7);
    assert_eq!(html.matches("class=\"cloze-box\"").count(), hits);
    assert_eq!(html.matches("class=\"cloze-hint\"").count(), hits);
    assert!(html.contains("(viessu)"));
    assert!(html.contains("Mun oidnen"));
    assert!(!html.contains("skuvllas"));
}

#[component]
fn Wrote(slot: Written) -> Element {
    rsx! {
        ClozeToken {
            lemma: Some("skuvla".to_string()),
            slot,
            onwrite: move |_| {},
            onhint: move |()| {},
        }
    }
}

#[test]
fn a_written_form_shows_how_it_went() {
    use teaksta_web::ui::exercise::cloze::State;

    let right = render(
        Wrote,
        WroteProps {
            slot: Written {
                written: "skuvllain".to_string(),
                state: State::Right,
                tries: 1,
            },
        },
    );
    let wrong = render(
        Wrote,
        WroteProps {
            slot: Written {
                written: "skuvla".to_string(),
                state: State::Wrong,
                tries: 1,
            },
        },
    );
    let shown = render(
        Wrote,
        WroteProps {
            slot: Written {
                written: "skuvllas/skuvllain".to_string(),
                state: State::Shown,
                tries: 0,
            },
        },
    );

    assert!(right.contains("pick-right"));
    assert!(right.contains("skuvllain"));
    assert!(!right.contains("<input"));
    assert!(wrong.contains("cloze-miss"));
    assert!(wrong.contains("<input"));
    assert!(shown.contains("cloze-shown"));
    assert!(shown.contains("skuvllas/skuvllain"));
}
