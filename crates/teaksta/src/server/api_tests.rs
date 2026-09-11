//! The endpoint layer's own decisions: what a mode is, where a page comes
//! from, and what the registry offers. The analysis behind the endpoints
//! needs the real models and is exercised in `tests/pipeline_models.rs`.

use super::*;

use std::path::{Path, PathBuf};

use poem::Endpoint;
use poem::test::{TestClient, TestResponse};

/// The smallest activity descriptor the registry loads. It declares no
/// language, so no pipeline is built for it.
const DESCRIPTOR: &str = "<activity enabled=\"yes\"><server-cfg></server-cfg></activity>";

/// The two files a built web client is recognised by: the document every
/// client route is answered with, and one asset it loads. The real bundle is
/// what `dx bundle --platform web` leaves behind; nothing here builds it.
const CLIENT_INDEX: &str =
    "<!DOCTYPE html><html><head><title>Teaksta</title></head><body></body></html>";
const CLIENT_ASSET: &str = ".teaksta-page { color: rebeccapurple; }";

fn webapp_with(names: &[&str]) -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("temp dir");
    for name in names {
        let directory = root.path().join("activities").join(name);
        std::fs::create_dir_all(&directory).expect("activity directory");
        std::fs::write(directory.join("activity.xml"), DESCRIPTOR).expect("activity.xml");
    }
    root
}

fn config_for(root: &Path) -> Config {
    Config {
        listen: "127.0.0.1:0".to_string(),
        webapp_root: root.to_path_buf(),
        webapp_dist: None,
        // The descriptors these activities name are absent anyway, so no
        // pipeline is built and nothing reaches the tree.
        classpath_root: root.join("desc"),
        activities_dir: root.join("activities"),
        analysis_dir: root.join("analysed"),
        upload_keep_dir: root.join("keep"),
        upload_temp_dir: root.join("temp"),
    }
}

/// One page stored where an accepted upload is stored, addressed the way the
/// upload endpoint addresses it.
fn stored_upload(config: &Config, page: &str) -> String {
    std::fs::create_dir_all(&config.upload_temp_dir).expect("the upload directory");
    let stored = config.upload_temp_dir.join("aB3xY9zQ1w");
    std::fs::write(&stored, page).expect("the stored page");
    upload::file_url(&stored).expect("a file url")
}

/// A built web client under the deployment root, as a bundle would be laid
/// out: one document and one asset beside it.
fn client_bundle_under(root: &Path) -> PathBuf {
    let dist = root.join("public");
    std::fs::create_dir_all(dist.join("assets")).expect("the asset directory");
    std::fs::write(dist.join("index.html"), CLIENT_INDEX).expect("index.html");
    std::fs::write(dist.join("assets").join("teaksta.css"), CLIENT_ASSET).expect("the asset");
    dist
}

#[test]
fn modes_are_the_four_exercise_names() {
    let names: Vec<&str> = Mode::ALL.into_iter().map(Mode::name).collect();

    assert_eq!(names, vec!["colorize", "click", "mc", "cloze"]);
    for mode in Mode::ALL {
        assert_eq!(Mode::parse(mode.name()), Some(mode));
    }
    assert_eq!(Mode::parse("Colorize"), None);
    assert_eq!(Mode::parse(""), None);
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+3/test]
#[test]
fn a_bare_host_is_taken_as_http() {
    assert_eq!(
        page_url("example.org/artihkal").unwrap().as_str(),
        "http://example.org/artihkal"
    );
    assert_eq!(
        page_url(" https://example.org/a ").unwrap().as_str(),
        "https://example.org/a"
    );
    assert_eq!(
        page_url("file:///srv/upload/abc").unwrap().as_str(),
        "file:///srv/upload/abc"
    );
    assert!(page_url("http://").is_err());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn+5/test]
#[test]
fn each_page_source_gets_its_own_key() {
    let page = "<html><body><p>Mun oidnen viesu.</p></body></html>";

    assert_eq!(cache_key(page), cache_key(page));
    assert_ne!(cache_key(page), cache_key("http://example.org/a"));
    // The key is a filename component, so it carries no separator and
    // nothing else a path is read for.
    for subject in [page, "http://example.org/a/b", "", "../../etc/passwd"] {
        let key = cache_key(subject);
        assert!(
            key.starts_with(&format!("v{CACHE_FORMAT_VERSION}-")),
            "{key}"
        );
        assert_eq!(key.len(), 35, "{key}");
        assert!(
            key.strip_prefix(&format!("v{CACHE_FORMAT_VERSION}-"))
                .is_some_and(|digest| digest.chars().all(|c| c.is_ascii_hexdigit())),
            "{key}"
        );
    }
}

// The key is a stated value, not whatever the toolchain hashes to this
// month: a deployment that upgrades its compiler keeps reaching the analyses
// it has already paid for.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn+5/test]
#[test]
fn the_key_of_a_page_is_fixed() {
    assert_eq!(
        cache_key("http://example.org/artihkal"),
        "v1-011989fb2c268625fd589de14e3d74f6"
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.init-fn+2/test]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.activities-fn/test]
#[test]
fn the_registry_lists_each_activity_directory() {
    let root = webapp_with(&["Substantive", "Adverbial"]);

    let state = AppState::new(config_for(root.path())).expect("the state boots");

    let names: Vec<&str> = state
        .topics
        .iter()
        .map(|topic| topic.name.as_str())
        .collect();
    assert_eq!(names, vec!["Adverbial", "Substantive"]);
    assert_eq!(
        state.topics[0].label.as_deref(),
        Some("Adverbi\u{e1}la"),
        "the North Sámi name reaches the picker"
    );
    assert!(state.knows_topic("Substantive"));
    assert!(!state.knows_topic("Verbs"));
}

/// A router over a deployment holding two topics and no pipeline, which is
/// every decision an endpoint makes before it reaches the analyser.
fn served(config: Config) -> TestClient<impl Endpoint> {
    let state = Arc::new(AppState::new(config).expect("the state boots"));
    TestClient::new(routes(&state.config).data(state))
}

/// One whole-page request, with the topic and the exercise held fixed so that
/// the address is the only thing under test.
async fn enhanced<E: Endpoint>(client: &TestClient<E>, address: &str) -> TestResponse {
    client
        .get("/api/enhance")
        .query("url", &address)
        .query("activity", &"Substantive")
        .query("mode", &"colorize")
        .send()
        .await
}

/// One span-map request over a body the caller composed.
async fn spans<E: Endpoint>(client: &TestClient<E>, body: &serde_json::Value) -> TestResponse {
    client.post("/api/enhance").body_json(body).send().await
}

/// One block request over a body the caller composed.
async fn blocks<E: Endpoint>(client: &TestClient<E>, body: &serde_json::Value) -> TestResponse {
    client
        .post("/api/enhance/blocks")
        .body_json(body)
        .send()
        .await
}

/// A deployment carrying no web client, which is the API-only one.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.index-fn+2/test]
#[tokio::test]
async fn the_index_lists_every_endpoint() {
    let root = webapp_with(&["Substantive"]);

    let response = served(config_for(root.path())).get("/").send().await;

    response.assert_status_is_ok();
    let body = response.0.into_body().into_string().await.expect("a body");
    for path in [
        "GET  /api/activities",
        "GET  /api/enhance",
        "POST /api/enhance",
        "POST /api/enhance/blocks",
        "POST /api/upload",
    ] {
        assert!(body.contains(path), "{path} is missing from {body}");
    }
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.index-fn+2/test]
#[tokio::test]
async fn a_configured_client_answers_the_root() {
    let root = webapp_with(&["Substantive"]);
    let mut config = config_for(root.path());
    config.webapp_dist = Some(client_bundle_under(root.path()));

    let response = served(config).get("/").send().await;

    response.assert_status_is_ok();
    let body = response.0.into_body().into_string().await.expect("a body");
    assert_eq!(body, CLIENT_INDEX);
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.index-fn+2/test]
#[tokio::test]
async fn a_client_route_is_answered_by_the_document() {
    let root = webapp_with(&["Substantive"]);
    let mut config = config_for(root.path());
    config.webapp_dist = Some(client_bundle_under(root.path()));
    let client = served(config);

    // The bundle has no file for either, so both are the client's own
    // routes and reach its router rather than a 404.
    for path in ["/Substantive/colorize", "/deep/client/route"] {
        let response = client.get(path).send().await;

        response.assert_status_is_ok();
        let body = response.0.into_body().into_string().await.expect("a body");
        assert_eq!(body, CLIENT_INDEX, "{path} was not answered by the client");
    }
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.index-fn+2/test]
#[tokio::test]
async fn an_asset_is_served_from_the_bundle() {
    let root = webapp_with(&["Substantive"]);
    let mut config = config_for(root.path());
    config.webapp_dist = Some(client_bundle_under(root.path()));

    let response = served(config).get("/assets/teaksta.css").send().await;

    response.assert_status_is_ok();
    let body = response.0.into_body().into_string().await.expect("a body");
    assert_eq!(body, CLIENT_ASSET);
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.activities-fn/test]
#[tokio::test]
async fn the_api_answers_before_the_client() {
    let root = webapp_with(&["Substantive"]);
    let mut config = config_for(root.path());
    config.webapp_dist = Some(client_bundle_under(root.path()));

    let response = served(config).get("/api/activities").send().await;

    response.assert_status_is_ok();
    let body = response.0.into_body().into_string().await.expect("a body");
    assert!(body.contains("\"Substantive\""), "{body}");
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.activities-fn/test]
#[tokio::test]
async fn the_registry_answers_topics_and_modes() {
    let root = webapp_with(&["Substantive", "Adverbial"]);

    let response = served(config_for(root.path()))
        .get("/api/activities")
        .send()
        .await;

    response.assert_status_is_ok();
    response
        .assert_json(serde_json::json!({
            "activities": [
                { "name": "Adverbial", "label": "Adverbi\u{e1}la", "enabled": true },
                { "name": "Substantive", "label": "Substantiivvat", "enabled": true },
            ],
            "modes": [
                { "name": "colorize", "label": "Geah\u{10d}a ivdnejuvvon s\u{e1}niid." },
                { "name": "click", "label": "Coahkkal rivttes s\u{e1}niid!" },
                { "name": "mc", "label": "V\u{e1}llje rivttes s\u{e1}niid!" },
                { "name": "cloze", "label": "\u{10c}\u{e1}le rivttes s\u{e1}niid!" },
            ],
        }))
        .await;
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+3/test]
#[tokio::test]
async fn the_page_endpoint_needs_all_three() {
    let root = webapp_with(&["Substantive"]);
    let client = served(config_for(root.path()));

    for query in [
        "activity=Substantive&mode=colorize",
        "url=http://example.org/a&mode=colorize",
        "url=http://example.org/a&activity=Substantive",
        "url=http://example.org/a&activity=Substantive&mode=shuffle",
        "url=http://example.org/a&activity=Kitchens&mode=colorize",
        "url=%20&activity=Substantive&mode=colorize",
    ] {
        client
            .get(format!("/api/enhance?{query}"))
            .send()
            .await
            .assert_status(StatusCode::BAD_REQUEST);
    }
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn+5/test]
#[tokio::test]
async fn the_span_endpoint_needs_one_source() {
    let root = webapp_with(&["Substantive"]);
    let client = served(config_for(root.path()));

    for body in [
        serde_json::json!({ "activity": "Substantive", "mode": "colorize" }),
        serde_json::json!({
            "html": "<p>a</p>",
            "url": "http://example.org/a",
            "activity": "Substantive",
            "mode": "colorize",
        }),
        serde_json::json!({ "html": "<p>a</p>", "mode": "colorize" }),
        serde_json::json!({ "html": "<p>a</p>", "activity": "Substantive" }),
        serde_json::json!({ "html": "<p>a</p>", "activity": "Kitchens", "mode": "mc" }),
    ] {
        spans(&client, &body)
            .await
            .assert_status(StatusCode::BAD_REQUEST);
    }
}

/// An enhancement request whose page weighs the given number of bytes, which
/// is the only member of the body that grows. Both POST endpoints take the
/// same four members, so both are weighed with this.
fn span_body_of(page_bytes: usize) -> String {
    let page = "a".repeat(page_bytes);
    format!("{{\"html\":\"{page}\",\"activity\":\"Kitchens\",\"mode\":\"colorize\"}}")
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn+5/test]
#[tokio::test]
async fn an_oversized_span_body_is_refused() {
    let root = webapp_with(&["Substantive"]);
    let client = served(config_for(root.path()));
    let body = span_body_of(MAX_ENHANCE_BODY);

    // A body that declares its weight, which is the ordinary request.
    client
        .post("/api/enhance")
        .content_type("application/json")
        .header("content-length", body.len())
        .body(body.clone())
        .send()
        .await
        .assert_status(StatusCode::PAYLOAD_TOO_LARGE);

    // And one that declares nothing, which is what a chunked request is: the
    // read itself is bounded, so it is refused at the same weight rather than
    // being buffered for as long as it streams.
    client
        .post("/api/enhance")
        .content_type("application/json")
        .body(body)
        .send()
        .await
        .assert_status(StatusCode::PAYLOAD_TOO_LARGE);
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn+5/test]
#[tokio::test]
async fn a_body_under_the_cap_reaches_the_handler() {
    let root = webapp_with(&["Substantive"]);
    let body = span_body_of(1024 * 1024);

    let response = served(config_for(root.path()))
        .post("/api/enhance")
        .content_type("application/json")
        .header("content-length", body.len())
        .body(body)
        .send()
        .await;

    // Only the handler knows `Kitchens` is not a topic the registry loaded,
    // so a 400 says the megabyte passed the cap and was parsed.
    response.assert_status(StatusCode::BAD_REQUEST);
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn+5/test]
#[tokio::test]
async fn a_body_that_is_not_json_is_refused() {
    let root = webapp_with(&["Substantive"]);
    let client = served(config_for(root.path()));
    let body = "html=%3Cp%3Ea%3C%2Fp%3E&activity=Substantive&mode=colorize";

    for content_type in ["application/x-www-form-urlencoded", "text/plain"] {
        client
            .post("/api/enhance")
            .content_type(content_type)
            .body(body)
            .send()
            .await
            .assert_status(StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.blocks-fn/test]
#[tokio::test]
async fn the_block_endpoint_needs_one_source() {
    let root = webapp_with(&["Substantive"]);
    let client = served(config_for(root.path()));

    for body in [
        serde_json::json!({ "activity": "Substantive", "mode": "colorize" }),
        serde_json::json!({
            "html": "<p>a</p>",
            "url": "http://example.org/a",
            "activity": "Substantive",
            "mode": "colorize",
        }),
        serde_json::json!({ "html": "<p>a</p>", "mode": "colorize" }),
        serde_json::json!({ "html": "<p>a</p>", "activity": "Substantive" }),
        serde_json::json!({ "html": "<p>a</p>", "activity": "Substantive", "mode": "shuffle" }),
        serde_json::json!({ "html": "<p>a</p>", "activity": "Kitchens", "mode": "mc" }),
    ] {
        blocks(&client, &body)
            .await
            .assert_status(StatusCode::BAD_REQUEST);
    }
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.blocks-fn/test]
#[tokio::test]
async fn an_oversized_block_body_is_refused() {
    let root = webapp_with(&["Substantive"]);
    let client = served(config_for(root.path()));
    let body = span_body_of(MAX_ENHANCE_BODY);

    // The same cap the span endpoint applies, declared and undeclared alike.
    client
        .post("/api/enhance/blocks")
        .content_type("application/json")
        .header("content-length", body.len())
        .body(body.clone())
        .send()
        .await
        .assert_status(StatusCode::PAYLOAD_TOO_LARGE);

    client
        .post("/api/enhance/blocks")
        .content_type("application/json")
        .body(body)
        .send()
        .await
        .assert_status(StatusCode::PAYLOAD_TOO_LARGE);

    // A megabyte passes the cap and is parsed: only the handler knows
    // `Kitchens` is not a topic the registry loaded.
    let under = span_body_of(1024 * 1024);
    client
        .post("/api/enhance/blocks")
        .content_type("application/json")
        .header("content-length", under.len())
        .body(under)
        .send()
        .await
        .assert_status(StatusCode::BAD_REQUEST);
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.blocks-fn/test]
#[tokio::test]
async fn a_block_body_must_announce_json() {
    let root = webapp_with(&["Substantive"]);
    let client = served(config_for(root.path()));
    let body = "html=%3Cp%3Ea%3C%2Fp%3E&activity=Substantive&mode=colorize";

    for content_type in ["application/x-www-form-urlencoded", "text/plain"] {
        client
            .post("/api/enhance/blocks")
            .content_type(content_type)
            .body(body)
            .send()
            .await
            .assert_status(StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.blocks-fn/test]
#[tokio::test]
async fn the_block_endpoint_refuses_them_too() {
    let root = webapp_with(&["Substantive"]);
    let client = served(config_for(root.path()));

    for address in ["http://169.254.169.254/", "file:///etc/passwd"] {
        let body = serde_json::json!({
            "url": address,
            "activity": "Substantive",
            "mode": "colorize",
        });

        blocks(&client, &body)
            .await
            .assert_status(StatusCode::BAD_REQUEST);
    }
}

/// The block path sits under the span path, and neither takes the other's
/// requests: the span endpoint answers `/api/enhance` alone and the block one
/// answers only its own address, under POST alone.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.blocks-fn/test]
#[tokio::test]
async fn the_block_path_is_its_own() {
    let root = webapp_with(&["Substantive"]);
    let client = served(config_for(root.path()));

    client
        .get("/api/enhance/blocks")
        .send()
        .await
        .assert_status(StatusCode::METHOD_NOT_ALLOWED);

    // The span endpoint is untouched by the sibling beneath it.
    spans(
        &client,
        &serde_json::json!({ "html": "<p>a</p>", "activity": "Kitchens", "mode": "colorize" }),
    )
    .await
    .assert_status(StatusCode::BAD_REQUEST);
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.index-fn+2/test]
#[tokio::test]
async fn a_panicking_handler_is_answered_rather_than_dropped() {
    let root = webapp_with(&["Substantive"]);

    let response = served(config_for(root.path()))
        .get("/api/panic")
        .send()
        .await;

    response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
}

/// The addresses this deployment will not fetch, and none of them costs a
/// socket to turn away: each names its IP or its path outright, so no name is
/// looked up and no connection is tried. A 400 rather than the 502 a
/// connection that was made and failed would answer with is what says so.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+3/test]
#[tokio::test]
async fn a_refused_address_reaches_nothing() {
    let root = webapp_with(&["Substantive"]);
    let elsewhere = tempfile::tempdir().expect("temp dir");
    let planted = elsewhere.path().join("secret.html");
    std::fs::write(&planted, "<p>secret</p>").expect("the secret");
    let client = served(config_for(root.path()));

    let refused = [
        // The private network, in both families.
        "http://127.0.0.1/a".to_string(),
        "http://127.0.0.1:8080/a".to_string(),
        "http://[::1]/a".to_string(),
        // The instance-metadata address of every cloud there is.
        "http://169.254.169.254/latest/meta-data/".to_string(),
        "http://10.0.0.7/a".to_string(),
        "http://192.168.1.1/a".to_string(),
        "http://172.16.9.9/a".to_string(),
        "http://0.0.0.0/a".to_string(),
        // The same loopback address in the spellings a URL parser accepts.
        "http://2130706433/a".to_string(),
        "http://0177.0.0.1/a".to_string(),
        "http://localhost/a".to_string(),
        // Files this deployment does not serve.
        "file:///etc/passwd".to_string(),
        "file:///etc/shadow".to_string(),
        upload::file_url(&planted).expect("a file url"),
        // Schemes that are none of the three.
        "ftp://example.org/a".to_string(),
        "gopher://example.org/a".to_string(),
    ];

    for address in refused {
        enhanced(&client, &address)
            .await
            .assert_status(StatusCode::BAD_REQUEST);
    }
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn+5/test]
#[tokio::test]
async fn the_span_endpoint_refuses_them_too() {
    let root = webapp_with(&["Substantive"]);
    let client = served(config_for(root.path()));

    for address in ["http://169.254.169.254/", "file:///etc/passwd"] {
        let body = serde_json::json!({
            "url": address,
            "activity": "Substantive",
            "mode": "colorize",
        });

        spans(&client, &body)
            .await
            .assert_status(StatusCode::BAD_REQUEST);
    }
}

/// The counterpart: a stored upload clears the confinement and reaches the
/// analyser, which this deployment has no pipeline for. A 500 rather than a
/// 400 is what says the address was accepted.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+3/test]
#[tokio::test]
async fn a_stored_upload_clears_the_confinement() {
    let root = webapp_with(&["Substantive"]);
    let config = config_for(root.path());
    let address = stored_upload(
        &config,
        "<html><body><p>Mun oidnen viesu.</p></body></html>",
    );
    let client = served(config);

    let response = enhanced(&client, &address).await;

    response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
    let body = response.0.into_body().into_string().await.expect("a body");
    assert!(body.contains("no pipeline is registered"), "{body}");
}

/// A path this deployment serves that holds nothing is unreadable, not
/// forbidden: the caller is told the far end failed rather than that they
/// asked for something they may not have.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+3/test]
#[tokio::test]
async fn a_swept_upload_is_unreadable_rather_than_refused() {
    let root = webapp_with(&["Substantive"]);
    let config = config_for(root.path());
    std::fs::create_dir_all(&config.upload_temp_dir).expect("the upload directory");
    let address = upload::file_url(&config.upload_temp_dir.join("swept")).expect("a file url");
    let client = served(config);

    enhanced(&client, &address)
        .await
        .assert_status(StatusCode::BAD_GATEWAY);
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+3/test]
#[tokio::test]
async fn the_retired_paths_answer_nothing() {
    let root = webapp_with(&["Substantive"]);
    let client = served(config_for(root.path()));

    for path in [
        "/WERTiServlet?url=a&activity=b",
        "/UploadDownloadFileServlet",
    ] {
        client
            .get(path)
            .send()
            .await
            .assert_status(StatusCode::NOT_FOUND);
    }
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn+3/test]
#[tokio::test]
async fn an_upload_without_a_file_is_refused() {
    let root = webapp_with(&["Substantive"]);
    let boundary = "teaksta-unit-boundary";
    let body = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"keep\"\r\n\r\n\
         true\r\n\
         --{boundary}--\r\n"
    );

    let response = served(config_for(root.path()))
        .post("/api/upload")
        .content_type(format!("multipart/form-data; boundary={boundary}"))
        .header("content-length", body.len())
        .body(body)
        .send()
        .await;

    response.assert_status(StatusCode::BAD_REQUEST);
    response
        .assert_json(serde_json::json!({ "error": "no-file" }))
        .await;
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.init-fn+2/test]
#[test]
fn a_missing_activity_tree_fails_the_boot() {
    let root = tempfile::tempdir().expect("temp dir");

    let Err(error) = AppState::new(config_for(root.path())) else {
        panic!("an absent activity tree must abort the boot");
    };

    assert!(
        format!("{error:#}").contains("scanning activities"),
        "{error:#}"
    );
}
