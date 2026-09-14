//! What the access log writes, read back from a subscriber.
//!
//! It is a suite of its own rather than another case in the endpoint tests,
//! and for one reason: whether a log site is worth reaching is decided once
//! and remembered for the whole process. A test sharing a binary with several
//! hundred others that also serve requests would have that decided for it by
//! whichever of them got there first, with no subscriber installed — so the
//! line would be written or not depending on the order the tests ran in. One
//! test alone in a process has no such neighbour, and this needs no models.

use std::sync::Arc;

use poem::EndpointExt;
use poem::http::StatusCode;
use poem::test::TestClient;

use teaksta::context::Config;
use teaksta::server::api::{AppState, routes};

mod common;

use common::Recorded;

/// One line for every request: the client, the method, the path, the status
/// and how long it took — and never the query, which on this endpoint carries
/// the address a learner asked to read.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.access-log-fn/test]
#[tokio::test]
async fn every_request_writes_one_line_without_its_query() {
    let root = tempfile::tempdir().expect("a data directory");
    let config = Config {
        listen: "127.0.0.1:0".to_string(),
        webapp_dist: None,
        // The compiled-in registry, so no topics file has to be written: what
        // is under test is the layer over the map, not the map's contents.
        topics: None,
        analysis_dir: root.path().join("analysed"),
        upload_keep_dir: root.path().join("keep"),
        upload_temp_dir: root.path().join("temp"),
        // The log resolves a client the same way the limiter does, so the
        // header proves both that it is read and that the line carries it.
        trust_proxy: true,
        rate_limit: None,
        max_page_bytes: 5 * 1024 * 1024,
    };
    let state = Arc::new(AppState::new(config).expect("the state boots"));
    let client = TestClient::new(routes(&state.config).data(state));

    let written = Recorded::default();
    let _recording = written.recording();

    // A topic the registry has not loaded, so the request reaches a handler,
    // is refused there, and fetches nothing — and the address it named is in
    // the query, where the log must not look.
    client
        .get("/api/enhance")
        .query("url", &"http://example.org/artihkal")
        .query("activity", &"Kitchens")
        .query("mode", &"colorize")
        .header("x-forwarded-for", "9.9.9.9, 203.0.113.77")
        .send()
        .await
        .assert_status(StatusCode::BAD_REQUEST);

    let log = written.read();
    let lines: Vec<&str> = log.lines().filter(|line| line.contains("access")).collect();
    assert_eq!(lines.len(), 1, "one request, one line:\n{log}");

    let line = lines[0];
    for field in [
        "client=203.0.113.77",
        "method=GET",
        "path=/api/enhance",
        "status=400",
        "duration_ms=",
    ] {
        assert!(line.contains(field), "{field} is missing from {line}");
    }
    assert!(
        !line.contains("example.org"),
        "the address a learner asked for reached the log: {line}"
    );
}
