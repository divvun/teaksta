//! The words teaksta shows, in North Sámi and in English.
//!
//! Every string a learner or a teacher reads lives in `locales/se.ftl` and
//! `locales/en.ftl` and is looked up here by message id, by the web client
//! and by the server alike. The two files are compiled in, so neither side
//! carries a tree beside it.
//!
//! The interface is bilingual on one screen rather than switched between
//! languages: a North Sámi line is shown with its English gloss beside it. So
//! both bundles are always read, and nothing negotiates a locale. English is
//! the reference — every message has an English value — and North Sámi is
//! present where it has been written. A message the North Sámi file lacks is
//! shown in English alone, which is how a string waiting on a translator
//! reads until one adds it to `se.ftl`.

use std::sync::LazyLock;

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentResource, FluentValue};
use unic_langid::LanguageIdentifier;

pub use fluent_bundle::FluentArgs;

/// The North Sámi messages, as written.
pub const SE_SOURCE: &str = include_str!("../locales/se.ftl");

/// The English messages, as written.
pub const EN_SOURCE: &str = include_str!("../locales/en.ftl");

static SE: LazyLock<FluentBundle<FluentResource>> = LazyLock::new(|| bundle("se", SE_SOURCE));
static EN: LazyLock<FluentBundle<FluentResource>> = LazyLock::new(|| bundle("en", EN_SOURCE));

/// One language's messages. The files are compiled in and their tests parse
/// them, so a file that fails here is a build nobody ran the tests on.
///
/// Isolation marks are off: a placeable here is a host name, a count or a
/// word form set into text the page lays out itself, and the marks would
/// reach the page as stray characters around each of them.
fn bundle(language: &str, source: &str) -> FluentBundle<FluentResource> {
    let language: LanguageIdentifier = language.parse().expect("a language identifier");
    let resource = FluentResource::try_new(source.to_string())
        .unwrap_or_else(|(_, errors)| panic!("the {language} messages parse: {errors:?}"));
    let mut bundle = FluentBundle::new_concurrent(vec![language.clone()]);
    bundle.set_use_isolating(false);
    bundle.add_resource(resource).unwrap_or_else(|errors| {
        panic!("the {language} messages are each defined once: {errors:?}")
    });
    bundle
}

fn format(
    bundle: &FluentBundle<FluentResource>,
    id: &str,
    args: Option<&FluentArgs>,
) -> Option<String> {
    let pattern = bundle.get_message(id)?.value()?;
    let mut errors = Vec::new();
    Some(
        bundle
            .format_pattern(pattern, args, &mut errors)
            .into_owned(),
    )
}

/// The North Sámi for a message, or none where it has not been written.
pub fn se(id: &str) -> Option<String> {
    format(&SE, id, None)
}

/// [`se`] for a message with placeables.
pub fn se_with(id: &str, args: &FluentArgs) -> Option<String> {
    format(&SE, id, Some(args))
}

/// The English for a message. An id with no English reads as the id itself,
/// so a misspelt one shows on the page rather than as an empty line; the
/// client's tests check that every id it asks for is defined.
pub fn en(id: &str) -> String {
    format(&EN, id, None).unwrap_or_else(|| id.to_string())
}

/// [`en`] for a message with placeables.
pub fn en_with(id: &str, args: &FluentArgs) -> String {
    format(&EN, id, Some(args)).unwrap_or_else(|| id.to_string())
}

/// What is shown where only one language fits — a button, a link, a label
/// the server sends: the North Sámi, or the English where there is none.
pub fn se_or_en(id: &str) -> String {
    se(id).unwrap_or_else(|| en(id))
}

/// Whether a message is defined, for ids built from a name a deployment
/// chose, which this file may know nothing about.
pub fn has(id: &str) -> bool {
    EN.has_message(id)
}

/// Placeables for one message, from name and value pairs.
pub fn args<'a, V>(pairs: impl IntoIterator<Item = (&'a str, V)>) -> FluentArgs<'a>
where
    V: Into<FluentValue<'a>>,
{
    pairs.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    use fluent_bundle::FluentResource;
    use fluent_syntax::ast::Entry;

    fn ids(source: &str) -> Vec<String> {
        let resource = FluentResource::try_new(source.to_string()).expect("the messages parse");
        resource
            .entries()
            .filter_map(|entry| match entry {
                Entry::Message(message) => Some(message.id.name.to_string()),
                _ => None,
            })
            .collect()
    }

    /// English is the reference, so there is nothing in North Sámi that has
    /// no English beside it.
    #[test]
    fn every_north_sami_message_has_english() {
        let english = ids(EN_SOURCE);
        for id in ids(SE_SOURCE) {
            assert!(english.contains(&id), "{id} is in se.ftl and not in en.ftl");
        }
    }

    #[test]
    fn both_files_load() {
        assert!(!ids(SE_SOURCE).is_empty());
        assert!(!ids(EN_SOURCE).is_empty());
        assert_eq!(se("send").as_deref(), Some("S\u{e1}dde"));
        assert_eq!(en("send"), "Send");
    }

    /// A message with no North Sámi is shown in English alone.
    #[test]
    fn an_untranslated_message_falls_back_to_english() {
        assert_eq!(se("start-heading"), None);
        assert_eq!(se_or_en("start-heading"), en("start-heading"));
    }

    #[test]
    fn an_unknown_id_reads_as_itself() {
        assert_eq!(en("no-such-message"), "no-such-message");
        assert!(!has("no-such-message"));
    }

    /// A placeable is set without isolation marks, and a count selects its
    /// plural.
    #[test]
    fn placeables_are_set_plainly() {
        let said = en_with(
            "page-failure-site-error",
            &args([
                ("site", FluentValue::from("kursa.oahpa.no")),
                ("status", 502.into()),
            ]),
        );
        assert!(
            said.starts_with("kursa.oahpa.no answered with an error (502)."),
            "{said}"
        );

        assert_eq!(
            en_with("mc-tries", &args([("tries", 1)])),
            "1 form chosen so far"
        );
        assert_eq!(
            en_with("mc-tries", &args([("tries", 2)])),
            "2 forms chosen so far"
        );
    }
}
