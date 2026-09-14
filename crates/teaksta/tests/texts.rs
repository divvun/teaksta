//! The kept-text endpoint, over a whole deployment: what `/api/texts/<id>`
//! serves, what it refuses, and what the enhancement endpoints do with the
//! address it answers at.
//!
//! None of it needs the models. Serving a stored text analyses nothing, and
//! an address this deployment will not read is refused before any analysis
//! starts — so what is asserted here is the address and never the exercise.
//! The round trip through the real language gate is
//! `pipeline_models.rs`'s.

use std::path::Path;
use std::sync::Arc;

use poem::Endpoint;
use poem::EndpointExt;
use poem::http::StatusCode;
use poem::test::{TestClient, TestResponse};
use tempfile::TempDir;

use teaksta::context::Config;
use teaksta::server::api::{AppState, routes};
use teaksta::server::texts::TextStore;

const PAGE: &str = "<html><body><p>Mun oidnen viesu.</p></body></html>";

/// A page that announces itself as XHTML, so the type a stored text is served
/// as is shown to be read off the bytes rather than assumed.
const XHTML: &str = "<?xml version=\"1.0\"?>\
                     <html xmlns=\"http://www.w3.org/1999/xhtml\"><body><p>Viessu.</p></body></html>";

/// A deployment with the compiled-in registry and its directories under a
/// temporary root. No rate limit: the in-process transport carries no peer
/// address, so every request here would otherwise be one client.
fn config_under(root: &Path) -> Config {
    Config {
        listen: "127.0.0.1:0".to_string(),
        webapp_dist: None,
        topics: None,
        analysis_dir: root.join("analysed"),
        upload_keep_dir: Some(root.join("keep")),
        upload_temp_dir: root.join("temp"),
        trust_proxy: false,
        rate_limit: None,
        max_page_bytes: 5 * 1024 * 1024,
        // No Azure, so kept texts go to the local backing under the keep
        // directory — which is the backing every test and every laptop has.
        azure: None,
    }
}

/// A deployment holding the given pages as kept texts, with the address each
/// was stored at.
///
/// The store is put through directly rather than through the upload endpoint,
/// because that endpoint's language gate runs the analyser and what is under
/// test here is the address. The gate reaching the store is
/// `pipeline_models.rs`'s to show.
async fn deployment(pages: &[&str]) -> (TempDir, TestClient<impl Endpoint>, Vec<String>) {
    let root = TempDir::new().expect("a deployment root");
    let config = config_under(root.path());
    for directory in [&config.analysis_dir, &config.upload_temp_dir]
        .into_iter()
        .chain(&config.upload_keep_dir)
    {
        std::fs::create_dir_all(directory).expect("a deployment directory");
    }

    let store = TextStore::from_config(&config)
        .expect("the text store opens")
        .expect("a deployment naming a keep directory keeps texts");
    let mut addresses = Vec::new();
    for page in pages {
        let id = store
            .put(page.as_bytes().to_vec())
            .await
            .expect("the text is stored");
        addresses.push(id.reference());
    }

    let state = Arc::new(AppState::new(config).expect("the deployment boots"));
    let client = TestClient::new(routes(&state.config).data(state));
    (root, client, addresses)
}

/// One whole-page request, with the topic and the exercise held fixed so the
/// address is the only thing under test.
async fn enhanced<E: Endpoint>(client: &TestClient<E>, address: &str) -> TestResponse {
    client
        .get("/api/enhance")
        .query("url", &address)
        .query("activity", &"Substantive")
        .query("mode", &"colorize")
        .send()
        .await
}

/// A kept text is reachable at the address the upload endpoint hands back,
/// served as the type the gate accepted and under the headers that keep a
/// page somebody uploaded from running as this deployment.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.texts-fn+1/test]
#[tokio::test]
async fn a_kept_text_is_served_at_its_address() {
    let (_root, client, addresses) = deployment(&[PAGE, XHTML]).await;

    let response = client.get(&addresses[0]).send().await;

    response.assert_status_is_ok();
    response.assert_header("content-type", "text/html; charset=UTF-8");
    // The page is served from this deployment's own origin, so it is served
    // as nobody: the sandbox puts it in an origin of its own and the sniffing
    // is off.
    response.assert_header("x-content-type-options", "nosniff");
    response.assert_header("content-security-policy", "sandbox");
    response.assert_text(PAGE).await;

    // XHTML announces itself in its bytes and is served as itself.
    let xhtml = client.get(&addresses[1]).send().await;
    xhtml.assert_header("content-type", "application/xhtml+xml");
    xhtml.assert_text(XHTML).await;
}

/// A name that holds nothing, a name that is not a name, and a name asked for
/// through the enhancement endpoints are one 404: nothing a caller writes
/// reaches an object key, and probing tells them nothing.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.texts-fn+1/test]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+8/test]
#[tokio::test]
async fn an_unheld_text_is_not_found() {
    let (_root, client, _) = deployment(&[]).await;

    for name in [
        // Well formed, holding nothing.
        "0123456789abcdef0123456789abcdef",
        // Not a name at all.
        "nonsense",
        "0123456789ABCDEF0123456789abcdef",
        "..",
        "%2e%2e%2f%2e%2e%2fetc%2fpasswd",
    ] {
        client
            .get(format!("/api/texts/{name}"))
            .send()
            .await
            .assert_status(StatusCode::NOT_FOUND);
    }

    // And through the enhancement endpoints, which is the one place a stored
    // text does not answer as the `file:` era's swept upload did: a name is
    // either held or it is not, so it is a 404 rather than a far end that
    // failed.
    enhanced(&client, "/api/texts/0123456789abcdef0123456789abcdef")
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

/// A kept text named as a page to enhance is read from the store rather than
/// fetched, so the address reaches the analyser instead of being refused by
/// the policy that would — quite correctly — refuse this deployment a
/// connection to itself.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+8/test]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.blocks-fn+3/test]
#[tokio::test]
async fn a_kept_text_is_enhanced_without_being_fetched() {
    let (_root, client, addresses) = deployment(&[PAGE]).await;
    let address = &addresses[0];

    assert_ne!(
        enhanced(&client, address).await.0.status(),
        StatusCode::BAD_REQUEST,
        "a kept text is an address this deployment reads"
    );

    // The block endpoint takes the same address the same way, and it is the
    // one the web client actually asks with.
    let answered = client
        .post("/api/enhance/blocks")
        .body_json(&serde_json::json!({
            "url": address,
            "activity": "Substantive",
            "mode": "colorize",
        }))
        .send()
        .await;
    assert_ne!(answered.0.status(), StatusCode::BAD_REQUEST);
}

/// The recognition is of a reference, not of a path. An address that merely
/// looks like one and carries a host is an ordinary address of that host, and
/// is judged exactly as any other address of it would be — so nothing about
/// the stored-text shape lets a caller past the private-address policy.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+8/test]
#[tokio::test]
async fn a_text_path_on_a_host_is_refused() {
    // A name that really is held here, so what the table below shows is the
    // host and never the name.
    let (_root, client, addresses) = deployment(&[PAGE]).await;
    let held = &addresses[0];
    let name = held
        .strip_prefix("/api/texts/")
        .expect("a stored-text reference");

    for address in [
        format!("http://127.0.0.1/api/texts/{name}"),
        format!("http://localhost/api/texts/{name}"),
        format!("http://169.254.169.254/api/texts/{name}"),
        format!("http://[::1]/api/texts/{name}"),
        format!("http://10.0.0.7/api/texts/{name}"),
        // The protocol-relative form names a host by leaving out only the
        // scheme, so it is not a reference to anything here.
        format!("//169.254.169.254/api/texts/{name}"),
        // Neither is a path of the right shape under another prefix.
        format!("/api/enhance/../texts/{name}"),
        format!("/texts/{name}"),
    ] {
        enhanced(&client, &address)
            .await
            .assert_status(StatusCode::BAD_REQUEST);
    }

    // The reference itself is read, so the table above is refused for the
    // host each entry names and not because the name is unknown.
    assert_ne!(
        enhanced(&client, held).await.0.status(),
        StatusCode::BAD_REQUEST
    );
}
