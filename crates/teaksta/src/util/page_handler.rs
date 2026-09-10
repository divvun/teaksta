//! Methods needed for processing a document regardless of whether it came
//! from the web form or from the add-on.
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
use tracing::{error, info};

use crate::server::processors::{AnalysisEngine, Processors};
use crate::types::Document;

/// The two checked failures the original tells apart by catch clause. They
/// carry different log messages but produce the same `ServletException`.
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

/// `AnalysisEngine#process(JCas)`: runs the engine's fixed flow over the CAS.
fn analysis_engine_process(
    engine: &AnalysisEngine,
    cas: &mut Document,
) -> std::result::Result<(), EngineError> {
    engine
        .process(cas)
        .map_err(|e| EngineError::AnalysisEngineProcess(format!("{e:#}")))
}

/// `CasIOUtil.readXmi(cas, file)`: replaces the CAS contents with the cached
/// document. The encoding is JSON rather than XMI — the document model is not
/// a UIMA type system and has no XMI representation — so a cache file written
/// by another build is rejected as unreadable, which is the case the caller
/// already handles.
fn read_xmi(cas: &mut Document, casfile: &Path) -> std::io::Result<()> {
    let encoded = std::fs::read_to_string(casfile)?;
    *cas = serde_json::from_str(&encoded)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(())
}

/// `CasIOUtil.writeXmi(cas, file)`. The counterpart of [`read_xmi`].
fn write_xmi(cas: &Document, casfile: &Path) -> std::io::Result<()> {
    let encoded = serde_json::to_string(cas)?;
    std::fs::write(casfile, encoded)
}

// [spec:teaksta:def:sme.src.main.java.werti.util.page-handler.page-handler]
pub struct PageHandler<'a> {
    processors: &'a Processors,
    topic: String,
    text: String,
    lang: String,
    url: String,
    path: String,
}

impl<'a> PageHandler<'a> {
    // [spec:teaksta:def:sme.src.main.java.werti.util.page-handler.page-handler.page-handler-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.page-handler.page-handler.page-handler-fn]
    pub fn new(
        a_processors: &'a Processors,
        a_topic: &str,
        a_url: &str,
        a_path: &str,
        a_text: &str,
        a_lang: &str,
    ) -> Self {
        // The assignment order differs from the parameter order: `url` and
        // `path` are the third and fourth arguments but the fifth and sixth
        // assignments.
        PageHandler {
            processors: a_processors,
            topic: a_topic.to_string(),
            text: a_text.to_string(),
            lang: a_lang.to_string(),
            url: a_url.to_string(),
            path: a_path.to_string(),
        }
        // A disabled branch here would have forced `lang` to `sme` whenever
        // `topic` was `Conjunctions`.
    }

    /// Creates a CAS from the text and runs the pre- and postprocessors for the
    /// topic.
    // [spec:teaksta:def:sme.src.main.java.werti.util.page-handler.page-handler.process-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.page-handler.page-handler.process-fn+2]
    pub fn process(&self) -> Result<Option<Document>> {
        let preprocessor = self.processors.get_preprocessor(&self.lang, &self.topic);
        let postprocessor = self.processors.get_postprocessor(&self.lang, &self.topic);
        if let (Some(preprocessor), Some(postprocessor)) = (preprocessor, postprocessor) {
            // to process
            let processed = (|| -> std::result::Result<Document, EngineError> {
                let mut cas = new_jcas(preprocessor)?;
                // convert HTML entities to characters, if there are any
                let normalised_text = html_escape::decode_html_entities(&self.text).into_owned();
                // add the normalised text to cas
                cas.text = normalised_text;
                cas.language = self.lang.clone();
                let casfile_path = PathBuf::from(&self.path);
                if !casfile_path.exists() {
                    let _ = std::fs::create_dir_all(&casfile_path);
                }
                let casfile = casfile_path.join(format!("cas_{}.xmi", self.url));
                if casfile.is_file() {
                    let cached = read_xmi(&mut cas, &casfile);
                    match cached {
                        // The postprocessor runs from inside the same try
                        // block as the read; only the read raises the
                        // `IOException` that lands in the handler below.
                        Ok(()) => analysis_engine_process(postprocessor, &mut cas)?,
                        Err(cas_read) => {
                            info!("Failed to load cas from file! {}", cas_read);
                        }
                    }
                } else {
                    analysis_engine_process(preprocessor, &mut cas)?;
                    let written = write_xmi(&cas, &casfile);
                    match written {
                        // The postprocessor runs from inside the same try
                        // block as the write, so a failed write skips it.
                        Ok(()) => analysis_engine_process(postprocessor, &mut cas)?,
                        Err(cas_write) => {
                            info!("Failed to write cas to file! {}", cas_write);
                        }
                    }
                }
                Ok(cas)
            })();

            return match processed {
                Ok(cas) => Ok(Some(cas)),
                Err(aepe @ EngineError::AnalysisEngineProcess(_)) => {
                    error!("Analysis Engine encountered errors! {}", aepe);
                    Err(anyhow::Error::new(aepe).context("Text analysis failed."))
                }
                Err(rie @ EngineError::ResourceInitialization(_)) => {
                    error!("Resource Initialization Engine encountered errors! {}", rie);
                    Err(anyhow::Error::new(rie).context("Text analysis failed."))
                }
            };
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::server::activities::Activities;

    /// A registry built over a directory holding no activities, so no engine
    /// pair is registered for any (language, topic).
    fn empty_processors() -> Processors {
        let activity_dir = tempfile::tempdir().expect("temp dir");
        let mut activities = Activities::new(activity_dir.path()).expect("activities");
        Processors::new(&mut activities).expect("processors")
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.page-handler.page-handler.page-handler-fn/test]
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
        );

        assert!(std::ptr::eq(handler.processors, &processors));
        assert_eq!(handler.topic, "Nouns");
        assert_eq!(handler.url, "http:--example.org-page");
        assert_eq!(handler.path, "/home/teaksta/analyzedTexts");
        assert_eq!(handler.text, "Sámegiella lea somá.");
        assert_eq!(handler.lang, "sme");
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.page-handler.page-handler.page-handler-fn/test]
    #[test]
    fn constructor_stores_every_argument_untouched() {
        let processors = empty_processors();

        let handler = PageHandler::new(&processors, "Conjunctions", "  ", "", " &amp; ", "eng");

        assert_eq!(handler.topic, "Conjunctions");
        assert_eq!(handler.url, "  ");
        assert_eq!(handler.path, "");
        assert_eq!(handler.text, " &amp; ");
        assert_eq!(handler.lang, "eng");
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.page-handler.page-handler.process-fn+2/test]
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
            "sme",
        );

        let processed = handler.process().expect("lookup miss is not an error");

        assert!(processed.is_none());
        // The lookup miss returns before the cache directory is created.
        assert!(!cache_dir.exists());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.page-handler.page-handler.process-fn+2/test]
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
        );

        assert!(
            handler
                .process()
                .expect("lookup miss is not an error")
                .is_none()
        );
    }
}
