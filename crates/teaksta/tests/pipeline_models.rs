//! Integration tests for the topic pipelines against the shipped activity
//! and descriptor trees plus the real models. They run only when
//! TEAKSTA_BUNDLE (.drb with tokenize/analyze/sentences pipelines) and
//! TEAKSTA_GENERATOR (generator-gt-norm.hfstol) are set; without the models
//! each test reports itself skipped and passes.
//!
//! The URL fetch in the servlet is skipped: the page handler is driven
//! directly on an HTML string, which is the same entry point both the web
//! form and the add-on protocol reach once their document is in hand.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

use teaksta::morpho::{BUNDLE_ENV, GENERATOR_ENV};
use teaksta::server::activities::Activities;
use teaksta::server::activities::HttpServletRequest as SessionRequest;
use teaksta::server::activities::{HttpSession, ServletContext as SessionServletContext};
use teaksta::server::activity_configuration::ActivityConfiguration;
use teaksta::server::activity_configuration::set_classpath_root;
use teaksta::server::processors::Processors;
use teaksta::server::servlet::{
    ENHANCEMENT_TYPE, HttpServletRequest, HttpServletResponse, ServletConfig, ServletContext,
    WertiServlet,
};
use teaksta::types::Document;
use teaksta::util::html_enhancer::HtmlEnhancer;
use teaksta::util::json_enhancer::JsonEnhancer;
use teaksta::util::page_handler::PageHandler;

/// The page the servlet hands the page handler, as it was fetched.
const DOCUMENT: &str = concat!(
    "<html><head><title>t</title></head><body>",
    "<p>Mun oidnen viesu ikte.</p>",
    "<p>Viesut leat stuorr\u{e1}t.</p>",
    "</body></html>"
);

/// The same page as the add-on posts it: its own `wertiview` markers, which
/// the servlet still rewrites into `<e>` elements before analysis.
const POSTED_DOCUMENT: &str = concat!(
    "<html><head><title>t</title></head><body>",
    "<p><span class=\"wertiview\" wertiviewid=\"1\">Mun oidnen viesu ikte.</span></p>",
    "<p><span class=\"wertiview\" wertiviewid=\"2\">Viesut leat stuorr\u{e1}t.</span></p>",
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

/// The activity the enhancers read is one process-wide field, so the runs
/// that set it take turns rather than clobbering each other.
static ACTIVITY: Mutex<()> = Mutex::new(());

fn activity_lock() -> MutexGuard<'static, ()> {
    ACTIVITY
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// One page-handler run over [`DOCUMENT`], against a cache directory of its
/// own so the run is not answered from an earlier one.
fn analysed(topic: &str, enhancement: &str) -> Document {
    let _activity = activity_lock();
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

/// One add-on request, driven through the servlet the browser extension
/// posts to. Returns the JSON span map from the response body.
fn posted(topic: &str, activity: &str) -> HashMap<String, String> {
    // The registry pins the descriptor tree the servlet's own load reads.
    shipped_processors();
    let _activity = activity_lock();
    // Whatever an earlier request left behind is what the add-on protocol
    // used to be stuck with.
    set_enhancement("colorize");
    let webapp = repository_root().join("sme/src/main/webapp");
    let cache = tempfile::tempdir().expect("a cache directory");
    let mut servlet = WertiServlet::new();
    servlet
        .init(ServletConfig {
            servlet_context: ServletContext {
                init_parameters: BTreeMap::from([(
                    "files_anl_dir".to_string(),
                    cache.path().to_string_lossy().into_owned(),
                )]),
                servlet_context_name: Some("VIEW".to_string()),
            },
        })
        .expect("the servlet initialises");
    let req = HttpServletRequest {
        body: serde_json::json!({
            "type": "page",
            "version": "0.10",
            "topic": topic,
            "activity": activity,
            "language": "en",
            "url": format!("http://example.org/{topic}-{activity}"),
            "document": POSTED_DOCUMENT,
        })
        .to_string(),
        ..HttpServletRequest::default()
    };
    let mut session_request =
        SessionRequest::new(HttpSession::new(SessionServletContext::new(Some(webapp))));
    let mut resp = HttpServletResponse::default();

    servlet
        .handle_post(&req, &mut session_request, &mut resp)
        .expect("the add-on request is answered");

    assert_eq!(resp.content_type.as_deref(), Some("text/plain"));
    serde_json::from_str(&resp.body).expect("a JSON object")
}

fn span_starts(cas: &Document) -> Vec<String> {
    cas.enhancements
        .iter()
        .map(|e| e.enhance_start.clone())
        .collect()
}

/// The span map's entries in document order, which is the order of their
/// keys read as positions rather than as strings.
fn ordered(spans: &HashMap<String, String>) -> Vec<String> {
    let mut entries: Vec<(usize, String)> = spans
        .iter()
        .map(|(at, span)| (at.parse().expect("a document position"), span.clone()))
        .collect();
    entries.sort();
    entries.into_iter().map(|(_, span)| span).collect()
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
        assert!(start.contains("class=\"wertiviewtoken wertiviewSubstantive\""));
        assert!(start.contains(" lemma=\"viessu\""));
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
fn the_page_flow_wraps_tokens_in_the_page() {
    if !models_available() {
        return;
    }
    let cas = analysed("Substantive", "colorize");
    let dir = tempfile::tempdir().expect("a config directory");
    let path = dir.path().join("activity.xml");
    std::fs::write(&path, "<activity><server-cfg></server-cfg></activity>").expect("a descriptor");
    let config = ActivityConfiguration::new(&path).expect("the descriptor loads");
    let mut req = HttpServletRequest {
        request_url: "http://example.org/teaksta".to_string(),
        ..HttpServletRequest::default()
    };
    req.parameters
        .insert("client.enhancement".to_string(), "colorize".to_string());

    let page = HtmlEnhancer::new(&cas)
        .enhance(
            "Substantive",
            "http://example.org/page.html",
            &req,
            &config,
            "teaksta",
        )
        .expect("an enhanced page");

    // The page comes back whole, with the enhancements wrapped around the
    // words they cover and nothing else added but the base URL.
    assert!(
        page.contains("<base href=\"http://example.org/page.html\">"),
        "{page}"
    );
    assert!(!page.contains("<script"), "{page}");
    assert!(page.contains("<title>t</title>"), "{page}");
    assert!(page.contains("Mun oidnen "), "{page}");
    assert!(page.contains(">viesu</span> ikte."), "{page}");
    assert!(page.contains(">Viesut</span> leat"), "{page}");
    assert_eq!(page.matches("token").count(), 2, "{page}");
}

#[test]
fn the_json_protocol_keys_spans_by_position() {
    if !models_available() {
        return;
    }
    let cas = analysed("Substantive", "colorize");

    let json = JsonEnhancer::new(&cas, "colorize")
        .enhance()
        .expect("a span map");
    let spans: HashMap<String, String> = serde_json::from_str(&json).expect("a JSON object");

    assert_eq!(spans.len(), 2);
    // Each key is the position in the document text the enhancement covers.
    for (at, span) in &spans {
        let at: usize = at.parse().expect("a document position");
        let covered = cas
            .enhancements
            .iter()
            .find(|e| e.begin == at)
            .expect("an enhancement at that position");
        assert!(
            span.contains(&format!(">{}</span>", cas.covered_text(at, covered.end))),
            "{span}"
        );
        assert!(span.contains("token"), "{span}");
        assert!(
            span.starts_with("<span class=\"teaksta-page\" style="),
            "{span}"
        );
    }

    let ordered = ordered(&spans);
    assert!(ordered[0].contains(">viesu</span>"), "{}", ordered[0]);
    assert!(ordered[1].contains(">Viesut</span>"), "{}", ordered[1]);
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
    let _activity = activity_lock();
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

#[test]
fn the_json_protocol_serves_the_requested_activity() {
    if !models_available() {
        return;
    }
    let colorize = posted("Substantive", "colorize");
    let mc = posted("Substantive", "mc");
    let cloze = posted("Substantive", "cloze");

    for spans in [&colorize, &mc, &cloze] {
        assert_eq!(spans.len(), 2, "{spans:#?}");
    }
    let colorize = ordered(&colorize);
    let mc = ordered(&mc);
    let cloze = ordered(&cloze);
    // colorize only marks the token up; mc and cloze reach the generator
    // over the same protocol.
    assert!(!colorize[0].contains("distractors="), "{}", colorize[0]);
    assert!(!colorize[0].contains("possibleforms="), "{}", colorize[0]);
    assert!(colorize[0].contains(">viesu</span>"), "{}", colorize[0]);
    assert!(mc[0].contains("distractors="), "{}", mc[0]);
    assert!(mc[0].contains("answer=\"viesu\""), "{}", mc[0]);
    assert!(cloze[0].contains("possibleforms=\"viesu"), "{}", cloze[0]);
}
