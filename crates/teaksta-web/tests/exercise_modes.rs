//! The four exercises, rendered from the analysed text the backend really
//! answered with.
//!
//! The fixtures are `POST /api/enhance/blocks` replies for the `Substantive`
//! topic over one North Sámi page, saved as they arrived — see
//! `cargo run -p teaksta --example capture_fixtures`, which holds the page
//! they are taken from. Each mode has its own reply because each mode is
//! enhanced differently: colorize, mc and cloze carry the topic's hits alone,
//! while click carries a span for every word of the text — the hits among the
//! decoys the learner may pick instead.

use std::rc::Rc;

use dioxus::prelude::*;

use teaksta_web::api::parse_blocks;
use teaksta_web::ui::exercise::click::{ClickMode, ClickModeProps, ClickToken, Verdict};
use teaksta_web::ui::exercise::cloze::{
    ClozeMode, ClozeModeProps, ClozeToken, Slot as Written, State, slot_width,
};
use teaksta_web::ui::exercise::colorize::{ColorizeMode, ColorizeModeProps, HIT_CLASS};
use teaksta_web::ui::exercise::markup::{Markup, TokenSpan, parse};
use teaksta_web::ui::exercise::mc::{self, McMode, McModeProps, McToken, Slot as Chosen};
use teaksta_web::ui::exercise::{EnhancedText, EnhancedTextProps};

const COLORIZE: &str = include_str!("fixtures/substantive-colorize.json");
const CLICK: &str = include_str!("fixtures/substantive-click.json");
const MC: &str = include_str!("fixtures/substantive-mc.json");
const CLOZE: &str = include_str!("fixtures/substantive-cloze.json");

const TOPIC: &str = "Substantive";

/// The blocks of one saved reply, in the order the backend answered them.
fn blocks(reply: &str) -> Vec<String> {
    parse_blocks(reply)
        .expect("the backend's reply parses")
        .into_iter()
        .map(|block| block.html)
        .collect()
}

/// One saved reply, read as the exercises read it.
fn read(reply: &str) -> Markup {
    parse(&blocks(reply))
}

/// How many words of the click page belong to the topic, and how many are
/// there for the learner to mistake for one.
const HITS: usize = 8;
const DECOYS: usize = 13;

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
            markup: Rc::new(read(COLORIZE)),
            topic: TOPIC.to_string(),
            prompt: "Geahča ivdnejuvvon sániid.".to_string(),
        },
    )
}

fn click_page() -> String {
    render(
        ClickMode,
        ClickModeProps {
            markup: Rc::new(read(CLICK)),
            topic: TOPIC.to_string(),
            prompt: "Coahkkal rivttes sániid!".to_string(),
        },
    )
}

fn mc_page() -> String {
    render(
        McMode,
        McModeProps {
            markup: Rc::new(read(MC)),
            topic: TOPIC.to_string(),
            prompt: "Vállje rivttes sániid!".to_string(),
        },
    )
}

fn cloze_page() -> String {
    render(
        ClozeMode,
        ClozeModeProps {
            markup: Rc::new(read(CLOZE)),
            topic: TOPIC.to_string(),
            prompt: "Čále rivttes sániid!".to_string(),
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
            EnhancedText,
            EnhancedTextProps {
                blocks: blocks(COLORIZE),
                topic: TOPIC.to_string(),
                mode: mode.to_string(),
                prompt: "Geahča ivdnejuvvon sániid.".to_string(),
            },
        );

        assert!(html.contains(rendered), "{mode} did not render {rendered}");
    }
}

/// Interactive modes ride a looser leading so the controls in the lines never
/// push them apart unevenly; the two that only read keep the reading default.
#[test]
fn only_the_inline_modes_loosen_the_leading() {
    for (mode, inline) in [
        ("colorize", false),
        ("click", false),
        ("mc", true),
        ("cloze", true),
    ] {
        let html = render(
            EnhancedText,
            EnhancedTextProps {
                blocks: blocks(COLORIZE),
                topic: TOPIC.to_string(),
                mode: mode.to_string(),
                prompt: String::new(),
            },
        );

        assert!(html.contains("tk-prose"), "{mode}");
        assert_eq!(html.contains("tk-prose--inline"), inline, "{mode}");
    }
    // Only the chooser opens over the text, so only it keeps room underneath.
    assert!(mc_page().contains("tk-reading--menus"));
    assert!(!cloze_page().contains("tk-reading--menus"));
}

#[test]
fn the_enhancer_marks_the_nouns_it_found() {
    let markup = read(COLORIZE);

    assert_eq!(markup.hits(TOPIC), HITS);
    assert_eq!(
        token_reading(&markup, "viesu").lemma.as_deref(),
        Some("viessu")
    );
    // Colorize is asked for the topic's words and gets those and no others.
    assert_eq!(markup.tokens().len(), HITS);
    assert!(markup.tokens().iter().all(|token| token.is_hit(TOPIC)));
}

#[test]
fn the_click_text_offers_the_hits_among_decoys() {
    let markup = read(CLICK);

    // Same page, same nouns — but every other word is offered alongside them.
    assert_eq!(markup.hits(TOPIC), HITS);
    assert_eq!(markup.tokens().len(), HITS + DECOYS);

    let decoys: Vec<&TokenSpan> = markup
        .tokens()
        .iter()
        .filter(|token| !token.is_hit(TOPIC))
        .collect();
    assert_eq!(decoys.len(), DECOYS);

    // A decoy is a bare token: the topic put nothing on it, so it carries no
    // class of the topic's and no base form to give it away.
    for decoy in &decoys {
        assert_eq!(decoy.classes, ["teaksta-token"], "{:?}", decoy.text);
        assert_eq!(decoy.lemma, None, "{:?}", decoy.text);
    }
    let forms: Vec<&str> = decoys.iter().map(|decoy| decoy.text.as_str()).collect();
    for word in ["Mun", "oidnen", "ikte", "leat", "stuorrát", "lea", "logai"] {
        assert!(forms.contains(&word), "no decoy for {word}: {forms:?}");
    }

    // The hits kept the topic's own span, base form and all.
    assert_eq!(
        token_reading(&markup, "viesu").lemma.as_deref(),
        Some("viessu")
    );
    for word in ["Teakstabihttá", "viesu", "Viesut", "skuvllas", "Beana"] {
        assert!(
            token_reading(&markup, word).is_hit(TOPIC),
            "{word} is not a hit"
        );
    }
}

/// The prose the backend answered with reaches the learner whole: the
/// sentences either side of a token, the punctuation between them, and
/// nothing of the document the page was cut out of.
#[test]
fn the_text_survives_being_taken_apart() {
    let html = colorize_page();

    assert!(html.contains("Mun oidnen"));
    assert!(html.contains("leat stuorrát"));
    assert!(html.contains("viehká olgun, ja mii boahtit ruoktot"));
    assert!(html.contains("ikte."));
    assert!(!html.contains("<script"));
    for absent in ["<html", "<head", "<body", "<title", "<base"] {
        assert!(!html.contains(absent), "{absent} reached the learner");
    }
}

/// The blocks arrive one per element of the page and are rendered in that
/// order, the heading among the paragraphs.
#[test]
fn every_block_is_rendered_where_it_arrived() {
    let markup = read(COLORIZE);
    let html = colorize_page();

    assert_eq!(blocks(COLORIZE).len(), 4);
    assert_eq!(markup.blocks().len(), 4);
    assert_eq!(html.matches("class=\"enhanced-head").count(), 1);
    assert_eq!(html.matches("class=\"enhanced-line").count(), 3);

    let heading = html.find("enhanced-head").expect("the heading is rendered");
    let first = html.find("Mun oidnen").expect("the first paragraph");
    let last = html.find("viehká olgun").expect("the last paragraph");
    assert!(
        heading < first && first < last,
        "the blocks are out of order"
    );
}

/// Nothing of the old servlet's naming reaches the client. It was written as
/// span ids as well as class names, and the exercises drop the ids, so the
/// replies the backend really answered with are read here rather than the
/// rendering of them: the spelling itself is the guard, not any one of the
/// shapes it was written in.
#[test]
fn no_naming_from_before_the_rename_survives() {
    let rendered = colorize_page();

    for (name, reply) in [
        ("colorize", COLORIZE),
        ("mc", MC),
        ("cloze", CLOZE),
        ("rendered", rendered.as_str()),
    ] {
        assert!(!reply.contains("WERTi"), "{name}");
        assert!(!reply.contains("wertiview"), "{name}");
    }
}

#[test]
fn colorize_styles_every_topic_word() {
    let markup = read(COLORIZE);
    let hits = markup.hits(TOPIC);
    let html = colorize_page();

    assert_eq!(html.matches(HIT_CLASS).count(), hits);
    for form in ["Teakstabihttá", "viesu", "Viesut", "skuvllas", "Beana"] {
        assert!(html.contains(&format!(">{form}</span>")), "missing {form}");
    }
    // The enhancer's own classes ride along, and the highlight sits on top of
    // them rather than replacing them.
    assert!(html.contains(&format!("teaksta-token teaksta-Substantive {HIT_CLASS}")));
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
    let markup = read(CLICK);
    let html = click_page();

    // Every word is offered, hits and decoys alike, and none gives away which
    // it is before it is picked — not in what it looks like, and not in what
    // the markup is made of either.
    assert_eq!(
        html.matches("class=\"teaksta-token tk-pick\"").count(),
        markup.tokens().len()
    );
    assert_eq!(
        html.matches("class=\"teaksta-token tk-pick\"").count(),
        HITS + DECOYS
    );
    assert!(!html.contains("tk-correct"));
    assert!(!html.contains("tk-wrong"));
    assert!(!html.contains(HIT_CLASS));
    assert!(!html.contains("teaksta-Substantive"));
    // Only the hits are worth finding, however many words are on offer.
    assert!(html.contains(&format!("0 / {HITS}")));
    assert!(html.contains("Rivttes"));
}

#[test]
fn click_marks_a_topic_word_right() {
    let markup = read(CLICK);

    for word in ["Viesut", "viesu", "Beana", "skuvllas"] {
        assert_eq!(
            teaksta_web::ui::exercise::click::judge(&token_reading(&markup, word), TOPIC),
            Verdict::Right,
            "{word} was not judged right"
        );
    }
}

#[test]
fn click_marks_a_decoy_wrong() {
    let markup = read(CLICK);

    // The words the text really carries beside the nouns: picking one is the
    // mistake the exercise exists to catch.
    for word in ["Mun", "oidnen", "ikte", "leat", "stuorrát", "lea", "logai"] {
        assert_eq!(
            teaksta_web::ui::exercise::click::judge(&token_reading(&markup, word), TOPIC),
            Verdict::Wrong,
            "{word} was not judged wrong"
        );
    }

    // A hit belongs to its own topic and to no other.
    assert_eq!(
        teaksta_web::ui::exercise::click::judge(
            &token_reading(&markup, "Viesut"),
            "VerbConjugation"
        ),
        Verdict::Wrong
    );
}

#[test]
fn click_scores_only_the_hits() {
    let markup = read(CLICK);

    // What the score line counts: the words the topic marked, not the words
    // on offer.
    assert_eq!(markup.hits(TOPIC), HITS);
    assert!(markup.tokens().len() > HITS);

    let right = markup
        .tokens()
        .iter()
        .filter(|token| teaksta_web::ui::exercise::click::judge(token, TOPIC) == Verdict::Right)
        .count();
    let wrong = markup.tokens().len() - right;

    assert_eq!(right, HITS);
    assert_eq!(wrong, DECOYS);
}

#[component]
fn Picked(verdict: Option<Verdict>) -> Element {
    rsx! {
        ClickToken { text: "Viesut".to_string(), verdict, onchoose: move |_| {} }
    }
}

/// A judged word wears the band and a mark, because state is never colour
/// alone.
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

    assert!(right.contains("tk-correct"));
    assert!(right.contains("✓"));
    assert!(right.contains("disabled"));
    assert!(wrong.contains("tk-wrong"));
    assert!(wrong.contains("✕"));
    assert!(!wrong.contains("tk-correct"));
}

#[test]
fn mc_offers_the_servers_distractor_forms() {
    let markup = read(MC);
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

/// One chooser per slot, and not a `<select>` anywhere — a native select
/// cannot be set in the reading, and its option list is exactly what this
/// markup replaces.
#[test]
fn mc_renders_a_chooser_per_hit() {
    let markup = read(MC);
    let html = mc_page();

    assert_eq!(html.matches("class=\"tk-mc\"").count(), markup.hits(TOPIC));
    assert_eq!(
        html.matches("aria-haspopup=\"listbox\"").count(),
        markup.hits(TOPIC)
    );
    assert!(!html.contains("<select"));
    assert!(!html.contains("<option"));
    assert!(html.contains("Rivttes"));
    // Nothing is open until a learner opens something, so no menu stands on
    // the page and no form has been given away.
    assert!(!html.contains("tk-mc-menu"));
    for form in ["viessu", "vissui", "viesus", "viesuin"] {
        assert!(!html.contains(form), "{form} was on the page unasked");
    }
}

/// One chooser on its own, at whatever point in its own life a test needs.
#[component]
fn Chose(choices: Vec<String>, cursor: Option<usize>, slot: Chosen) -> Element {
    rsx! {
        McToken {
            id: "teaksta-slot-0".to_string(),
            choices,
            slot,
            cursor,
            onmove: move |_| {},
        }
    }
}

fn open_menu(choices: &[String]) -> String {
    render(
        Chose,
        ChoseProps {
            choices: choices.to_vec(),
            cursor: Some(0),
            slot: Chosen::default(),
        },
    )
}

/// Every form the backend sent reaches the open menu. The count is taken from
/// `choices` rather than written out here, so a menu that renders one option
/// where five were offered fails in this suite rather than in a browser.
#[test]
fn an_open_menu_offers_every_form() {
    let markup = read(MC);
    let offered = mc::choices(&token_reading(&markup, "viesu"));
    let html = open_menu(&offered);

    assert_eq!(offered.len(), mc::MAX_CHOICES);
    assert_eq!(html.matches("role=\"option\"").count(), offered.len());
    assert_eq!(
        html.matches("class=\"tk-mc-option\"").count(),
        offered.len()
    );
    for form in &offered {
        assert!(html.contains(&format!(">{form}</span>")), "missing {form}");
    }
    assert!(html.contains("role=\"listbox\""));
    assert!(html.contains("aria-expanded=\"true\""));
}

/// The menu lives inside the paragraph it belongs to, and a list there would
/// close that paragraph out from under it. So it is spans, and stays spans.
#[test]
fn an_open_menu_is_not_a_list() {
    let markup = read(MC);
    let html = open_menu(&mc::choices(&token_reading(&markup, "viesu")));

    for element in ["<ul", "</ul>", "<li", "</li>", "<select", "<option"] {
        assert!(!html.contains(element), "the menu holds {element}");
    }
}

/// Focus stays on the chooser the whole time its menu is open — there is
/// nothing inside the menu to move it to — so the option the keyboard is on
/// has to be named rather than focused.
#[test]
fn an_open_menu_names_its_active_option() {
    let html = render(
        Chose,
        ChoseProps {
            choices: vec!["Viesu".to_string(), "Viesut".to_string()],
            cursor: Some(1),
            slot: Chosen::default(),
        },
    );

    assert!(html.contains("aria-activedescendant=\"teaksta-slot-0-opt-1\""));
    assert!(html.contains("id=\"teaksta-slot-0-opt-1\""));
    assert!(html.contains("aria-controls=\"teaksta-slot-0-menu\""));
    assert!(html.contains("id=\"teaksta-slot-0-menu\""));
    assert_eq!(html.matches("aria-selected=\"true\"").count(), 1);
    assert_eq!(html.matches("aria-selected=\"false\"").count(), 1);
}

#[test]
fn mc_hides_the_answer_behind_the_capitals() {
    let markup = read(MC);
    let offered = mc::choices(&token_reading(&markup, "Viesut"));

    assert_eq!(offered.len(), mc::MAX_CHOICES);
    assert!(offered.iter().all(|form| form.starts_with('V')));
    assert!(offered.contains(&"Viesut".to_string()));
}

#[test]
fn mc_accepts_only_the_form_read() {
    let markup = read(MC);
    let token = token_reading(&markup, "viesu");

    assert!(token.accepts("viesu"));
    assert!(token.accepts("Viesu"));
    assert!(!token.accepts("viessu"));
    assert!(!token.accepts("viesuin"));
}

#[test]
fn an_ungenerable_form_stays_answerable() {
    let markup = read(MC);
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

/// A slot that settles stops being a control and becomes green text, so the
/// sentence gets shorter as the learner gets further.
#[test]
fn a_right_choice_fixes_the_slot() {
    let offered = vec!["Viesut".to_string(), "Viesuid".to_string()];
    let right = render(
        Chose,
        ChoseProps {
            choices: offered.clone(),
            cursor: None,
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
            choices: offered,
            cursor: None,
            slot: Chosen {
                chosen: "Viesuid".to_string(),
                settled: false,
                tries: 1,
            },
        },
    );

    assert!(right.contains("tk-slot tk-correct"));
    assert!(right.contains("✓"));
    assert!(!right.contains("tk-mc"));
    assert!(!right.contains("<button"));

    // A form that did not fit leaves the slot answerable, says so in red, and
    // holds on to what was tried.
    assert!(wrong.contains("class=\"tk-mc tk-wrong\""));
    assert!(wrong.contains(">Viesuid</span>"));
    assert!(wrong.contains("aria-expanded=\"false\""));
}

#[test]
fn the_cloze_page_carries_parallel_forms() {
    let markup = read(CLOZE);
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
    let markup = read(CLOZE);
    let token = token_of_lemma(&markup, "skuvla");

    assert_eq!(token.possible_forms, ["skuvllas", "skuvllain"]);
    assert!(token.accepts("skuvllas"));
    assert!(token.accepts("skuvllain"));
    assert!(token.accepts(" SKUVLLAIN "));
}

#[test]
fn cloze_rejects_a_form_off_the_paradigm() {
    let markup = read(CLOZE);
    let token = token_of_lemma(&markup, "skuvla");

    assert!(!token.accepts("skuvla"));
    assert!(!token.accepts("skuvllii"));
    assert!(!token.accepts(""));
    assert!(!token_of_lemma(&markup, "girji").accepts("girji"));
}

/// What a learner who asks for help is given is the base form, and only that.
/// Where the text inflected the word, the hint therefore stops short of the
/// answer — including on the parallel-form slots, where it gives up neither of
/// the several forms the exercise exists to teach.
///
/// `beana` is the exception the rule survives: its nominative singular *is*
/// its lemma, so there the hint and the answer are the same word. The hint
/// still gives up no more than the base form — the word simply has only one.
#[test]
fn the_hint_shows_the_lemma_alone() {
    let markup = read(CLOZE);

    for lemma in ["skuvla", "viessu", "beana", "girji"] {
        let token = token_of_lemma(&markup, lemma);

        assert_eq!(token.hint(), Some(lemma));
    }

    for token in markup.tokens() {
        let lemma = token.hint().expect("every hit carries a lemma");
        if lemma == token.text.to_lowercase() {
            continue;
        }
        for form in token.accepted_forms() {
            assert_ne!(lemma, form, "{lemma} handed over an accepted form");
        }
    }

    let parallel = token_of_lemma(&markup, "skuvla");
    assert_eq!(parallel.possible_forms, ["skuvllas", "skuvllain"]);
    assert_eq!(parallel.hint(), Some("skuvla"));
}

#[test]
fn cloze_renders_a_box_per_hit() {
    let markup = read(CLOZE);
    let html = cloze_page();
    let hits = markup.hits(TOPIC);

    assert_eq!(hits, 7);
    assert_eq!(html.matches("class=\"tk-cloze-input\"").count(), hits);
    assert_eq!(html.matches("class=\"tk-hint-btn\"").count(), hits);
    assert!(html.contains("Mun oidnen"));
    // Every slot is drawn as wide as the form it expects, in characters of
    // the reading serif rather than in a fixed few pixels.
    for token in markup.tokens() {
        let width = slot_width(&token.text);
        assert!(
            html.contains(&format!("width: {width}ch")),
            "{}",
            token.text
        );
    }
}

/// Nothing the slot wants is on the page before it is answered — neither the
/// form the text had nor the base form it came from. The hint is a press
/// away, and until it is pressed it gives up nothing.
#[test]
fn cloze_keeps_the_form_and_the_lemma_back() {
    let markup = read(CLOZE);
    let html = cloze_page();

    assert!(!html.contains("skuvllas"));
    for token in markup.tokens() {
        let lemma = token.lemma.as_deref().expect("every hit carries a lemma");
        assert!(!html.contains(lemma), "{lemma} was on the page unasked");
        assert!(
            !html.contains(&token.text),
            "{} was on the page",
            token.text
        );
    }
}

#[component]
fn Wrote(slot: Written) -> Element {
    rsx! {
        ClozeToken {
            lemma: Some("skuvla".to_string()),
            width: 10,
            slot,
            onwrite: move |_| {},
            onhint: move |()| {},
        }
    }
}

fn wrote(slot: Written) -> String {
    render(Wrote, WroteProps { slot })
}

#[test]
fn a_written_form_shows_how_it_went() {
    let right = wrote(Written {
        written: "skuvllain".to_string(),
        state: State::Right,
        tries: 1,
        hinted: false,
    });
    let wrong = wrote(Written {
        written: "skuvla".to_string(),
        state: State::Wrong,
        tries: 1,
        hinted: false,
    });

    assert!(right.contains("tk-cloze-input tk-correct"));
    assert!(right.contains("skuvllain"));
    assert!(right.contains("✓"));
    // A settled slot stops asking: it keeps the form, and the hint goes with
    // the question it answered.
    assert!(right.contains("readonly"));
    assert!(!right.contains("tk-hint"));

    // A form that did not fit leaves the slot editable, so it can be
    // corrected where it stands.
    assert!(wrong.contains("tk-cloze-input tk-wrong"));
    assert!(wrong.contains("✕"));
    assert!(!wrong.contains("readonly"));
    assert!(wrong.contains("tk-hint-btn"));
}

/// The hint that was asked for gives back the lemma, in the yellow that means
/// help, and the slot stays open to be answered from it.
#[test]
fn a_hinted_slot_shows_the_lemma() {
    let hinted = wrote(Written {
        written: String::new(),
        state: State::Open,
        tries: 0,
        hinted: true,
    });

    assert!(hinted.contains("class=\"tk-hint\""));
    assert!(hinted.contains("class=\"tk-hint-mark\""));
    assert!(hinted.contains(">skuvla</em>"));
    // The button has done its work and is gone, and nothing that would have
    // been accepted came with the lemma.
    assert!(!hinted.contains("tk-hint-btn"));
    assert!(!hinted.contains("skuvllas"));
    assert!(!hinted.contains("skuvllain"));
    assert!(hinted.contains("<input"));
}
