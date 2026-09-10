//! JSONEnhancer produces a JSON object holding each enhanced span of a
//! document, for a client that already has the page and only wants the
//! fragments that changed.
//!
//! Author: Adriane Boyd

use anyhow::Result;

use crate::server::api::Mode;
use crate::types::Document;
use crate::util::html_utils;

// [spec:teaksta:def:sme.src.main.java.werti.util.json-enhancer.json-enhancer+1]
pub struct JsonEnhancer<'a> {
    cas: &'a Document,
    mode: Mode,
}

impl<'a> JsonEnhancer<'a> {
    // [spec:teaksta:def:sme.src.main.java.werti.util.json-enhancer.json-enhancer.json-enhancer-fn+1]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.json-enhancer.json-enhancer.json-enhancer-fn+1]
    pub fn new(c_cas: &'a Document, a_mode: Mode) -> Self {
        JsonEnhancer {
            cas: c_cas,
            mode: a_mode,
        }
    }

    /// Converts a document with Enhancements to a JSON object of enhanced
    /// spans, keyed by the position in the document text each one covers.
    // [spec:teaksta:def:sme.src.main.java.werti.util.json-enhancer.json-enhancer.enhance-fn+3]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.json-enhancer.json-enhancer.enhance-fn+3]
    pub fn enhance(&self) -> Result<String> {
        let spans = html_utils::render_spans(&self.cas.page, self.cas, Some(self.mode))?;

        Ok(serde_json::to_string(&spans)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PIPELINE_LANGUAGE;
    use std::collections::HashMap;

    use crate::types::Enhancement;
    use crate::util::enhancer_utils::{ADDED_SPAN_STYLE, PAGE_SPAN_CLASS};

    const PAGE: &str = concat!(
        "<html><head><title>t</title></head>",
        "<body><p>Mun oidnen viesu.</p><p>Viesut leat stuorr\u{e1}t.</p></body></html>"
    );

    fn enhancement(begin: usize, end: usize, id: &str, relevant: bool) -> Enhancement {
        Enhancement {
            begin,
            end,
            enhance_start: format!("<span id=\"{}\" class=\"teaksta-token\">", id),
            enhance_end: "</span>".to_string(),
            relevant,
        }
    }

    /// A document seeded from `PAGE`, carrying the given enhancements.
    fn analysed(enhancements: Vec<Enhancement>) -> Document {
        let (mut cas, map) = html_utils::extract(PAGE);
        cas.page = map;
        cas.enhancements = enhancements;
        cas
    }

    fn spans(json: &str) -> HashMap<String, String> {
        serde_json::from_str(json).expect("JSON object")
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.json-enhancer.json-enhancer.json-enhancer-fn+1/test]
    #[test]
    fn the_constructor_stores_the_cas_and_exercise_unchanged() {
        let cas = Document::new("beana", PIPELINE_LANGUAGE);

        let enhancer = JsonEnhancer::new(&cas, Mode::Click);

        assert!(std::ptr::eq(enhancer.cas, &cas));
        assert_eq!(enhancer.mode, Mode::Click);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.json-enhancer.json-enhancer.enhance-fn+3/test]
    #[test]
    fn each_enhancement_is_keyed_by_its_position() {
        let cas = analysed(vec![
            enhancement(11, 16, "teaksta-span-1", true),
            enhancement(18, 24, "teaksta-span-2", true),
        ]);

        let json = JsonEnhancer::new(&cas, Mode::Colorize).enhance().unwrap();

        assert_eq!(
            spans(&json),
            HashMap::from([
                (
                    "11".to_string(),
                    format!(
                        "<span class=\"{}\" style=\"{}\"><span class=\"teaksta-token\" id=\"teaksta-span-1\">viesu</span></span>",
                        PAGE_SPAN_CLASS, ADDED_SPAN_STYLE
                    )
                ),
                (
                    "18".to_string(),
                    format!(
                        "<span class=\"{}\" style=\"{}\"><span class=\"teaksta-token\" id=\"teaksta-span-2\">Viesut</span></span>",
                        PAGE_SPAN_CLASS, ADDED_SPAN_STYLE
                    )
                ),
            ])
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.json-enhancer.json-enhancer.enhance-fn+3/test]
    #[test]
    fn text_the_client_gets_back_is_escaped() {
        let (mut cas, map) =
            html_utils::extract("<html><body><p>Tom &amp; Jerry</p></body></html>");
        cas.page = map;
        cas.enhancements
            .push(enhancement(0, 11, "teaksta-span-1", true));

        let json = JsonEnhancer::new(&cas, Mode::Colorize).enhance().unwrap();

        assert!(
            spans(&json)["0"].contains(">Tom &amp; Jerry</span>"),
            "{json}"
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.json-enhancer.json-enhancer.enhance-fn+3/test]
    #[test]
    fn irrelevant_spans_are_served_to_click_alone() {
        let cas = analysed(vec![enhancement(11, 16, "teaksta-span-1", false)]);

        assert_eq!(
            JsonEnhancer::new(&cas, Mode::Colorize).enhance().unwrap(),
            "{}"
        );
        assert_eq!(
            spans(&JsonEnhancer::new(&cas, Mode::Click).enhance().unwrap()).len(),
            1
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.json-enhancer.json-enhancer.enhance-fn+3/test]
    #[test]
    fn a_document_without_enhancements_yields_no_spans() {
        let cas = analysed(Vec::new());

        assert_eq!(
            JsonEnhancer::new(&cas, Mode::Click).enhance().unwrap(),
            "{}"
        );
    }
}
