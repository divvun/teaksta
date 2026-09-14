//! What a deployment with nowhere to keep a text answers, over the whole
//! HTTP surface.
//!
//! Taking a teacher's text is a capability: an Azure container or a named
//! keep directory, or neither. This is the neither — which is what the
//! cluster is until an operator mints the storage secret — read back through
//! the three places a client can see it: the two paths that are not there,
//! the plain-text listing that does not name them, and the registry field a
//! client reads before it offers anybody a file field.
//!
//! None of it needs the models. Not registering a route analyses nothing, and
//! the registry is read off state built at startup.

use std::path::Path;
use std::sync::Arc;

use poem::Endpoint;
use poem::EndpointExt;
use poem::http::StatusCode;
use poem::test::TestClient;
use tempfile::TempDir;

use teaksta::context::Config;
use teaksta::server::api::{AppState, routes};

/// A deployment with the compiled-in registry, its directories under a
/// temporary root, and whichever answer to the capability the caller wants.
/// No rate limit: the in-process transport carries no peer address, so every
/// request here would otherwise be one client.
fn deployment(root: &Path, uploads: bool) -> Config {
    Config {
        listen: "127.0.0.1:0".to_string(),
        webapp_dist: None,
        topics: None,
        analysis_dir: root.join("analysed"),
        // The whole difference between the two deployments here. Naming no
        // Azure container and no keep directory is a deployment that keeps no
        // texts; naming the directory is how a laptop turns the flow on.
        upload_keep_dir: uploads.then(|| root.join("keep")),
        upload_temp_dir: root.join("temp"),
        trust_proxy: false,
        rate_limit: None,
        max_page_bytes: 5 * 1024 * 1024,
        azure: None,
    }
}

fn served(root: &Path, uploads: bool) -> TestClient<impl Endpoint> {
    let state = Arc::new(AppState::new(deployment(root, uploads)).expect("the deployment boots"));
    TestClient::new(routes(&state.config).data(state))
}

/// The listing is of the map this deployment built, so a deployment that
/// keeps no texts names neither upload path in it. Everything else is still
/// there: what is gone is the flow with nowhere to put a text, not the
/// service.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.index-fn+6/test]
#[tokio::test]
async fn the_index_names_only_what_answers() {
    let root = TempDir::new().expect("a deployment root");

    let response = served(root.path(), false).get("/").send().await;

    response.assert_status_is_ok();
    let body = response.0.into_body().into_string().await.expect("a body");
    for path in ["POST /api/upload", "GET  /api/texts/<id>"] {
        assert!(!body.contains(path), "{path} is listed but answers nothing");
    }
    for path in [
        "GET  /api/activities",
        "GET  /api/enhance",
        "POST /api/enhance/blocks",
        "GET  /api/health",
        "GET  /api/health/deep",
    ] {
        assert!(body.contains(path), "{path} is missing from {body}");
    }

    // The same deployment with somewhere to keep a text lists both.
    let offered = served(root.path(), true).get("/").send().await;
    offered.assert_status_is_ok();
    let body = offered.0.into_body().into_string().await.expect("a body");
    for path in ["POST /api/upload", "GET  /api/texts/<id>"] {
        assert!(body.contains(path), "{path} answers but is not listed");
    }
}

/// Neither of the two paths a stored text lives behind answers on a
/// deployment that keeps none. Both are 404 — the answer any path this map
/// does not hold gets — because neither route is registered at all.
// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn+6/test]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.texts-fn+1/test]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+8/test]
#[tokio::test]
async fn no_store_answers_neither_upload_path() {
    let root = TempDir::new().expect("a deployment root");
    let client = served(root.path(), false);
    let boundary = "teaksta-uploads-boundary";
    let body = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"sami.html\"\r\n\
         Content-Type: text/html\r\n\r\n\
         <p>Mun oidnen viesu.</p>\r\n\
         --{boundary}--\r\n"
    );

    client
        .post("/api/upload")
        .content_type(format!("multipart/form-data; boundary={boundary}"))
        .header("content-length", body.len())
        .body(body)
        .send()
        .await
        .assert_status(StatusCode::NOT_FOUND);

    // A well-formed name and a name that is not one alike: this deployment
    // holds no text under either, and probing tells nobody anything.
    for name in ["0123456789abcdef0123456789abcdef", "nonsense"] {
        client
            .get(format!("/api/texts/{name}"))
            .send()
            .await
            .assert_status(StatusCode::NOT_FOUND);
    }

    // And a reference named as a page to enhance is a text this deployment
    // does not have, which is the same 404 a name it never stored gets.
    client
        .get("/api/enhance")
        .query("url", &"/api/texts/0123456789abcdef0123456789abcdef")
        .query("activity", &"Substantive")
        .query("mode", &"colorize")
        .send()
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

/// The capability a client cannot work out for itself. A deployment with a
/// store says so and one without says so, and the answer is the same thing
/// the two routes are registered on — so a client told yes can ask, and one
/// told no shows a teacher nothing that would 404.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.activities-fn+2/test]
#[tokio::test]
async fn the_registry_says_whether_uploads_are_taken() {
    let root = TempDir::new().expect("a deployment root");

    for uploads in [true, false] {
        let response = served(root.path(), uploads)
            .get("/api/activities")
            .send()
            .await;

        response.assert_status_is_ok();
        let body = response.0.into_body().into_string().await.expect("a body");
        let offered: serde_json::Value = serde_json::from_str(&body).expect("a JSON object");
        assert_eq!(offered["uploads"], serde_json::json!(uploads), "{body}");
        // The rest of the reply is the same registry either way.
        assert!(
            offered["activities"]
                .as_array()
                .is_some_and(|a| !a.is_empty())
        );
    }
}
