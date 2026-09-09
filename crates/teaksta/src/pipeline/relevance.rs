//! Generic relevance annotator.
//!
//! Marks all parts of the document that aren't between EnhanceXML (`<e>`)
//! tags as irrelevant.

use anyhow::Result;
use tracing::debug;

use crate::types::{Document, EnhanceXml, RelevantText};

/// Annotation index order: ascending `begin`, then descending `end`.
fn index_order(tags: &[EnhanceXml]) -> Vec<&EnhanceXml> {
    let mut ordered: Vec<&EnhanceXml> = tags.iter().collect();
    ordered.sort_by(|a, b| a.begin.cmp(&b.begin).then(b.end.cmp(&a.end)));
    ordered
}

// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.generic-relevance-annotator.generic-relevance-annotator]
#[derive(Debug, Clone, Copy, Default)]
pub struct GenericRelevanceAnnotator;

impl GenericRelevanceAnnotator {
    pub fn new() -> Self {
        GenericRelevanceAnnotator
    }

    /// Marks all parts of the document annotated as inside EnhanceXML tags by
    /// [`crate::pipeline::enhance_xml::EnhanceXmlAnnotator`] as relevant.
    ///
    /// The `"No EnhanceXML tags were found!"` failure path of the original is
    /// unreachable: an index iterator that reports a next element never
    /// yields an absent one.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.generic-relevance-annotator.generic-relevance-annotator.process-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.generic-relevance-annotator.generic-relevance-annotator.process-fn]
    pub fn process(&self, cas: &mut Document) -> Result<()> {
        debug!("Starting relevance annotation");
        let tag_index = index_order(&cas.enhance_xml);
        let mut tit = tag_index.into_iter();

        let Some(first) = tit.next() else {
            return Ok(());
        };

        let mut tag = first;

        // Collected first because the walk borrows the tag store; the
        // original added each one to the indexes as it went.
        let mut indexed: Vec<RelevantText> = Vec::new();
        for next in tit {
            let mut rt = RelevantText::default();

            // get type/end info from the current tag
            rt.begin = tag.end;

            // move to the next tag
            tag = next;

            // get begin info from the next tag
            rt.end = tag.begin;

            // An opening tag leaves the constructed span unindexed, so only
            // the inner content of each `<e>...</e>` span is retrievable.
            if tag.closing {
                indexed.push(rt);
            }
        }

        cas.relevant_texts.extend(indexed);

        debug!("Finished relevance annotation");
        Ok(())
    }
}
