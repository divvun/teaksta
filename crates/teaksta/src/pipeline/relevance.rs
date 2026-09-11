//! Generic relevance annotator.
//!
//! Turns the fetched page the document still holds as markup into the text
//! that is worth analysing, and records which stretches of that text the
//! page's own elements contributed.

use anyhow::Result;
use tracing::debug;

use crate::types::Document;
use crate::util::html_utils;

// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.generic-relevance-annotator.generic-relevance-annotator]
#[derive(Debug, Clone, Copy, Default)]
pub struct GenericRelevanceAnnotator;

impl GenericRelevanceAnnotator {
    pub fn new() -> Self {
        GenericRelevanceAnnotator
    }

    /// Replaces the document text with the page's analysable text and marks
    /// every stretch of it that came from one of the page's text nodes as
    /// relevant, keeping the map back to the page for the enhanced output.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.ae.generic-relevance-annotator.generic-relevance-annotator.process-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.generic-relevance-annotator.generic-relevance-annotator.process-fn+2]
    pub fn process(&self, doc: &mut Document) -> Result<()> {
        debug!("Starting relevance annotation");

        let (seed, map) = html_utils::extract(&doc.text);
        doc.text = seed.text;
        doc.relevant_texts.extend(seed.relevant_texts);
        doc.page = map;

        debug!("Finished relevance annotation");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RelevantText;

    const PAGE: &str = concat!(
        "<html><head><title>t</title></head>",
        "<body><p>Mun oidnen viesu.</p><p>Viesut leat stuorr\u{e1}t.</p></body></html>"
    );

    fn annotated(html: &str) -> Document {
        let mut doc = Document::new(html);
        GenericRelevanceAnnotator::new().process(&mut doc).unwrap();
        doc
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.generic-relevance-annotator.generic-relevance-annotator.process-fn+2/test]
    #[test]
    fn process_replaces_markup_with_analysable_text() {
        let doc = annotated(PAGE);

        assert_eq!(doc.text, "Mun oidnen viesu.\nViesut leat stuorr\u{e1}t.");
        assert_eq!(doc.page.html, PAGE);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.generic-relevance-annotator.generic-relevance-annotator.process-fn+2/test]
    #[test]
    fn process_indexes_one_span_per_text_node() {
        let doc = annotated(PAGE);

        assert_eq!(doc.relevant_texts.len(), 2);
        assert_eq!(
            (doc.relevant_texts[0].begin, doc.relevant_texts[0].end),
            (0, 17)
        );
        assert_eq!(
            (doc.relevant_texts[1].begin, doc.relevant_texts[1].end),
            (18, 40)
        );
        assert!(doc.relevant_texts.iter().all(|span| span.relevant));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.generic-relevance-annotator.generic-relevance-annotator.process-fn+2/test]
    #[test]
    fn process_maps_every_span_onto_a_node() {
        let doc = annotated(PAGE);

        assert_eq!(doc.page.segments.len(), doc.relevant_texts.len());
        for (segment, span) in doc.page.segments.iter().zip(&doc.relevant_texts) {
            assert_eq!((segment.begin, segment.end), (span.begin, span.end));
            assert_eq!(segment.block_start, span.block_start);
        }
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.generic-relevance-annotator.generic-relevance-annotator.process-fn+2/test]
    #[test]
    fn process_produces_nothing_for_a_page_without_prose() {
        let doc = annotated("<html><head><title>t</title></head><body>  </body></html>");

        assert!(doc.text.is_empty());
        assert!(doc.relevant_texts.is_empty());
        assert!(doc.page.segments.is_empty());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.generic-relevance-annotator.generic-relevance-annotator.process-fn+2/test]
    #[test]
    fn process_appends_to_the_existing_relevant_text_store() {
        let mut doc = Document::new(PAGE);
        doc.relevant_texts.push(RelevantText {
            begin: 100,
            end: 200,
            ..Default::default()
        });

        GenericRelevanceAnnotator::new().process(&mut doc).unwrap();

        assert_eq!(doc.relevant_texts.len(), 3);
        assert_eq!(
            (doc.relevant_texts[0].begin, doc.relevant_texts[0].end),
            (100, 200)
        );
    }
}
