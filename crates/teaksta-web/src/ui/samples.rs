//! The ready-made texts a topic offers, for a learner with no page of their
//! own in mind.
//!
//! These are the recommended pages the webapp teaksta replaces listed beside
//! six of its topics, and they work as those did: a text is a page elsewhere,
//! and picking one starts the exercise on its address exactly as typing the
//! address would.

use dioxus::prelude::*;
use teaksta_l10n::{FluentArgs, args, en, en_with, has, se, se_or_en, se_with};

use crate::api::SampleText;

/// The list, under its heading. A topic that offers nothing shows nothing.
#[component]
pub fn Samples(texts: Vec<SampleText>, onpick: EventHandler<String>) -> Element {
    if texts.is_empty() {
        return rsx! {};
    }

    rsx! {
        section { class: "samples",
            div { class: "tk-fieldhead",
                h2 { lang: "se",
                    "{se_or_en(\"texts-heading\")} "
                    span { class: "tk-gloss", {en("texts-heading")} }
                }
            }
            ul { class: "sample-list",
                for text in texts {
                    li { key: "{text.url}",
                        SampleButton { text, onpick }
                    }
                }
            }
        }
    }
}

#[component]
fn SampleButton(text: SampleText, onpick: EventHandler<String>) -> Element {
    let (sme, english) = about(&text);
    let url = text.url.clone();

    rsx! {
        button {
            r#type: "button",
            class: "sample",
            onclick: move |_| onpick.call(url.clone()),
            span { class: "sample-title", lang: "se", "{text.title}" }
            if let Some(sme) = sme {
                span { class: "sample-about sample-about--sme", lang: "se", "{sme}" }
            }
            span { class: "sample-about", "{english}" }
        }
    }
}

/// What kind of text it is and how long, with the tense its verbs are in
/// where the topic says: in North Sámi where the words for it are written,
/// and in English.
fn about(text: &SampleText) -> (Option<String>, String) {
    let genre_id = format!("genre-{}", text.genre);
    let tense_id = text.tense.as_ref().map(|tense| format!("tense-{tense}"));
    let id = if tense_id.is_some() {
        "text-about-tense"
    } else {
        "text-about"
    };

    let filled = |genre: String, tense: Option<String>| -> FluentArgs<'static> {
        let mut filled = args([("genre", genre), ("tense", tense.unwrap_or_default())]);
        filled.set("words", text.words);
        filled
    };

    let known = has(&genre_id);
    let genre_se = if known {
        se(&genre_id)
    } else {
        Some(text.genre.clone())
    };
    let genre_en = if known {
        en(&genre_id)
    } else {
        text.genre.clone()
    };
    let tense_se = tense_id.as_ref().map(|id| se(id));
    let tense_en = tense_id.as_ref().map(|id| en(id));

    // The North Sámi line is written only when every word in it is.
    let sme = match (genre_se, tense_se) {
        (Some(genre), None) => se_with(id, &filled(genre, None)),
        (Some(genre), Some(Some(tense))) => se_with(id, &filled(genre, Some(tense))),
        _ => None,
    };
    let english = en_with(id, &filled(genre_en, tense_en));

    (sme, english)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(genre: &str, tense: Option<&str>) -> SampleText {
        SampleText {
            title: "Mearragáttis".to_string(),
            url: "https://kursa.oahpa.no/2011/11/01/mearragattis/".to_string(),
            genre: genre.to_string(),
            words: 190,
            tense: tense.map(str::to_string),
        }
    }

    /// The description the webapp gave, word for word, with its English.
    #[test]
    fn a_text_is_described_as_the_webapp_did() {
        let (sme, english) = about(&text("story", None));
        assert_eq!(sme.as_deref(), Some("muitalus, su. 190 sáni"));
        assert_eq!(english, "story, about 190 words");

        let (sme, english) = about(&text("story", Some("present")));
        assert_eq!(
            sme.as_deref(),
            Some("muitalus, su. 190 sáni, vearbbat leat preseanssas")
        );
        assert_eq!(
            english,
            "story, about 190 words, the verbs are in the present tense"
        );
    }

    /// A kind of text a deployment adds is shown by its own name.
    #[test]
    fn an_unknown_genre_reads_as_its_name() {
        let (sme, english) = about(&text("poem", None));
        assert_eq!(sme.as_deref(), Some("poem, su. 190 sáni"));
        assert_eq!(english, "poem, about 190 words");
    }
}
