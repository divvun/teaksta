//! Produces an HTML document with enhancements from a document carrying
//! Enhancements and the page they were found in.
//!
//! Author: Adriane Boyd
//!
//! The page is rendered from the document's own map of it, so the only thing
//! left to add here is the base URL the fetched page's relative links need.

use anyhow::Result;

use crate::server::api::Mode;
use crate::types::Document;
use crate::util::html_utils;

/// North Sámi names for the topics and the exercise types, for a caller
/// putting a reminder of the chosen exercise on the page.
#[rustfmt::skip]
static SAMI_LABELS: &[(&str, &str)] = &[
    ("SubstantiveSingular", "Substantiivvat ovttaidlogus"),
    ("SubstantivePlural", "Substantiivvat m\u{e1}\u{14b}ggaidlogus"),
    ("VerbConjugation", "Finihtta vearbbat"),
    ("NegVerbs", "Biehttalanvearbbat"),
    ("InfiniteVerbs", "Infinihtta vearbbat"),
    ("Conjunctions", "Konjunk\u{161}uvnnat"),
    ("Substantive", "Substantiivvat"),
    ("Subject", "Subjeakta"),
    ("Object", "Objeakta"),
    ("Adverbial", "Adverbi\u{e1}la"),
    ("colorize", "Geah\u{10d}a ivdnejuvvon s\u{e1}niid."),
    ("click", "Coahkkal rivttes s\u{e1}niid!"),
    ("mc", "V\u{e1}llje rivttes s\u{e1}niid!"),
    ("cloze", "\u{10c}\u{e1}le rivttes s\u{e1}niid!"),
];

/// The North Sámi name of a topic or exercise type, if it has one.
pub fn sami_label(name: &str) -> Option<&'static str> {
    SAMI_LABELS
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, label)| *label)
}

/// The chosen topic and exercise type in North Sámi, as a short reminder of
/// what the learner is looking at. A name with no North Sámi label is used
/// as it stands.
pub fn topic_title(activity: &str, enhancement: Option<&str>) -> String {
    let topic = sami_label(activity).unwrap_or(activity);
    match enhancement {
        Some(enhancement) => format!(
            "{}: {}",
            topic,
            sami_label(enhancement).unwrap_or(enhancement)
        ),
        None => topic.to_string(),
    }
}

// [spec:teaksta:def:sme.src.main.java.werti.util.html-enhancer.html-enhancer]
pub struct HtmlEnhancer<'a> {
    cas: &'a Document,
}

impl<'a> HtmlEnhancer<'a> {
    // [spec:teaksta:def:sme.src.main.java.werti.util.html-enhancer.html-enhancer.html-enhancer-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.html-enhancer-fn]
    pub fn new(c_cas: &'a Document) -> Self {
        HtmlEnhancer { cas: c_cas }
    }

    /// Converts an HTML document with Enhancements to an HTML string. The
    /// topic reaches the page through the client rather than through the
    /// markup, so only the requested exercise is read here.
    // [spec:teaksta:def:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn+5]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn+5]
    pub fn enhance(&self, mode: Option<Mode>, base_url: &str) -> Result<String> {
        html_utils::render_page(&self.cas.page, self.cas, mode, Some(base_url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PIPELINE_LANGUAGE;

    use crate::types::Enhancement;
    use crate::util::html_utils;

    const PAGE: &str =
        "<html><head><title>Old</title></head><body><p>Mun oidnen viesu.</p></body></html>";

    /// A document seeded from `PAGE`, carrying one enhancement over `viesu`.
    fn analysed(relevant: bool) -> Document {
        let (mut cas, map) = html_utils::extract(PAGE);
        cas.page = map;
        cas.enhancements.push(Enhancement {
            begin: 11,
            end: 16,
            enhance_start: "<span id=\"teaksta-span-1\" class=\"teaksta-token\">".to_string(),
            enhance_end: "</span>".to_string(),
            relevant,
        });
        cas
    }

    fn enhance(cas: &Document, base_url: &str, mode: Option<Mode>) -> String {
        HtmlEnhancer::new(cas).enhance(mode, base_url).unwrap()
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.html-enhancer-fn/test]
    #[test]
    fn the_constructor_stores_the_cas_by_reference() {
        let cas = Document::new(PAGE, PIPELINE_LANGUAGE);

        let enhancer = HtmlEnhancer::new(&cas);

        assert!(std::ptr::eq(enhancer.cas, &cas));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn+5/test]
    #[test]
    fn head_gets_the_base_url_and_nothing_else() {
        let html = enhance(
            &analysed(true),
            "http://example.org/page.html",
            Some(Mode::Colorize),
        );

        assert!(
            html.contains("<base href=\"http://example.org/page.html\">"),
            "{}",
            html
        );
        assert!(!html.contains("<script"), "{}", html);
        assert!(!html.contains("js-lib"), "{}", html);
        assert!(html.contains("<title>Old</title>"), "{}", html);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn+5/test]
    #[test]
    fn the_enhanced_span_is_wrapped_around_its_text() {
        let html = enhance(&analysed(true), "http://example.org/", Some(Mode::Colorize));

        assert!(
            html.contains(
                "<p>Mun oidnen <span class=\"teaksta-token\" id=\"teaksta-span-1\">viesu</span>.</p>"
            ),
            "{}",
            html
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn+5/test]
    #[test]
    fn an_irrelevant_span_reaches_click_alone() {
        let cas = analysed(false);

        assert!(
            !enhance(&cas, "http://example.org/", Some(Mode::Colorize)).contains("teaksta-span-1")
        );
        assert!(enhance(&cas, "http://example.org/", Some(Mode::Click)).contains("teaksta-span-1"));
        assert!(!enhance(&cas, "http://example.org/", None).contains("teaksta-span-1"));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn+5/test]
    #[test]
    fn the_base_url_is_escaped_as_an_attribute() {
        let html = enhance(
            &analysed(true),
            "http://example.org/?a=\"1\"&b=2",
            Some(Mode::Mc),
        );

        assert!(
            html.contains("<base href=\"http://example.org/?a=&quot;1&quot;&amp;b=2\">"),
            "{}",
            html
        );
    }

    #[test]
    fn north_sami_labels_are_available_to_callers() {
        assert_eq!(sami_label("Substantive"), Some("Substantiivvat"));
        assert_eq!(sami_label("Unknown"), None);
        assert_eq!(
            topic_title("Substantive", Some(Mode::Colorize.name())),
            "Substantiivvat: Geah\u{10d}a ivdnejuvvon s\u{e1}niid."
        );
        assert_eq!(topic_title("Unknown", None), "Unknown");
    }
}
