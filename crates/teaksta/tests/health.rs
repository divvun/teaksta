//! What the cluster's probes are answered with, from outside the crate — the
//! view the kubelet has of a pod.
//!
//! These need no models and say what a deployment that has none answers, which
//! is the state every build without them is in. What the deep check answers
//! once the models are there, and what it costs to ask it twice, is
//! `health_models.rs`'s to say.

use std::sync::Arc;
use std::time::Duration;

use poem::Endpoint;
use poem::EndpointExt;
use poem::http::StatusCode;
use poem::test::{TestClient, TestResponse};
use tempfile::TempDir;

use teaksta::context::{Config, RateLimit};
use teaksta::morpho::{BUNDLE_ENV, GENERATOR_ENV};
use teaksta::server::api::{AppState, routes};

/// A limit two requests wide that replenishes once an hour, so what a test
/// sees is decided by the requests it makes and never by how long the machine
/// took to make them.
const TEST_LIMIT: RateLimit = RateLimit {
    burst: 2,
    count: 1,
    period: Duration::from_secs(60 * 60),
};

/// A deployment serving the compiled-in registry, caches under a directory of
/// its own, limited or not as the caller asks. The state comes back beside the
/// client because the liveness probe answers for what the registry loaded, and
/// a test that hardcoded that number would be a test about the shipped topic
/// list rather than about the probe.
fn served(root: &TempDir, limit: Option<RateLimit>) -> (Arc<AppState>, TestClient<impl Endpoint>) {
    let config = Config {
        listen: "127.0.0.1:0".to_string(),
        webapp_dist: None,
        topics: None,
        analysis_dir: root.path().join("analysed"),
        upload_keep_dir: root.path().join("keep"),
        upload_temp_dir: root.path().join("temp"),
        trust_proxy: false,
        rate_limit: limit,
        max_page_bytes: 5 * 1024 * 1024,
    };
    for directory in [
        &config.analysis_dir,
        &config.upload_keep_dir,
        &config.upload_temp_dir,
    ] {
        std::fs::create_dir_all(directory).expect("a cache directory");
    }
    let state = Arc::new(AppState::new(config).expect("the shipped deployment boots"));
    let client = TestClient::new(routes(&state.config).data(state.clone()));
    (state, client)
}

/// One request that reaches a handler and is refused there, so anything but a
/// 400 means the limiter answered instead of the endpoint. The topic is not
/// one the registry loaded, which only the handler knows, and no address is
/// fetched on the way to finding out.
async fn asked<E: Endpoint>(client: &TestClient<E>) -> TestResponse {
    client
        .get("/api/enhance")
        .query("url", &"http://example.org/a")
        .query("activity", &"Kitchens")
        .query("mode", &"colorize")
        .send()
        .await
}

/// The liveness probe reads what the process already holds and nothing else,
/// so it answers a deployment with no models beside it exactly as it answers
/// one that has them. This is the probe that decides whether the pod lives,
/// and what it must never do is depend on the work the pod is busy with.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.health-fn/test]
#[tokio::test]
async fn the_liveness_probe_answers_from_memory() {
    let root = TempDir::new().expect("a data directory");
    let (state, client) = served(&root, None);
    let loaded = state.registry.topics().len();

    let response = client.get("/api/health").send().await;

    response.assert_status_is_ok();
    response
        .assert_json(serde_json::json!({ "status": "ok", "topics": loaded }))
        .await;
    assert!(loaded > 0, "the shipped registry carries topics");
}

/// A deployment built without models serves the registry and refuses every
/// analysis, and the startup probe is where it says so: 503, naming the
/// variables that are not set, rather than a bare failure an operator has to
/// go and interpret.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.health-deep-fn/test]
#[tokio::test]
async fn the_startup_probe_names_the_missing_models() {
    // Either variable being set makes this a different deployment from the one
    // under test, and the one `health_models.rs` speaks for.
    if std::env::var_os(BUNDLE_ENV).is_some() || std::env::var_os(GENERATOR_ENV).is_some() {
        eprintln!("skipped: {BUNDLE_ENV}/{GENERATOR_ENV} are set");
        return;
    }
    let root = TempDir::new().expect("a data directory");
    let (_state, client) = served(&root, None);

    let response = client.get("/api/health/deep").send().await;

    response.assert_status(StatusCode::SERVICE_UNAVAILABLE);
    let body = response.0.into_body().into_string().await.expect("a body");
    for named in [BUNDLE_ENV, GENERATOR_ENV] {
        assert!(body.contains(named), "{named} is missing from {body}");
    }
    assert!(body.contains("failed"), "{body}");
}

/// Neither probe is counted. The kubelet asks from the pod network, so every
/// probe of every pod on a node presents as one address — and an allowance
/// spent by the analysis traffic from that address would take the requests
/// that decide whether this pod lives down with it.
///
/// The allowance is spent first, and then far more probes are asked than it
/// holds. The liveness probe answers every one; the startup probe answers
/// whatever its models let it — a 200 where they are configured, a 503 where
/// they are not — and never the 429 that would fail a probe.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.health-fn/test]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.health-deep-fn/test]
#[tokio::test]
async fn no_probe_is_ever_rate_limited() {
    let root = TempDir::new().expect("a data directory");
    let (_state, client) = served(&root, Some(TEST_LIMIT));

    for _ in 0..TEST_LIMIT.burst {
        asked(&client).await.assert_status(StatusCode::BAD_REQUEST);
    }
    // and the allowance really is spent, so what follows is asked over it.
    asked(&client)
        .await
        .assert_status(StatusCode::TOO_MANY_REQUESTS);

    for _ in 0..16 {
        client.get("/api/health").send().await.assert_status_is_ok();
    }

    for _ in 0..4 {
        let status = client.get("/api/health/deep").send().await.0.status();

        assert_ne!(
            status,
            StatusCode::TOO_MANY_REQUESTS,
            "a probe the limiter answered is a probe that failed"
        );
        assert!(
            matches!(status, StatusCode::OK | StatusCode::SERVICE_UNAVAILABLE),
            "{status}"
        );
    }
}
