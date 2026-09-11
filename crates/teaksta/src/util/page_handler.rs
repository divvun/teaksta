//! Running one topic's pipeline pair over a page, with the analysed document
//! cached on disk under a caller-chosen key.
//!
//! Author: Adriane Boyd
//!
//! The flow pair a topic runs is handed out by
//! [`crate::server::registry::Registry`]; what the pair produces is a
//! [`crate::types::Document`], which the cache holds as JSON under the key
//! the caller chose.

use std::path::{Path, PathBuf};

use anyhow::Result;
use tracing::{error, info, warn};

use crate::pipeline::flow::Flow;
use crate::server::api::Mode;
use crate::server::registry::Registry;
use crate::types::Document;

/// A pipeline run that did not finish, carrying the reason the stage that
/// broke gave. It reaches the caller as the request's analysis failure.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct AnalysisFailed(String);

/// Runs a flow's stages over the document for the requested exercise.
fn run_flow(
    flow: &Flow,
    doc: &mut Document,
    mode: Mode,
) -> std::result::Result<(), AnalysisFailed> {
    flow.run(doc, mode)
        .map_err(|e| AnalysisFailed(format!("{e:#}")))
}

/// The cached document, decoded from the JSON an earlier run wrote. A file
/// written by a build whose document model differs will not decode and is
/// unreadable, which is the case the caller already handles.
///
/// The file is the one input to the pipeline nothing in this process wrote,
/// so its offsets are checked against the text it carries before it is handed
/// on: a span the text cannot be read at makes the file unreadable, on the
/// same footing as one that will not decode at all.
fn read_cached(path: &Path) -> std::io::Result<Document> {
    let encoded = std::fs::read_to_string(path)?;
    let doc: Document = serde_json::from_str(&encoded)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    if let Some(span) = doc.invalid_span() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            span.to_string(),
        ));
    }
    Ok(doc)
}

/// Writes an analysed document to the cache. The counterpart of
/// [`read_cached`].
fn write_cached(doc: &Document, path: &Path) -> std::io::Result<()> {
    let encoded = serde_json::to_string(doc)?;
    std::fs::write(path, encoded)
}

/// One request's analysis: the page, where to cache it and which pipeline
/// pair to run over it.
///
/// Everything but the exercise is borrowed for the life of the handler, which
/// is built, used and dropped inside the call that assembled its arguments —
/// so a copy of the page would be a copy of the whole page for nothing.
// [spec:teaksta:def:sme.src.main.java.werti.util.page-handler.page-handler]
pub struct PageHandler<'a> {
    registry: &'a Registry,
    topic: &'a str,
    text: &'a str,
    url: &'a str,
    path: &'a str,
    /// The exercise the request asked for, carried to the postprocessing
    /// enhancers that decide what to attach to a token from it.
    mode: Mode,
}

impl<'a> PageHandler<'a> {
    // [spec:teaksta:def:sme.src.main.java.werti.util.page-handler.page-handler.page-handler-fn+4]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.page-handler.page-handler.page-handler-fn+4]
    pub fn new(
        a_registry: &'a Registry,
        a_topic: &'a str,
        a_url: &'a str,
        a_path: &'a str,
        a_text: &'a str,
        a_mode: Mode,
    ) -> Self {
        // The assignment order differs from the parameter order: `url` and
        // `path` are the third and fourth arguments but the fourth and fifth
        // assignments.
        PageHandler {
            registry: a_registry,
            topic: a_topic,
            text: a_text,
            url: a_url,
            path: a_path,
            mode: a_mode,
        }
    }

    /// Builds a document from the text and runs the pre- and postprocessors
    /// for the topic.
    // [spec:teaksta:def:sme.src.main.java.werti.util.page-handler.page-handler.process-fn+7]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.page-handler.page-handler.process-fn+7]
    pub fn process(&self) -> Result<Option<Document>> {
        let preprocessor = self.registry.get_preprocessor(self.topic);
        let postprocessor = self.registry.get_postprocessor(self.topic);
        let (Some(preprocessor), Some(postprocessor)) = (preprocessor, postprocessor) else {
            return Ok(None);
        };

        match self.analysed(preprocessor, postprocessor) {
            Ok(doc) => Ok(Some(doc)),
            Err(failed) => {
                error!("The analysis pipeline failed! {}", failed);
                Err(anyhow::Error::new(failed).context("Text analysis failed."))
            }
        }
    }

    /// Everything a failing stage can break out of: building the document,
    /// reading or writing its cache, and the two pipeline runs.
    fn analysed(
        &self,
        preprocessor: &Flow,
        postprocessor: &Flow,
    ) -> std::result::Result<Document, AnalysisFailed> {
        // convert HTML entities to characters, if there are any
        let mut doc = Document::new(html_escape::decode_html_entities(self.text).into_owned());

        let cache_dir = PathBuf::from(self.path);
        if !cache_dir.exists()
            && let Err(e) = std::fs::create_dir_all(&cache_dir)
        {
            // The cache is an optimisation, so a directory that cannot be
            // made costs this request nothing beyond its cache — but it
            // costs every later one the same, which is a deployment fault
            // worth naming rather than discarding.
            warn!(
                "Failed to create the cache directory {}! {}",
                cache_dir.display(),
                e
            );
        }
        let cache_file = cache_dir.join(format!("{}.json", self.url));
        let cached = match cache_file.is_file() {
            true => read_cached(&cache_file)
                .inspect_err(|failed| {
                    info!("Failed to load the analysis from file! {}", failed);
                })
                .ok(),
            false => None,
        };

        match cached {
            // The cached document is the preprocessor's output, so the
            // preprocessor is not run over it again.
            Some(cached) => doc = cached,
            // A file that could not be read is rewritten from this run's
            // analysis, so a cache the deployment cannot decode costs one
            // request rather than every later one.
            None => {
                run_flow(preprocessor, &mut doc, self.mode)?;
                if let Err(failed) = write_cached(&doc, &cache_file) {
                    info!("Failed to write the analysis to file! {}", failed);
                }
            }
        }

        // Whichever branch produced the document, the request is answered
        // from the postprocessor's output: a cache that cannot be read or
        // written costs the request its cache, not its enhancement.
        run_flow(postprocessor, &mut doc, self.mode)?;
        Ok(doc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::server::registry::{Enhancer, TopicConfig};
    use crate::types::{PageMap, TextSegment, Token};

    /// The page the cache tests analyse. Its text is North Sámi, so a token
    /// offset one byte out lands inside a character rather than between two.
    const PAGE: &str = "<html><body><p>Sámegiella lea somá.</p></body></html>";

    /// What the preprocessing flow leaves as the document text for [`PAGE`].
    const ANALYSED: &str = "Sámegiella lea somá.";

    /// The cache key the handler builds its filename from.
    const KEY: &str = "http:--example.org-page";

    /// A registry offering nothing, so no flow pair is registered for any
    /// topic.
    fn empty_registry() -> Registry {
        Registry::empty()
    }

    /// A registry whose flow pair needs no models: the preprocessor turns the
    /// page into analysable text and the postprocessor wraps every token
    /// carrying an `N` tag, so which of the two ran over a document is
    /// readable off the document itself.
    ///
    /// The shipped preprocessing flow tokenises and runs the constraint
    /// grammar, which needs the models; what these tests are about is the
    /// cache around a flow, not the flow.
    fn model_free_registry() -> Registry {
        use crate::pipeline::flow::Stage;
        use crate::pipeline::relevance::GenericRelevanceAnnotator;

        let preprocessor = Flow::of_stages(vec![Stage::Relevance(GenericRelevanceAnnotator)]);
        let postprocessor = Flow::postprocessor(&TopicConfig {
            name: "Nouns".to_string(),
            label: "Substantiivvat".to_string(),
            enabled: true,
            enhancer: Enhancer::Noun,
            tags: "Sg Nom".to_string(),
            token_tags: "N".to_string(),
            use_lemma_filter: false,
        })
        .expect("the token enhancer takes its tags");

        Registry::of_flows("Nouns", preprocessor, postprocessor)
    }

    /// A handler over [`PAGE`] caching under `cache_dir`.
    fn handler_over<'a>(registry: &'a Registry, cache_dir: &'a Path) -> PageHandler<'a> {
        PageHandler::new(
            registry,
            "Nouns",
            KEY,
            cache_dir.to_str().expect("utf-8 path"),
            PAGE,
            Mode::Colorize,
        )
    }

    fn cache_file(cache_dir: &Path) -> PathBuf {
        cache_dir.join(format!("{KEY}.json"))
    }

    /// A cache file holding the analysed text with one token over `span`,
    /// written straight to disk the way an earlier run's cache would be.
    fn cache_a_document(cache_dir: &Path, span: (usize, usize)) {
        let mut cached = Document::new(ANALYSED);
        cached.page = PageMap {
            html: PAGE.to_string(),
            segments: vec![TextSegment {
                begin: 0,
                end: ANALYSED.len(),
                node: 0,
                block_start: true,
            }],
        };
        cached.tokens.push(Token {
            begin: span.0,
            end: span.1,
            tag: Some("N".to_string()),
            ..Token::default()
        });

        std::fs::create_dir_all(cache_dir).expect("the cache directory");
        std::fs::write(
            cache_file(cache_dir),
            serde_json::to_string(&cached).expect("the document encodes"),
        )
        .expect("the cache file");
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.page-handler.page-handler.page-handler-fn+4/test]
    #[test]
    fn constructor_maps_third_fourth_args_to_url_path() {
        let registry = empty_registry();

        let handler = PageHandler::new(
            &registry,
            "Nouns",
            "http:--example.org-page",
            "/home/teaksta/analyzedTexts",
            "Sámegiella lea somá.",
            Mode::Colorize,
        );

        assert!(std::ptr::eq(handler.registry, &registry));
        assert_eq!(handler.topic, "Nouns");
        assert_eq!(handler.url, "http:--example.org-page");
        assert_eq!(handler.path, "/home/teaksta/analyzedTexts");
        assert_eq!(handler.text, "Sámegiella lea somá.");
        assert_eq!(handler.mode, Mode::Colorize);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.page-handler.page-handler.page-handler-fn+4/test]
    #[test]
    fn constructor_stores_every_argument_untouched() {
        let registry = empty_registry();

        let handler = PageHandler::new(&registry, "Conjunctions", "  ", "", " &amp; ", Mode::Cloze);

        assert_eq!(handler.topic, "Conjunctions");
        assert_eq!(handler.url, "  ");
        assert_eq!(handler.path, "");
        assert_eq!(handler.text, " &amp; ");
        assert_eq!(handler.mode, Mode::Cloze);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.page-handler.page-handler.process-fn+7/test]
    #[test]
    fn process_returns_nothing_when_topic_lacks_pipelines() {
        let registry = empty_registry();
        let cache_root = tempfile::tempdir().expect("temp dir");
        let cache_dir = cache_root.path().join("analyzedTexts");
        let handler = PageHandler::new(
            &registry,
            "Nouns",
            "http:--example.org-page",
            cache_dir.to_str().expect("utf-8 path"),
            "Sámegiella",
            Mode::Colorize,
        );

        let processed = handler.process().expect("lookup miss is not an error");

        assert!(processed.is_none());
        // The lookup miss returns before the cache directory is created.
        assert!(!cache_dir.exists());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.page-handler.page-handler.process-fn+7/test]
    #[test]
    fn an_undecodable_cache_file_is_replaced() {
        let registry = model_free_registry();
        let cache_root = tempfile::tempdir().expect("temp dir");
        let cache_dir = cache_root.path().join("analyzedTexts");
        std::fs::create_dir_all(&cache_dir).expect("the cache directory");
        std::fs::write(cache_file(&cache_dir), "{\"text\":").expect("a truncated cache file");

        let document = handler_over(&registry, &cache_dir)
            .process()
            .expect("a cache file that will not decode is not an analysis failure")
            .expect("the topic has a pipeline pair");

        // The preprocessor ran, so the request is answered from the page
        // rather than from the empty document the read left behind.
        assert_eq!(document.text, ANALYSED);
        assert_eq!(document.page.html, PAGE);
        // And its output replaced the file, so the next request is a hit.
        let repaired = read_cached(&cache_file(&cache_dir)).expect("the cache file was rewritten");
        assert_eq!(repaired.text, ANALYSED);
        assert_eq!(repaired.page.html, PAGE);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.page-handler.page-handler.process-fn+7/test]
    #[test]
    fn a_readable_cache_file_is_postprocessed() {
        let registry = model_free_registry();
        let cache_root = tempfile::tempdir().expect("temp dir");
        let cache_dir = cache_root.path().join("analyzedTexts");
        cache_a_document(&cache_dir, (0, "Sámegiella".len()));
        let before = std::fs::read_to_string(cache_file(&cache_dir)).expect("the cache file");

        let document = handler_over(&registry, &cache_dir)
            .process()
            .expect("analysis succeeds")
            .expect("the topic has a pipeline pair");

        // Only the cached document carries a token, so an enhancement over
        // one is the postprocessor running over what the file held.
        assert_eq!(document.enhancements.len(), 1);
        assert_eq!(
            (document.enhancements[0].begin, document.enhancements[0].end),
            (0, "Sámegiella".len())
        );
        assert!(document.enhancements[0].relevant);
        assert_eq!(
            std::fs::read_to_string(cache_file(&cache_dir)).expect("the cache file"),
            before,
            "a readable cache file is left as it is"
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.page-handler.page-handler.process-fn+7/test]
    #[test]
    fn an_unreadable_cached_span_is_a_miss() {
        for span in [
            // Inside the `á` of `Sámegiella`, which occupies two bytes.
            (0, 2),
            // Past the end of the text the same file carries.
            (0, 9_999),
        ] {
            let registry = model_free_registry();
            let cache_root = tempfile::tempdir().expect("temp dir");
            let cache_dir = cache_root.path().join("analyzedTexts");
            cache_a_document(&cache_dir, span);

            let document = handler_over(&registry, &cache_dir)
                .process()
                .unwrap_or_else(|e| panic!("the span {span:?} was not survived: {e:#}"))
                .expect("the topic has a pipeline pair");

            // The file was treated as unreadable, so its token never reached
            // the enhancer and the page was analysed instead.
            assert!(document.enhancements.is_empty(), "span {span:?}");
            assert_eq!(document.text, ANALYSED, "span {span:?}");
            assert_eq!(document.page.html, PAGE, "span {span:?}");
            let repaired =
                read_cached(&cache_file(&cache_dir)).expect("the cache file was rewritten");
            assert!(repaired.tokens.is_empty(), "span {span:?}");
        }
    }
}
