//! Annotate all `<e>` tags.

use anyhow::Result;
use regex::Regex;
use tracing::debug;

use crate::types::{Document, EnhanceXml};

// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.enhance-xml-annotator.enhance-xml-annotator]
#[derive(Debug, Clone, Copy, Default)]
pub struct EnhanceXmlAnnotator;

impl EnhanceXmlAnnotator {
    pub fn new() -> Self {
        EnhanceXmlAnnotator
    }

    /// Mark up all werti spans.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.enhance-xml-annotator.enhance-xml-annotator.process-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.enhance-xml-annotator.enhance-xml-annotator.process-fn]
    pub fn process(&self, cas: &mut Document) -> Result<()> {
        debug!("Starting markup recognition");

        // Matching runs over the lower-cased copy while the offsets recorded
        // are offsets into the original document text, so any character whose
        // lower-case form has a different length shifts every later offset.
        let s = cas.text.to_lowercase();

        // regex to match the <e> enhance spans
        let enhance_patt = Regex::new(r"(?s)<e( [^>]*)?>(.*?)</e>")?;

        for enhance_matcher in enhance_patt.captures_iter(&s) {
            let (Some(whole), Some(group2)) = (enhance_matcher.get(0), enhance_matcher.get(2))
            else {
                continue;
            };

            // create tag for enhance start tag
            let starttag = EnhanceXml {
                begin: whole.start(),
                end: group2.start(),
                tag_name: "spanwertiview".to_string(),
                closing: false,
                irrelevant: false,
            };
            cas.enhance_xml.push(starttag);

            // create tag for enhance end tag
            let endtag = EnhanceXml {
                begin: group2.end(),
                end: whole.end(),
                tag_name: "spanwertiview".to_string(),
                closing: true,
                irrelevant: false,
            };
            cas.enhance_xml.push(endtag);
        }

        debug!("Finished markup recognition");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn annotate(text: &str) -> Vec<EnhanceXml> {
        let mut doc = Document::new(text, "sme");
        EnhanceXmlAnnotator::new().process(&mut doc).unwrap();
        doc.enhance_xml
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.enhance-xml-annotator.enhance-xml-annotator.process-fn/test]
    #[test]
    fn process_brackets_span_with_opening_and_closing_tags() {
        let tags = annotate("<p>a <e>fisk</e> b</p>");

        assert_eq!(tags.len(), 2);
        assert_eq!((tags[0].begin, tags[0].end), (5, 8));
        assert_eq!(tags[0].tag_name, "spanwertiview");
        assert!(!tags[0].closing);
        assert!(!tags[0].irrelevant);
        assert_eq!((tags[1].begin, tags[1].end), (12, 16));
        assert_eq!(tags[1].tag_name, "spanwertiview");
        assert!(tags[1].closing);
        assert!(!tags[1].irrelevant);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.enhance-xml-annotator.enhance-xml-annotator.process-fn/test]
    #[test]
    fn process_covers_opening_tag_including_attributes() {
        let tags = annotate("<e class=\"x\">y</e>");

        assert_eq!((tags[0].begin, tags[0].end), (0, 13));
        assert_eq!((tags[1].begin, tags[1].end), (14, 18));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.enhance-xml-annotator.enhance-xml-annotator.process-fn/test]
    #[test]
    fn process_matches_upper_case_tags_via_lowercased_copy() {
        let tags = annotate("<E>Guolli</E>");

        assert_eq!(tags.len(), 2);
        assert_eq!((tags[0].begin, tags[0].end), (0, 3));
        assert_eq!((tags[1].begin, tags[1].end), (9, 13));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.enhance-xml-annotator.enhance-xml-annotator.process-fn/test]
    #[test]
    fn process_spans_newlines_inside_the_inner_content() {
        let tags = annotate("<e>a\nb</e>");

        assert_eq!(tags.len(), 2);
        assert_eq!((tags[0].begin, tags[0].end), (0, 3));
        assert_eq!((tags[1].begin, tags[1].end), (6, 10));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.enhance-xml-annotator.enhance-xml-annotator.process-fn/test]
    #[test]
    fn process_stops_inner_content_at_first_closing_tag() {
        let text = "<e>a<e>b</e>c</e>";
        let tags = annotate(text);

        assert_eq!(tags.len(), 2);
        assert_eq!((tags[0].begin, tags[0].end), (0, 3));
        assert_eq!((tags[1].begin, tags[1].end), (8, 12));
        assert_eq!(&text[tags[0].end..tags[1].begin], "a<e>b");
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.enhance-xml-annotator.enhance-xml-annotator.process-fn/test]
    #[test]
    fn process_records_offsets_skewed_by_lowercase_mapping() {
        let text = "\u{130}<e>x</e>";
        assert_eq!(text.len(), 10);
        assert_eq!(text.to_lowercase().len(), 11);

        let tags = annotate(text);

        assert_eq!((tags[0].begin, tags[0].end), (3, 6));
        assert_eq!(&text[tags[0].begin..tags[0].end], "e>x");
        assert_eq!((tags[1].begin, tags[1].end), (7, 11));
        assert!(tags[1].end > text.len());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.enhance-xml-annotator.enhance-xml-annotator.process-fn/test]
    #[test]
    fn process_leaves_text_and_other_stores_untouched() {
        let mut doc = Document::new("<p>no enhance tags here</p>", "sme");
        doc.tokens.push(crate::types::Token::default());

        EnhanceXmlAnnotator::new().process(&mut doc).unwrap();

        assert!(doc.enhance_xml.is_empty());
        assert_eq!(doc.text, "<p>no enhance tags here</p>");
        assert_eq!(doc.tokens.len(), 1);
        assert!(doc.relevant_texts.is_empty());
        assert!(doc.enhancements.is_empty());
    }
}
