//! What the startup probe costs, read off the server's own log. It runs only
//! when TEAKSTA_BUNDLE and TEAKSTA_GENERATOR are set; without the models it
//! reports itself skipped and passes.
//!
//! It is a suite of its own rather than another case in `health.rs`, and for
//! the reason `access_log.rs` is one: whether a log site is worth reaching is
//! decided once and remembered for the whole process, so a test sharing a
//! binary with others that reach the same site — `health.rs` asks the deep
//! check four times over — would have that decided for it by whichever of
//! them got there first, with no subscriber installed. One test alone in a
//! process has no such neighbour.
//!
//! Reading the log is what makes the assertion an assertion. The endpoint is
//! supposed to answer the same thing whether it analysed or remembered — a
//! caller cannot tell the two apart, and that is deliberate — so how long the
//! second reply took would be the only other evidence, and a wall clock on a
//! machine running the rest of this suite beside it is not evidence.

use std::sync::Arc;

use poem::Endpoint;
use poem::EndpointExt;
use poem::test::TestClient;
use tempfile::TempDir;

use teaksta::context::Config;
use teaksta::morpho::{BUNDLE_ENV, GENERATOR_ENV};
use teaksta::server::api::{AppState, routes};

mod common;

use common::Recorded;

/// The line the deep check writes when it has reached the models and they
/// answered. One of these is one analysis.
const ANALYSED: &str = "The models answered a sentence";

fn models_available() -> bool {
    let ok = std::env::var(BUNDLE_ENV).is_ok() && std::env::var(GENERATOR_ENV).is_ok();
    if !ok {
        eprintln!("skipped: {BUNDLE_ENV}/{GENERATOR_ENV} not set");
    }
    ok
}

/// How many times the models have been reached, as the log records it.
fn analyses(log: &str) -> usize {
    log.lines().filter(|line| line.contains(ANALYSED)).count()
}

/// A deployment serving the compiled-in registry with its caches under a
/// directory of its own.
fn served(root: &TempDir) -> TestClient<impl Endpoint> {
    let config = Config {
        listen: "127.0.0.1:0".to_string(),
        webapp_dist: None,
        topics: None,
        analysis_dir: root.path().join("analysed"),
        upload_keep_dir: Some(root.path().join("keep")),
        upload_temp_dir: root.path().join("temp"),
        trust_proxy: false,
        // The probes stand outside the limiter, which `api_tests.rs` shows
        // over a limited deployment; what is under test here is what the deep
        // check costs, so nothing counts it.
        rate_limit: None,
        max_page_bytes: 5 * 1024 * 1024,
        azure: None,
    };
    for directory in [&config.analysis_dir, &config.upload_temp_dir]
        .into_iter()
        .chain(&config.upload_keep_dir)
    {
        std::fs::create_dir_all(directory).expect("a cache directory");
    }
    let state = Arc::new(AppState::new(config).expect("the shipped deployment boots"));
    TestClient::new(routes(&state.config).data(state))
}

/// The startup probe proves the models once and answers from the verdict
/// afterwards.
///
/// The first ask reaches them, which is the whole point of a probe that runs
/// at boot: a deployment whose models were not baked in, or were baked in at
/// the wrong path, is caught before the pod is ever sent traffic. Every ask
/// after that is a flag read, which is the other half — the address is
/// anonymous and reachable by whoever finds it, and an endpoint that analysed
/// on demand would be an unmetered analysis endpoint standing outside the
/// rate limit.
///
/// The replies are byte-identical, so nothing about which of the two answered
/// reaches a caller.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.health-deep-fn/test]
#[tokio::test]
async fn the_startup_probe_proves_the_models_once() {
    if !models_available() {
        return;
    }
    let root = TempDir::new().expect("a data directory");
    let client = served(&root);

    let written = Recorded::default();
    let _recording = written.recording();

    let first = client.get("/api/health/deep").send().await;
    first.assert_status_is_ok();
    let first_body = first.0.into_body().into_string().await.expect("a body");

    assert_eq!(
        analyses(&written.read()),
        1,
        "the first ask must reach the models:\n{}",
        written.read()
    );
    let answered: serde_json::Value = serde_json::from_str(&first_body).expect("a JSON object");
    assert_eq!(answered["status"], "ok", "{first_body}");
    assert_eq!(answered["models"], "loaded", "{first_body}");

    // However many times it is asked again, the models are not reached again.
    for asked in 2..=8 {
        let again = client.get("/api/health/deep").send().await;
        again.assert_status_is_ok();
        let body = again.0.into_body().into_string().await.expect("a body");

        assert_eq!(body, first_body, "ask {asked} answered something else");
        assert_eq!(
            analyses(&written.read()),
            1,
            "ask {asked} reached the models again:\n{}",
            written.read()
        );
    }
}
