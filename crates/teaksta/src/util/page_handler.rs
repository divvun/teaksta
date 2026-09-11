//! Running one topic's pipeline pair over a page, with the analysed document
//! cached on disk under a caller-chosen key.
//!
//! Author: Adriane Boyd
//!
//! The UIMA CAS is [`crate::types::Document`]. Running an engine's flow over
//! a CAS is [`crate::pipeline::flow::Flow`]. The on-disk CAS cache is kept,
//! but XMI — the CAS's own serialisation format, which only exists inside
//! UIMA — is replaced by a JSON encoding of the document model; the cache
//! file keeps its `.xmi` name so a deployment's cache directory is still
//! recognisable.

use std::path::{Path, PathBuf};

use anyhow::Result;
use tracing::{error, info, warn};

use crate::server::api::Mode;
use crate::server::processors::{AnalysisEngine, Processors};
use crate::types::Document;

/// The two failure kinds the engine seam tells apart. They carry different
/// log messages but reach the caller as the same analysis failure.
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("AnalysisEngineProcessException: {0}")]
    AnalysisEngineProcess(String),
    #[error("ResourceInitializationException: {0}")]
    ResourceInitialization(String),
}

/// `AnalysisEngine#newJCas()`: an empty CAS over the engine's type system.
/// The document model is not derived from the descriptor, so this cannot fail
/// and the `ResourceInitializationException` arm below never fires.
fn new_jcas(_engine: &AnalysisEngine) -> std::result::Result<Document, EngineError> {
    Ok(Document::default())
}

/// `AnalysisEngine#process(JCas)`: runs the engine's fixed flow over the CAS
/// for the requested exercise.
fn analysis_engine_process(
    engine: &AnalysisEngine,
    cas: &mut Document,
    mode: Mode,
) -> std::result::Result<(), EngineError> {
    engine
        .process(cas, mode)
        .map_err(|e| EngineError::AnalysisEngineProcess(format!("{e:#}")))
}

/// `CasIOUtil.readXmi(cas, file)`: the cached document. The encoding is JSON
/// rather than XMI — the document model is not a UIMA type system and has no
/// XMI representation — so a cache file written by another build is rejected
/// as unreadable, which is the case the caller already handles.
///
/// The file is the one input to the pipeline nothing in this process wrote,
/// so its offsets are checked against the text it carries before it is handed
/// on: a span the text cannot be read at makes the file unreadable, on the
/// same footing as one that will not decode at all.
fn read_xmi(casfile: &Path) -> std::io::Result<Document> {
    let encoded = std::fs::read_to_string(casfile)?;
    let cas: Document = serde_json::from_str(&encoded)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    if let Some(span) = cas.invalid_span() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            span.to_string(),
        ));
    }
    Ok(cas)
}

/// `CasIOUtil.writeXmi(cas, file)`. The counterpart of [`read_xmi`].
fn write_xmi(cas: &Document, casfile: &Path) -> std::io::Result<()> {
    let encoded = serde_json::to_string(cas)?;
    std::fs::write(casfile, encoded)
}

/// One request's analysis: the page, where to cache it and which pipeline
/// pair to run over it.
///
/// Everything but the exercise is borrowed for the life of the handler. The
/// Java copied each argument into a field because a servlet request's strings
/// outlive nothing in particular; here the handler is built, used and dropped
/// inside the call that assembled its arguments, so a copy of the page would
/// be a copy of the whole page for nothing.
// [spec:teaksta:def:sme.src.main.java.werti.util.page-handler.page-handler]
pub struct PageHandler<'a> {
    processors: &'a Processors,
    topic: &'a str,
    text: &'a str,
    lang: &'a str,
    url: &'a str,
    path: &'a str,
    /// The exercise the request asked for, carried to the postprocessing
    /// enhancers that decide what to attach to a token from it.
    mode: Mode,
}

impl<'a> PageHandler<'a> {
    // [spec:teaksta:def:sme.src.main.java.werti.util.page-handler.page-handler.page-handler-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.page-handler.page-handler.page-handler-fn+2]
    pub fn new(
        a_processors: &'a Processors,
        a_topic: &'a str,
        a_url: &'a str,
        a_path: &'a str,
        a_text: &'a str,
        a_lang: &'a str,
        a_mode: Mode,
    ) -> Self {
        // The assignment order differs from the parameter order: `url` and
        // `path` are the third and fourth arguments but the fifth and sixth
        // assignments.
        PageHandler {
            processors: a_processors,
            topic: a_topic,
            text: a_text,
            lang: a_lang,
            url: a_url,
            path: a_path,
            mode: a_mode,
        }
        // A disabled branch here would have forced `lang` to `sme` whenever
        // `topic` was `Conjunctions`.
    }

    /// Creates a CAS from the text and runs the pre- and postprocessors for the
    /// topic.
    // [spec:teaksta:def:sme.src.main.java.werti.util.page-handler.page-handler.process-fn+5]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.page-handler.page-handler.process-fn+5]
    pub fn process(&self) -> Result<Option<Document>> {
        let preprocessor = self.processors.get_preprocessor(self.lang, self.topic);
        let postprocessor = self.processors.get_postprocessor(self.lang, self.topic);
        let (Some(preprocessor), Some(postprocessor)) = (preprocessor, postprocessor) else {
            return Ok(None);
        };

        match self.analysed(preprocessor, postprocessor) {
            Ok(cas) => Ok(Some(cas)),
            Err(aepe @ EngineError::AnalysisEngineProcess(_)) => {
                error!("Analysis Engine encountered errors! {}", aepe);
                Err(anyhow::Error::new(aepe).context("Text analysis failed."))
            }
            Err(rie @ EngineError::ResourceInitialization(_)) => {
                error!("Resource Initialization Engine encountered errors! {}", rie);
                Err(anyhow::Error::new(rie).context("Text analysis failed."))
            }
        }
    }

    /// The body of the `try` block: everything that can raise one of the two
    /// engine failures the caller above tells apart.
    fn analysed(
        &self,
        preprocessor: &AnalysisEngine,
        postprocessor: &AnalysisEngine,
    ) -> std::result::Result<Document, EngineError> {
        let mut cas = new_jcas(preprocessor)?;
        // convert HTML entities to characters, if there are any
        // add the normalised text to cas
        cas.text = html_escape::decode_html_entities(self.text).into_owned();
        cas.language = self.lang.to_string();

        let casfile_path = PathBuf::from(self.path);
        if !casfile_path.exists()
            && let Err(e) = std::fs::create_dir_all(&casfile_path)
        {
            // The cache is an optimisation, so a directory that cannot be
            // made costs this request nothing beyond its cache — but it
            // costs every later one the same, which is a deployment fault
            // worth naming rather than discarding.
            warn!(
                "Failed to create the cas directory {}! {}",
                casfile_path.display(),
                e
            );
        }
        let casfile = casfile_path.join(format!("cas_{}.xmi", self.url));
        let cached = match casfile.is_file() {
            true => read_xmi(&casfile)
                .inspect_err(|cas_read| {
                    info!("Failed to load cas from file! {}", cas_read);
                })
                .ok(),
            false => None,
        };

        match cached {
            // The cached document is the preprocessor's output, so the
            // preprocessor is not run over it again.
            Some(cached) => cas = cached,
            // A file that could not be read is rewritten from this run's
            // analysis, so a cache the deployment cannot decode costs one
            // request rather than every later one.
            None => {
                analysis_engine_process(preprocessor, &mut cas, self.mode)?;
                if let Err(cas_write) = write_xmi(&cas, &casfile) {
                    info!("Failed to write cas to file! {}", cas_write);
                }
            }
        }

        // Whichever branch produced the document, the request is answered
        // from the postprocessor's output: a cache that cannot be read or
        // written costs the request its cache, not its enhancement.
        analysis_engine_process(postprocessor, &mut cas, self.mode)?;
        Ok(cas)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::pipeline::flow::{Flow, Parameters};
    use crate::server::activities::Activities;
    use crate::types::{PIPELINE_LANGUAGE, PageMap, TextSegment, Token};

    /// The page the cache tests analyse. Its text is North Sámi, so a token
    /// offset one byte out lands inside a character rather than between two.
    const PAGE: &str = "<html><body><p>Sámegiella lea somá.</p></body></html>";

    /// What the preprocessing flow leaves as the document text for [`PAGE`].
    const ANALYSED: &str = "Sámegiella lea somá.";

    /// The cache key the handler builds its filename from.
    const KEY: &str = "http:--example.org-page";

    /// A registry built over a directory holding no activities, so no engine
    /// pair is registered for any (language, topic). Nothing resolves a
    /// descriptor, so the root the registry is handed is never reached.
    fn empty_processors() -> Processors {
        let activity_dir = tempfile::tempdir().expect("temp dir");
        let mut activities =
            Activities::new(activity_dir.path(), activity_dir.path()).expect("activities");
        Processors::new(&mut activities).expect("processors")
    }

    /// A registry whose engine pair needs no models: the preprocessor turns
    /// the page into analysable text and the postprocessor wraps every token
    /// carrying an `N` tag, so which of the two ran over a document is
    /// readable off the document itself.
    fn model_free_processors() -> Processors {
        let preprocessor = Flow::new(
            &["GenericRelevanceAnnotator".to_string()],
            &Parameters::new(),
        )
        .expect("the relevance annotator needs no parameters");
        let postprocessor = Flow::new(
            &["TokenEnhancer".to_string()],
            &Parameters::from([("Tags".to_string(), "N".to_string())]),
        )
        .expect("the token enhancer takes its tags");

        Processors::of_flows("sme", "Nouns", preprocessor, postprocessor)
    }

    /// A handler over [`PAGE`] caching under `cache_dir`.
    fn handler_over<'a>(processors: &'a Processors, cache_dir: &'a Path) -> PageHandler<'a> {
        PageHandler::new(
            processors,
            "Nouns",
            KEY,
            cache_dir.to_str().expect("utf-8 path"),
            PAGE,
            "sme",
            Mode::Colorize,
        )
    }

    fn casfile(cache_dir: &Path) -> PathBuf {
        cache_dir.join(format!("cas_{KEY}.xmi"))
    }

    /// A cache file holding the analysed text with one token over `span`,
    /// written straight to disk the way an earlier run's cache would be.
    fn cache_a_document(cache_dir: &Path, span: (usize, usize)) {
        let mut cached = Document::new(ANALYSED, PIPELINE_LANGUAGE);
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
            casfile(cache_dir),
            serde_json::to_string(&cached).expect("the document encodes"),
        )
        .expect("the cache file");
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.page-handler.page-handler.page-handler-fn+2/test]
    #[test]
    fn constructor_maps_third_fourth_args_to_url_path() {
        let processors = empty_processors();

        let handler = PageHandler::new(
            &processors,
            "Nouns",
            "http:--example.org-page",
            "/home/teaksta/analyzedTexts",
            "Sámegiella lea somá.",
            "sme",
            Mode::Colorize,
        );

        assert!(std::ptr::eq(handler.processors, &processors));
        assert_eq!(handler.topic, "Nouns");
        assert_eq!(handler.url, "http:--example.org-page");
        assert_eq!(handler.path, "/home/teaksta/analyzedTexts");
        assert_eq!(handler.text, "Sámegiella lea somá.");
        assert_eq!(handler.lang, "sme");
        assert_eq!(handler.mode, Mode::Colorize);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.page-handler.page-handler.page-handler-fn+2/test]
    #[test]
    fn constructor_stores_every_argument_untouched() {
        let processors = empty_processors();

        let handler = PageHandler::new(
            &processors,
            "Conjunctions",
            "  ",
            "",
            " &amp; ",
            "eng",
            Mode::Cloze,
        );

        assert_eq!(handler.topic, "Conjunctions");
        assert_eq!(handler.url, "  ");
        assert_eq!(handler.path, "");
        assert_eq!(handler.text, " &amp; ");
        assert_eq!(handler.lang, "eng");
        assert_eq!(handler.mode, Mode::Cloze);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.page-handler.page-handler.process-fn+5/test]
    #[test]
    fn process_returns_nothing_when_topic_lacks_engines() {
        let processors = empty_processors();
        let cache_root = tempfile::tempdir().expect("temp dir");
        let cache_dir = cache_root.path().join("analyzedTexts");
        let handler = PageHandler::new(
            &processors,
            "Nouns",
            "http:--example.org-page",
            cache_dir.to_str().expect("utf-8 path"),
            "Sámegiella",
            // The key a request really carries, so the miss is the topic's.
            PIPELINE_LANGUAGE,
            Mode::Colorize,
        );

        let processed = handler.process().expect("lookup miss is not an error");

        assert!(processed.is_none());
        // The lookup miss returns before the cache directory is created.
        assert!(!cache_dir.exists());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.page-handler.page-handler.process-fn+5/test]
    #[test]
    fn process_returns_nothing_when_the_language_is_unknown() {
        let processors = empty_processors();
        let cache_root = tempfile::tempdir().expect("temp dir");
        let handler = PageHandler::new(
            &processors,
            "Nouns",
            "page",
            cache_root.path().to_str().expect("utf-8 path"),
            "text",
            "klingon",
            Mode::Click,
        );

        assert!(
            handler
                .process()
                .expect("lookup miss is not an error")
                .is_none()
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.page-handler.page-handler.process-fn+5/test]
    #[test]
    fn an_undecodable_cache_file_is_replaced() {
        let processors = model_free_processors();
        let cache_root = tempfile::tempdir().expect("temp dir");
        let cache_dir = cache_root.path().join("analyzedTexts");
        std::fs::create_dir_all(&cache_dir).expect("the cache directory");
        std::fs::write(casfile(&cache_dir), "{\"text\":").expect("a truncated cache file");

        let document = handler_over(&processors, &cache_dir)
            .process()
            .expect("a cache file that will not decode is not an analysis failure")
            .expect("the topic has an engine pair");

        // The preprocessor ran, so the request is answered from the page
        // rather than from the empty document the read left behind.
        assert_eq!(document.text, ANALYSED);
        assert_eq!(document.page.html, PAGE);
        // And its output replaced the file, so the next request is a hit.
        let repaired = read_xmi(&casfile(&cache_dir)).expect("the cache file was rewritten");
        assert_eq!(repaired.text, ANALYSED);
        assert_eq!(repaired.page.html, PAGE);
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.page-handler.page-handler.process-fn+5/test]
    #[test]
    fn a_readable_cache_file_is_postprocessed() {
        let processors = model_free_processors();
        let cache_root = tempfile::tempdir().expect("temp dir");
        let cache_dir = cache_root.path().join("analyzedTexts");
        cache_a_document(&cache_dir, (0, "Sámegiella".len()));
        let before = std::fs::read_to_string(casfile(&cache_dir)).expect("the cache file");

        let document = handler_over(&processors, &cache_dir)
            .process()
            .expect("analysis succeeds")
            .expect("the topic has an engine pair");

        // Only the cached document carries a token, so an enhancement over
        // one is the postprocessor running over what the file held.
        assert_eq!(document.enhancements.len(), 1);
        assert_eq!(
            (document.enhancements[0].begin, document.enhancements[0].end),
            (0, "Sámegiella".len())
        );
        assert!(document.enhancements[0].relevant);
        assert_eq!(
            std::fs::read_to_string(casfile(&cache_dir)).expect("the cache file"),
            before,
            "a readable cache file is left as it is"
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.page-handler.page-handler.process-fn+5/test]
    #[test]
    fn an_unreadable_cached_span_is_a_miss() {
        for span in [
            // Inside the `á` of `Sámegiella`, which occupies two bytes.
            (0, 2),
            // Past the end of the text the same file carries.
            (0, 9_999),
        ] {
            let processors = model_free_processors();
            let cache_root = tempfile::tempdir().expect("temp dir");
            let cache_dir = cache_root.path().join("analyzedTexts");
            cache_a_document(&cache_dir, span);

            let document = handler_over(&processors, &cache_dir)
                .process()
                .unwrap_or_else(|e| panic!("the span {span:?} was not survived: {e:#}"))
                .expect("the topic has an engine pair");

            // The file was treated as unreadable, so its token never reached
            // the enhancer and the page was analysed instead.
            assert!(document.enhancements.is_empty(), "span {span:?}");
            assert_eq!(document.text, ANALYSED, "span {span:?}");
            assert_eq!(document.page.html, PAGE, "span {span:?}");
            let repaired = read_xmi(&casfile(&cache_dir)).expect("the cache file was rewritten");
            assert!(repaired.tokens.is_empty(), "span {span:?}");
        }
    }
}
