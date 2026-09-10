//! JSONEnhancer produces a JSON array containing each enhanced span from a CAS
//! containing a sequence of `<e>` spans and Enhancements.
//!
//! Author: Adriane Boyd

use std::collections::HashMap;
use std::sync::LazyLock;

use anyhow::Result;
use regex::Regex;

use crate::types::Document;
use crate::util::enhancer_utils;

/// regex to match the <e> enhance spans
static ENHANCE_PATT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)<e ([^>]*)>(.*?)</e>").expect("enhance-span pattern"));
static COUNTER_PATT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"id="(\d+)""#).expect("counter pattern"));

// [spec:teaksta:def:sme.src.main.java.werti.util.json-enhancer.json-enhancer]
pub struct JsonEnhancer<'a> {
    cas: &'a Document,
    activity: &'a str,
}

impl<'a> JsonEnhancer<'a> {
    // [spec:teaksta:def:sme.src.main.java.werti.util.json-enhancer.json-enhancer.json-enhancer-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.json-enhancer.json-enhancer.json-enhancer-fn]
    pub fn new(c_cas: &'a Document, a_activity: &'a str) -> Self {
        JsonEnhancer {
            cas: c_cas,
            activity: a_activity,
        }
    }

    /// Converts a CAS with Enhancements to an array of enhanced spans
    /// in JSON format.
    // [spec:teaksta:def:sme.src.main.java.werti.util.json-enhancer.json-enhancer.enhance-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.json-enhancer.json-enhancer.enhance-fn]
    pub fn enhance(&self) -> Result<String> {
        let enhanced = enhancer_utils::cas_to_enhanced(self.cas, Some(self.activity))?;
        let enhanced = self.enhanced_to_json(&enhanced)?;

        Ok(enhanced)
    }

    /// Converts a string containing a sequence of `<e></e>` enhanced spans into
    /// a JSON array where each array element is wrapped with a `<span>` that
    /// minimizes layout changes.
    // [spec:teaksta:def:sme.src.main.java.werti.util.json-enhancer.json-enhancer.enhanced-to-json-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.json-enhancer.json-enhancer.enhanced-to-json-fn]
    fn enhanced_to_json(&self, enhanced: &str) -> Result<String> {
        // The EnhanceXML annotations are no longer useful at this point
        // because the enhancements have already been inserted.

        let mut new_nodes: HashMap<i32, String> = HashMap::new();

        for enhance_match in ENHANCE_PATT.captures_iter(enhanced) {
            // find index
            let attributes = enhance_match.get(1).map_or("", |group| group.as_str());
            let content = enhance_match.get(2).map_or("", |group| group.as_str());

            let mut counter = 0;
            if let Some(counter_match) = COUNTER_PATT.captures(attributes) {
                counter = counter_match[1].parse::<i32>()?;
            }
            // wrap outer span around each group; add_span_style runs once
            // per group
            new_nodes.insert(
                counter,
                format!(
                    "<span class=\"wertiview\" style=\"{}\">{}</span>",
                    enhancer_utils::ADDED_SPAN_STYLE,
                    content
                ),
            );
        }

        // convert the list of new nodes to JSON and return
        Ok(serde_json::to_string(&new_nodes)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Enhancement;

    fn wrapped(content: &str) -> String {
        format!(
            "<span class=\"wertiview\" style=\"{}\">{}</span>",
            enhancer_utils::ADDED_SPAN_STYLE,
            content
        )
    }

    fn spans(json: &str) -> HashMap<String, String> {
        serde_json::from_str(json).expect("JSON object")
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.json-enhancer.json-enhancer.json-enhancer-fn/test]
    #[test]
    fn the_constructor_stores_the_cas_and_activity_unchanged() {
        let cas = Document::new("beana", "sme");

        let enhancer = JsonEnhancer::new(&cas, "click");

        assert!(std::ptr::eq(enhancer.cas, &cas));
        assert_eq!(enhancer.activity, "click");
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.json-enhancer.json-enhancer.enhanced-to-json-fn/test]
    #[test]
    fn enhance_spans_become_wertiview_spans_keyed_by_id() {
        let cas = Document::new("", "sme");
        let enhancer = JsonEnhancer::new(&cas, "click");

        let json = enhancer
            .enhanced_to_json("<p><e id=\"3\" class=\"x\">boaris</e></p>")
            .unwrap();

        assert_eq!(
            spans(&json),
            HashMap::from([("3".to_string(), wrapped("boaris"))])
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.json-enhancer.json-enhancer.enhanced-to-json-fn/test]
    #[test]
    fn an_attribute_less_enhance_span_never_matches() {
        let cas = Document::new("", "sme");
        let enhancer = JsonEnhancer::new(&cas, "click");

        let json = enhancer.enhanced_to_json("<e>boaris</e>").unwrap();

        assert_eq!(json, "{}");
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.json-enhancer.json-enhancer.enhanced-to-json-fn/test]
    #[test]
    fn spans_without_an_id_collapse_onto_key_zero() {
        let cas = Document::new("", "sme");
        let enhancer = JsonEnhancer::new(&cas, "click");

        let json = enhancer
            .enhanced_to_json("<e class=\"a\">first</e><e class=\"b\">second</e>")
            .unwrap();

        assert_eq!(
            spans(&json),
            HashMap::from([("0".to_string(), wrapped("second"))])
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.json-enhancer.json-enhancer.enhanced-to-json-fn/test]
    #[test]
    fn nested_span_content_stops_at_the_first_close() {
        let cas = Document::new("", "sme");
        let enhancer = JsonEnhancer::new(&cas, "click");

        let json = enhancer
            .enhanced_to_json("<e id=\"1\" >outer <e id=\"2\" >inner</e> tail</e>")
            .unwrap();

        assert_eq!(
            spans(&json),
            HashMap::from([("1".to_string(), wrapped("outer <e id=\"2\" >inner"))])
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.json-enhancer.json-enhancer.enhanced-to-json-fn/test]
    #[test]
    fn span_content_spans_newlines_and_is_copied_verbatim() {
        let cas = Document::new("", "sme");
        let enhancer = JsonEnhancer::new(&cas, "click");

        let json = enhancer
            .enhanced_to_json("<e id=\"9\">a &amp; <b>b</b>\nc</e>")
            .unwrap();

        assert_eq!(
            spans(&json),
            HashMap::from([("9".to_string(), wrapped("a &amp; <b>b</b>\nc"))])
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.json-enhancer.json-enhancer.enhance-fn/test]
    #[test]
    fn enhance_splices_cas_enhancements_and_extracts_json() {
        let mut cas = Document::new("<p>boaris beana</p>", "sme");
        cas.enhancements.push(Enhancement {
            begin: 3,
            end: 9,
            enhance_start: "<e id=\"1\" class=\"wertiviewtoken \">".to_string(),
            enhance_end: "</e>".to_string(),
            relevant: true,
        });
        cas.enhancements.push(Enhancement {
            begin: 10,
            end: 15,
            enhance_start: "<e id=\"2\" class=\"wertiviewtoken \">".to_string(),
            enhance_end: "</e>".to_string(),
            relevant: true,
        });
        let enhancer = JsonEnhancer::new(&cas, "click");

        let json = enhancer.enhance().unwrap();

        assert_eq!(
            spans(&json),
            HashMap::from([
                ("1".to_string(), wrapped("boaris")),
                ("2".to_string(), wrapped("beana")),
            ])
        );
    }
}
