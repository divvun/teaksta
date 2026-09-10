//! Integration tests for the topic pipelines against the shipped activity
//! and descriptor trees plus the real models. They run only when
//! TEAKSTA_BUNDLE (.drb with tokenize/analyze/sentences pipelines) and
//! TEAKSTA_GENERATOR (generator-gt-norm.hfstol) are set; without the models
//! each test reports itself skipped and passes.
//!
//! The URL fetch in the servlet is skipped: the page handler is driven
//! directly on an HTML string, which is the same entry point both the web
//! form and the add-on protocol reach once their document is in hand.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use teaksta::morpho::{BUNDLE_ENV, GENERATOR_ENV};
use teaksta::server::activities::Activities;
use teaksta::server::activity_configuration::set_classpath_root;
use teaksta::server::processors::Processors;
use teaksta::server::servlet::ENHANCEMENT_TYPE;
use teaksta::types::Document;
use teaksta::util::json_enhancer::JsonEnhancer;
use teaksta::util::page_handler::PageHandler;

/// The `<e>`-tagged document the servlet hands the page handler, with the
/// span ids the add-on protocol carries.
const DOCUMENT: &str = concat!(
    "<html><head><title>t</title></head><body>",
    "<p><e id=\"1\">Mun oidnen viesu ikte.</e></p>",
    "<p><e id=\"2\">Viesut leat stuorr\u{e1}t.</e></p>",
    "</body></html>"
);

fn models_available() -> bool {
    let ok = std::env::var(BUNDLE_ENV).is_ok() && std::env::var(GENERATOR_ENV).is_ok();
    if !ok {
        eprintln!("skipped: {BUNDLE_ENV}/{GENERATOR_ENV} not set");
    }
    ok
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The registry over the shipped activities, with the descriptor tree pinned
/// as the classpath. Both are process-wide, so they are set up once.
fn shipped_processors() -> &'static Processors {
    static PROCESSORS: OnceLock<Processors> = OnceLock::new();
    PROCESSORS.get_or_init(|| {
        let root = repository_root();
        set_classpath_root(root.join("sme/desc"));
        let mut activities = Activities::new(&root.join("sme/src/main/webapp/activities"))
            .expect("the shipped activity tree");
        Processors::new(&mut activities).expect("a pipeline per topic and language")
    })
}

fn set_enhancement(enhancement: &str) {
    *ENHANCEMENT_TYPE
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(enhancement.to_string());
}

/// One page-handler run over [`DOCUMENT`], against a cache directory of its
/// own so the run is not answered from an earlier one.
fn analysed(topic: &str, enhancement: &str) -> Document {
    set_enhancement(enhancement);
    let cache = tempfile::tempdir().expect("a cache directory");
    let handler = PageHandler::new(
        shipped_processors(),
        topic,
        &format!("http:--example.org-{topic}-{enhancement}"),
        cache.path().to_str().expect("utf-8 path"),
        DOCUMENT,
        "en",
    );
    handler
        .process()
        .expect("the topic pipeline runs")
        .expect("the topic is registered for the language")
}

fn span_starts(cas: &Document) -> Vec<String> {
    cas.enhancements
        .iter()
        .map(|e| e.enhance_start.clone())
        .collect()
}

#[test]
fn every_shipped_topic_builds_both_pipelines() {
    if !models_available() {
        return;
    }
    let processors = shipped_processors();

    for topic in [
        "Substantive",
        "SubstantiveSingular",
        "SubstantivePlural",
        "VerbConjugation",
        "NegVerbs",
        "InfiniteVerbs",
        "Adverbial",
        "Conjunctions",
        "Object",
        "Subject",
    ] {
        assert!(
            processors.get_preprocessor("en", topic).is_some(),
            "{topic} has no preprocessor"
        );
        assert!(
            processors.get_postprocessor("en", topic).is_some(),
            "{topic} has no postprocessor"
        );
    }
    // The activity tree declares "en" only, so no other language resolves.
    assert!(processors.get_preprocessor("sme", "Substantive").is_none());
}

#[test]
fn the_preprocessor_yields_cg_tokens_with_readings() {
    if !models_available() {
        return;
    }
    let cas = analysed("Substantive", "colorize");

    let readings: Vec<String> = cas
        .cg_tokens
        .iter()
        .map(|token| token.readings.concat().join("+"))
        .collect();
    assert!(
        readings
            .iter()
            .any(|r| r.contains("\"viessu\"+N+Sem/Build+Sg+Acc+@<OBJ")),
        "the accusative reading is missing: {readings:#?}"
    );
    assert!(
        readings
            .iter()
            .any(|r| r.contains("\"oaidnit\"+V+TV+Ind+Prt+Sg1+@+FMAINV")),
        "the verb reading is missing: {readings:#?}"
    );
    // The weight and cohort-tracking markers of the modern stream never
    // reach a reading.
    assert!(
        !readings
            .iter()
            .any(|r| r.contains("<W:") || r.contains("Cohort")),
        "a runtime marker survived: {readings:#?}"
    );
    // Every original token is replaced by the CG token covering its span.
    assert!(cas.tokens.is_empty());
}

#[test]
fn colorize_wraps_the_nouns_in_topic_spans() {
    if !models_available() {
        return;
    }
    let cas = analysed("Substantive", "colorize");

    let starts = span_starts(&cas);
    assert_eq!(starts.len(), 2, "{starts:#?}");
    for start in &starts {
        assert!(start.contains("class=\"wertiviewtoken  wertiviewSubstantive\""));
        assert!(start.contains("lemma=\"viessu\""));
    }
    assert_eq!(
        cas.covered_text(cas.enhancements[0].begin, cas.enhancements[0].end),
        "viesu"
    );
    assert_eq!(
        cas.covered_text(cas.enhancements[1].begin, cas.enhancements[1].end),
        "Viesut"
    );
}

#[test]
fn the_json_protocol_keys_spans_by_id() {
    if !models_available() {
        return;
    }
    let cas = analysed("Substantive", "colorize");

    let json = JsonEnhancer::new(&cas, "colorize")
        .enhance()
        .expect("a span map");
    let spans: std::collections::HashMap<String, String> =
        serde_json::from_str(&json).expect("a JSON object");

    assert_eq!(spans.len(), 2);
    assert!(
        spans["1"].contains("wertiviewSubstantive"),
        "{}",
        spans["1"]
    );
    assert!(spans["1"].contains(">viesu</span>"), "{}", spans["1"]);
    assert!(spans["2"].contains(">Viesut</span>"), "{}", spans["2"]);
    for span in spans.values() {
        assert!(
            span.starts_with("<span class=\"wertiview\" style="),
            "{span}"
        );
    }
}

#[test]
fn mc_attaches_distractors_and_the_answer() {
    if !models_available() {
        return;
    }
    let cas = analysed("Substantive", "mc");

    let starts = span_starts(&cas);
    assert_eq!(starts.len(), 2, "{starts:#?}");
    assert!(starts[0].contains("answer=\"viesu\""), "{}", starts[0]);
    assert!(starts[0].contains("distractors=\""), "{}", starts[0]);
    assert!(starts[1].contains("answer=\"viesut\""), "{}", starts[1]);
    for start in &starts {
        let distractors = start
            .split("distractors=\"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .expect("a distractor list");
        assert!(
            distractors.split(' ').count() >= 2,
            "too few distractors: {distractors}"
        );
    }
}

#[test]
fn cloze_attaches_the_generable_forms() {
    if !models_available() {
        return;
    }
    let cas = analysed("Substantive", "cloze");

    let starts = span_starts(&cas);
    assert_eq!(starts.len(), 2, "{starts:#?}");
    assert!(starts[0].contains("possibleforms=\"viesu"), "{}", starts[0]);
    assert!(
        starts[1].contains("possibleforms=\"viesut"),
        "{}",
        starts[1]
    );
}

#[test]
fn a_second_run_is_answered_from_the_cache() {
    if !models_available() {
        return;
    }
    set_enhancement("colorize");
    let cache = tempfile::tempdir().expect("a cache directory");
    let handler = PageHandler::new(
        shipped_processors(),
        "Substantive",
        "http:--example.org-cached",
        cache.path().to_str().expect("utf-8 path"),
        DOCUMENT,
        "en",
    );

    let first = handler.process().expect("first run").expect("a cas");
    let second = handler.process().expect("cached run").expect("a cas");

    assert!(
        cache
            .path()
            .join("cas_http:--example.org-cached.xmi")
            .is_file()
    );
    assert_eq!(span_starts(&first), span_starts(&second));
    assert_eq!(first.text, second.text);
}
