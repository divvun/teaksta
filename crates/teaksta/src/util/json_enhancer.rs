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
