//! End-to-end tests over the real endpoints and the real models. They run
//! only when TEAKSTA_BUNDLE (.drb with tokenize/analyze/sentences pipelines)
//! and TEAKSTA_GENERATOR (generator-gt-norm.hfstol) are set; without the
//! models each test reports itself skipped and passes.
//!
//! The requests go through the router, so what is exercised is the handler
//! layer a deployment serves: the topic registry, the whole-page and span
//! enhancement endpoints, and the upload gate.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use poem::Endpoint;
use poem::EndpointExt;
use poem::http::StatusCode;
use poem::test::TestClient;
use serde_json::Value;
use tempfile::TempDir;

use teaksta::context::Config;
use teaksta::morpho::{BUNDLE_ENV, GENERATOR_ENV};
use teaksta::server::activity_configuration::set_classpath_root;
use teaksta::server::api::{AppState, routes};

/// The page every enhancement test is run over.
const DOCUMENT: &str = concat!(
    "<html><head><title>t</title></head><body>",
    "<p>Mun oidnen viesu ikte.</p>",
    "<p>Viesut leat stuorr\u{e1}t.</p>",
    "</body></html>"
);

/// A page in another language, for the upload gate.
const ENGLISH_DOCUMENT: &str = concat!(
    "<html><head><title>t</title></head><body>",
    "<p>I saw the house yesterday and the weather was rather cold.</p>",
    "<p>The houses in this town are large and painted white.</p>",
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

/// The deployment every test is served from: the shipped activity tree and
/// descriptors, with the caches under a directory of their own. Both the
/// classpath and the model handles are process-wide, so this is built once.
fn deployment() -> &'static (Arc<AppState>, TempDir) {
    static DEPLOYMENT: OnceLock<(Arc<AppState>, TempDir)> = OnceLock::new();
    DEPLOYMENT.get_or_init(|| {
        let root = repository_root();
        set_classpath_root(root.join("sme/desc"));
        let data = TempDir::new().expect("a data directory");
        let webapp = root.join("sme/src/main/webapp");
        let config = Config {
            listen: "127.0.0.1:0".to_string(),
            activities_dir: webapp.join("activities"),
            webapp_root: webapp,
            webapp_dist: None,
            analysis_dir: data.path().join("analysed"),
            upload_keep_dir: data.path().join("keep"),
            upload_temp_dir: data.path().join("temp"),
        };
        for directory in [
            &config.analysis_dir,
            &config.upload_keep_dir,
            &config.upload_temp_dir,
        ] {
            std::fs::create_dir_all(directory).expect("a cache directory");
        }
        let state = AppState::new(config).expect("the shipped deployment boots");
        (Arc::new(state), data)
    })
}

fn client() -> TestClient<impl Endpoint> {
    let state = deployment().0.clone();
    TestClient::new(routes(&state.config).data(state))
}

/// The page as a `file:` URL, so the whole-page endpoint fetches it without
/// a network.
fn page_url(directory: &TempDir, name: &str, page: &str) -> String {
    let path = directory.path().join(name);
    std::fs::write(&path, page).expect("the page is written");
    format!("file://{}", path.display())
}

/// One multipart body carrying a single file part, with its content type.
fn multipart(file_name: &str, content: &str) -> (String, Vec<u8>) {
    let boundary = "teaksta-test-boundary";
    let body = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"{file_name}\"\r\n\
         Content-Type: text/html\r\n\r\n\
         {content}\r\n\
         --{boundary}--\r\n"
    );
    (
        format!("multipart/form-data; boundary={boundary}"),
        body.into_bytes(),
    )
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

/// One span-map request over the inline document.
async fn spans(activity: &str, mode: &str) -> HashMap<String, String> {
    let response = client()
        .post("/api/enhance")
        .body_json(&serde_json::json!({
            "html": DOCUMENT,
            "activity": activity,
            "mode": mode,
        }))
        .send()
        .await;

    response.assert_status_is_ok();
    response.assert_content_type("application/json");
    let body = response.0.into_body().into_string().await.expect("a body");
    serde_json::from_str(&body).expect("a JSON object")
}

#[tokio::test]
async fn the_registry_offers_every_shipped_topic() {
    if !models_available() {
        return;
    }
    let response = client().get("/api/activities").send().await;

    response.assert_status_is_ok();
    let body = response.0.into_body().into_string().await.expect("a body");
    let registry: Value = serde_json::from_str(&body).expect("a JSON object");

    let names: Vec<&str> = registry["activities"]
        .as_array()
        .expect("an activity list")
        .iter()
        .map(|activity| activity["name"].as_str().expect("a name"))
        .collect();
    for topic in [
        "Adverbial",
        "Conjunctions",
        "InfiniteVerbs",
        "NegVerbs",
        "Object",
        "Subject",
        "Substantive",
        "SubstantivePlural",
        "SubstantiveSingular",
        "VerbConjugation",
    ] {
        assert!(names.contains(&topic), "{topic} is missing from {names:?}");
    }
    assert_eq!(registry["activities"][0]["label"], "Adverbiála");

    let modes: Vec<&str> = registry["modes"]
        .as_array()
        .expect("a mode list")
        .iter()
        .map(|mode| mode["name"].as_str().expect("a name"))
        .collect();
    assert_eq!(modes, vec!["colorize", "click", "mc", "cloze"]);
}

#[tokio::test]
async fn the_page_endpoint_answers_one_request() {
    if !models_available() {
        return;
    }
    let directory = TempDir::new().expect("a page directory");
    let url = page_url(&directory, "artihkal.html", DOCUMENT);

    let response = client()
        .get("/api/enhance")
        .query("url", &url)
        .query("activity", &"Substantive")
        .query("mode", &"colorize")
        .send()
        .await;

    response.assert_status_is_ok();
    let page = response.0.into_body().into_string().await.expect("a body");

    // One request, one enhanced document: nothing is served meanwhile.
    assert!(!page.contains("Vuorddes"), "{page}");
    assert!(page.contains(&format!("<base href=\"{url}\">")), "{page}");
    assert!(page.contains("<title>t</title>"), "{page}");
    assert!(!page.contains("<script"), "{page}");
    assert!(page.contains(">viesu</span> ikte."), "{page}");
    assert!(page.contains(">Viesut</span> leat"), "{page}");
    assert_eq!(page.matches("teaksta-token").count(), 2, "{page}");
}

/// Click is the one exercise the decoys reach: every other word of the page
/// is wrapped too, carrying `teaksta-token` without the topic's own class,
/// so the learner has something to pick wrongly. Colorize over the same page
/// carries the hits alone.
#[tokio::test]
async fn the_click_page_carries_decoys() {
    if !models_available() {
        return;
    }
    let directory = TempDir::new().expect("a page directory");
    let url = page_url(&directory, "artihkal.html", DOCUMENT);

    let mut pages: HashMap<&str, String> = HashMap::new();
    for mode in ["click", "colorize"] {
        let response = client()
            .get("/api/enhance")
            .query("url", &url)
            .query("activity", &"Substantive")
            .query("mode", &mode)
            .send()
            .await;
        response.assert_status_is_ok();
        pages.insert(
            mode,
            response.0.into_body().into_string().await.expect("a body"),
        );
    }
    let click = &pages["click"];
    let colorize = &pages["colorize"];

    // The two nouns are hits under both, and only under click do the other
    // five words of the page get a span of their own.
    for hit in [">viesu</span>", ">Viesut</span>"] {
        assert!(click.contains(hit), "{click}");
        assert!(colorize.contains(hit), "{colorize}");
    }
    assert_eq!(click.matches("teaksta-Substantive").count(), 2, "{click}");
    assert_eq!(
        colorize.matches("teaksta-Substantive").count(),
        2,
        "{colorize}"
    );

    for decoy in ["Mun", "oidnen", "ikte", "leat", "stuorrát"] {
        assert!(
            click.contains(&format!(">{decoy}</span>")),
            "click marked no decoy for {decoy}: {click}"
        );
        assert!(
            !colorize.contains(&format!(">{decoy}</span>")),
            "colorize marked {decoy}: {colorize}"
        );
    }

    // Seven words, two of them the topic's: five decoys and two hits.
    assert_eq!(click.matches("teaksta-token").count(), 7, "{click}");
    assert_eq!(colorize.matches("teaksta-token").count(), 2, "{colorize}");

    // A hit is never wrapped twice: the decoy gives way to the topic's own
    // span, which is the one carrying the class and the base form.
    assert_eq!(click.matches("lemma=\"viessu\"").count(), 2, "{click}");
    assert!(!click.contains("></span>"), "{click}");
}

#[tokio::test]
async fn the_span_endpoint_keys_spans_by_position() {
    if !models_available() {
        return;
    }
    let spans = spans("Substantive", "colorize").await;

    assert_eq!(spans.len(), 2, "{spans:#?}");
    for span in spans.values() {
        assert!(
            span.starts_with("<span class=\"teaksta-page\" style="),
            "{span}"
        );
        assert!(span.contains("teaksta-token"), "{span}");
        assert!(span.contains("teaksta-Substantive"), "{span}");
    }

    let ordered = ordered(&spans);
    assert!(ordered[0].contains(">viesu</span>"), "{}", ordered[0]);
    assert!(ordered[1].contains(">Viesut</span>"), "{}", ordered[1]);
}

#[tokio::test]
async fn each_mode_attaches_its_own_fields() {
    if !models_available() {
        return;
    }
    let colorize = ordered(&spans("Substantive", "colorize").await);
    let mc = ordered(&spans("Substantive", "mc").await);
    let cloze = ordered(&spans("Substantive", "cloze").await);

    assert!(!colorize[0].contains("distractors="), "{}", colorize[0]);
    assert!(!colorize[0].contains("possibleforms="), "{}", colorize[0]);
    assert!(colorize[0].contains(">viesu</span>"), "{}", colorize[0]);
    assert!(mc[0].contains("answer=\"viesu\""), "{}", mc[0]);
    assert!(mc[0].contains("distractors="), "{}", mc[0]);
    assert!(cloze[0].contains("possibleforms=\"viesu"), "{}", cloze[0]);
}

#[tokio::test]
async fn a_malformed_request_is_a_bad_request() {
    if !models_available() {
        return;
    }
    let client = client();

    client
        .get("/api/enhance")
        .query("url", &"http://example.org/a")
        .query("activity", &"Substantive")
        .send()
        .await
        .assert_status(StatusCode::BAD_REQUEST);

    client
        .get("/api/enhance")
        .query("url", &"http://example.org/a")
        .query("activity", &"Substantive")
        .query("mode", &"shuffle")
        .send()
        .await
        .assert_status(StatusCode::BAD_REQUEST);

    client
        .get("/api/enhance")
        .query("url", &"http://example.org/a")
        .query("activity", &"Kitchens")
        .query("mode", &"colorize")
        .send()
        .await
        .assert_status(StatusCode::BAD_REQUEST);

    client
        .post("/api/enhance")
        .body_json(&serde_json::json!({
            "html": DOCUMENT,
            "url": "http://example.org/a",
            "activity": "Substantive",
            "mode": "colorize",
        }))
        .send()
        .await
        .assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn a_second_run_is_answered_from_the_cache() {
    if !models_available() {
        return;
    }
    let first = spans("SubstantivePlural", "colorize").await;
    let cached = std::fs::read_dir(&deployment().0.config.analysis_dir)
        .expect("the cache directory")
        .count();
    let second = spans("SubstantivePlural", "colorize").await;

    assert_eq!(ordered(&first), ordered(&second));
    assert!(cached > 0, "the analysed document was not cached");
}

#[tokio::test]
async fn the_upload_gate_reads_the_uploaded_text() {
    if !models_available() {
        return;
    }
    let client = client();

    let (content_type, body) = multipart("sami.html", DOCUMENT);
    let accepted = client
        .post("/api/upload")
        .content_type(content_type)
        .header("content-length", body.len())
        .body(body)
        .send()
        .await;
    accepted.assert_status_is_ok();
    let body = accepted.0.into_body().into_string().await.expect("a body");
    let stored: Value = serde_json::from_str(&body).expect("a JSON object");
    let url = stored["url"].as_str().expect("a file url").to_string();
    assert!(url.starts_with("file:///"), "{url}");

    let (content_type, body) = multipart("english.html", ENGLISH_DOCUMENT);
    let refused = client
        .post("/api/upload")
        .content_type(content_type)
        .header("content-length", body.len())
        .body(body)
        .send()
        .await;
    refused.assert_status(StatusCode::BAD_REQUEST);
    refused
        .assert_json(serde_json::json!({ "error": "not-north-sami" }))
        .await;

    // The accepted text is reachable at the URL the response handed back.
    let enhanced = client
        .get("/api/enhance")
        .query("url", &url)
        .query("activity", &"Substantive")
        .query("mode", &"click")
        .send()
        .await;
    enhanced.assert_status_is_ok();
    let page = enhanced.0.into_body().into_string().await.expect("a body");
    assert!(page.contains("teaksta-token"), "{page}");
}
