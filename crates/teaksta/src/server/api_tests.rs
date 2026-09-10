//! The endpoint layer's own decisions: what a mode is, where a page comes
//! from, and what the registry offers. The analysis behind the endpoints
//! needs the real models and is exercised in `tests/pipeline_models.rs`.

use super::*;

use std::path::Path;

use poem::Endpoint;
use poem::test::TestClient;

/// The smallest activity descriptor the registry loads. It declares no
/// language, so no pipeline is built for it.
const DESCRIPTOR: &str = "<activity enabled=\"yes\"><server-cfg></server-cfg></activity>";

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
        activities_dir: root.join("activities"),
        analysis_dir: root.join("analysed"),
        upload_keep_dir: root.join("keep"),
        upload_temp_dir: root.join("temp"),
    }
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

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+2/test]
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

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn+4/test]
#[test]
fn each_page_source_gets_its_own_key() {
    let page = "<html><body><p>Mun oidnen viesu.</p></body></html>";

    assert_eq!(cache_key(page), cache_key(page));
    assert_ne!(cache_key(page), cache_key("http://example.org/a"));
    assert_eq!(cache_key(page).len(), 16);
    // The key is a filename component, so it carries no separator.
    assert!(!cache_key("http://example.org/a/b").contains('/'));
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
fn served(root: &Path) -> TestClient<impl Endpoint> {
    let state = AppState::new(config_for(root)).expect("the state boots");
    TestClient::new(routes().data(Arc::new(state)))
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.index-fn/test]
#[tokio::test]
async fn the_index_lists_every_endpoint() {
    let root = webapp_with(&["Substantive"]);

    let response = served(root.path()).get("/").send().await;

    response.assert_status_is_ok();
    let body = response.0.into_body().into_string().await.expect("a body");
    for path in [
        "GET  /api/activities",
        "GET  /api/enhance",
        "POST /api/enhance",
        "POST /api/upload",
    ] {
        assert!(body.contains(path), "{path} is missing from {body}");
    }
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.activities-fn/test]
#[tokio::test]
async fn the_registry_answers_topics_and_modes() {
    let root = webapp_with(&["Substantive", "Adverbial"]);

    let response = served(root.path()).get("/api/activities").send().await;

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

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+2/test]
#[tokio::test]
async fn the_page_endpoint_needs_all_three() {
    let root = webapp_with(&["Substantive"]);
    let client = served(root.path());

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

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn+4/test]
#[tokio::test]
async fn the_span_endpoint_needs_one_source() {
    let root = webapp_with(&["Substantive"]);
    let client = served(root.path());

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
        client
            .post("/api/enhance")
            .body_json(&body)
            .send()
            .await
            .assert_status(StatusCode::BAD_REQUEST);
    }
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+2/test]
#[tokio::test]
async fn the_retired_paths_answer_nothing() {
    let root = webapp_with(&["Substantive"]);
    let client = served(root.path());

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

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn+1/test]
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

    let response = served(root.path())
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
