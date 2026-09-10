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

#[cfg(test)]
mod tests {
    use super::*;

    fn tag(begin: usize, end: usize, closing: bool) -> EnhanceXml {
        EnhanceXml {
            begin,
            end,
            tag_name: "spanwertiview".to_string(),
            closing,
            irrelevant: false,
        }
    }

    fn relevant_texts(tags: Vec<EnhanceXml>) -> Vec<RelevantText> {
        let mut doc = Document::new("", "sme");
        doc.enhance_xml = tags;
        GenericRelevanceAnnotator::new().process(&mut doc).unwrap();
        doc.relevant_texts
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.generic-relevance-annotator.generic-relevance-annotator.process-fn/test]
    #[test]
    fn process_indexes_the_inner_content_of_one_span() {
        let spans = relevant_texts(vec![tag(5, 8, false), tag(12, 16, true)]);

        assert_eq!(spans.len(), 1);
        assert_eq!((spans[0].begin, spans[0].end), (8, 12));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.generic-relevance-annotator.generic-relevance-annotator.process-fn/test]
    #[test]
    fn process_drops_the_span_between_two_enhance_spans() {
        let spans = relevant_texts(vec![
            tag(0, 3, false),
            tag(7, 11, true),
            tag(20, 23, false),
            tag(27, 31, true),
        ]);

        assert_eq!(spans.len(), 2);
        assert_eq!((spans[0].begin, spans[0].end), (3, 7));
        assert_eq!((spans[1].begin, spans[1].end), (23, 27));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.generic-relevance-annotator.generic-relevance-annotator.process-fn/test]
    #[test]
    fn process_leaves_features_beyond_offsets_at_default() {
        let spans = relevant_texts(vec![tag(0, 3, false), tag(7, 11, true)]);

        assert!(!spans[0].relevant);
        assert_eq!(spans[0].html_content_type, None);
        assert_eq!(spans[0].enclosing_tag, None);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.generic-relevance-annotator.generic-relevance-annotator.process-fn/test]
    #[test]
    fn process_produces_nothing_without_a_pair_of_tags() {
        assert!(relevant_texts(Vec::new()).is_empty());
        assert!(relevant_texts(vec![tag(0, 3, false)]).is_empty());
        assert!(relevant_texts(vec![tag(0, 3, true)]).is_empty());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.generic-relevance-annotator.generic-relevance-annotator.process-fn/test]
    #[test]
    fn process_walks_tags_in_index_not_insertion_order() {
        let spans = relevant_texts(vec![tag(12, 16, true), tag(5, 8, false)]);

        assert_eq!(spans.len(), 1);
        assert_eq!((spans[0].begin, spans[0].end), (8, 12));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.generic-relevance-annotator.generic-relevance-annotator.process-fn/test]
    #[test]
    fn process_orders_equal_begins_by_descending_end() {
        let spans = relevant_texts(vec![tag(0, 4, true), tag(0, 9, true)]);

        assert_eq!(spans.len(), 1);
        assert_eq!((spans[0].begin, spans[0].end), (9, 0));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.generic-relevance-annotator.generic-relevance-annotator.process-fn/test]
    #[test]
    fn process_appends_to_the_existing_relevant_text_store() {
        let mut doc = Document::new("", "sme");
        doc.relevant_texts.push(RelevantText {
            begin: 100,
            end: 200,
            ..Default::default()
        });
        doc.enhance_xml = vec![tag(0, 3, false), tag(7, 11, true)];

        GenericRelevanceAnnotator::new().process(&mut doc).unwrap();

        assert_eq!(doc.relevant_texts.len(), 2);
        assert_eq!(
            (doc.relevant_texts[0].begin, doc.relevant_texts[0].end),
            (100, 200)
        );
        assert_eq!(
            (doc.relevant_texts[1].begin, doc.relevant_texts[1].end),
            (3, 7)
        );
    }
}
