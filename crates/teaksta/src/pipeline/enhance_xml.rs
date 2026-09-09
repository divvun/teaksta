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
