//! Produces an HTML document with enhancements from a document carrying
//! Enhancements and the page they were found in.
//!
//! Author: Adriane Boyd
//!
//! The page is rendered from the document's own map of it, so the only thing
//! left to add here is the base URL the fetched page's relative links need.

use crate::server::api::Mode;
use crate::types::Document;
use crate::util::html_utils;

/// What the learner is asked to do, in North Sámi, for the registry endpoint
/// to serve alongside each exercise.
///
/// The exercises are four and are compiled in, so every one of them has a
/// label and the lookup cannot miss. A topic's label is not here: topics are
/// configuration, and each one carries its own in `topics.toml`.
pub fn mode_label(mode: Mode) -> &'static str {
    match mode {
        Mode::Colorize => "Geah\u{10d}a ivdnejuvvon s\u{e1}niid.",
        Mode::Click => "Coahkkal rivttes s\u{e1}niid!",
        Mode::Mc => "V\u{e1}llje rivttes s\u{e1}niid!",
        Mode::Cloze => "\u{10c}\u{e1}le rivttes s\u{e1}niid!",
    }
}

// [spec:teaksta:def:sme.src.main.java.werti.util.html-enhancer.html-enhancer]
pub struct HtmlEnhancer<'a> {
    doc: &'a Document,
}

impl<'a> HtmlEnhancer<'a> {
    // [spec:teaksta:def:sme.src.main.java.werti.util.html-enhancer.html-enhancer.html-enhancer-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.html-enhancer-fn]
    pub fn new(a_document: &'a Document) -> Self {
        HtmlEnhancer { doc: a_document }
    }

    /// Converts an HTML document with Enhancements to an HTML string. The
    /// topic reaches the page through the client rather than through the
    /// markup, so only the requested exercise is read here.
    // [spec:teaksta:def:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn+6]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn+6]
    pub fn enhance(&self, mode: Option<Mode>, base_url: &str) -> String {
        html_utils::render_page(&self.doc.page, self.doc, mode, Some(base_url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::types::Enhancement;
    use crate::util::html_utils;

    const PAGE: &str =
        "<html><head><title>Old</title></head><body><p>Mun oidnen viesu.</p></body></html>";

    /// A document seeded from `PAGE`, carrying one enhancement over `viesu`.
    fn analysed(relevant: bool) -> Document {
        let (mut doc, map) = html_utils::extract(PAGE);
        doc.page = map;
        doc.enhancements.push(Enhancement {
            begin: 11,
            end: 16,
            enhance_start: "<span id=\"teaksta-span-1\" class=\"teaksta-token\">".to_string(),
            enhance_end: "</span>".to_string(),
            relevant,
        });
        doc
    }

    fn enhance(doc: &Document, base_url: &str, mode: Option<Mode>) -> String {
        HtmlEnhancer::new(doc).enhance(mode, base_url)
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.html-enhancer-fn/test]
    #[test]
    fn the_constructor_stores_the_document_by_reference() {
        let doc = Document::new(PAGE);

        let enhancer = HtmlEnhancer::new(&doc);

        assert!(std::ptr::eq(enhancer.doc, &doc));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn+6/test]
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

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn+6/test]
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

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn+6/test]
    #[test]
    fn an_irrelevant_span_reaches_click_alone() {
        let doc = analysed(false);

        assert!(
            !enhance(&doc, "http://example.org/", Some(Mode::Colorize)).contains("teaksta-span-1")
        );
        assert!(enhance(&doc, "http://example.org/", Some(Mode::Click)).contains("teaksta-span-1"));
        assert!(!enhance(&doc, "http://example.org/", None).contains("teaksta-span-1"));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn+6/test]
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
    fn every_exercise_carries_a_north_sami_label() {
        assert_eq!(
            mode_label(Mode::Colorize),
            "Geah\u{10d}a ivdnejuvvon s\u{e1}niid."
        );
        for mode in Mode::ALL {
            assert!(!mode_label(mode).is_empty(), "{}", mode.name());
        }
    }
}
