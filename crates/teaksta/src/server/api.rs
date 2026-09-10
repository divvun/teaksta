//! The HTTP surface: a topic registry, two enhancement endpoints and an
//! upload endpoint, over one shared analysis state, with the built web client
//! under them when the deployment carries one.
//!
//! Analysis is synchronous and blocking — the morpho seam owns a runtime of
//! its own — so every endpoint that analyses hands the work to a blocking
//! thread. The exercise travels with the request that asked for it, so two
//! requests wanting different exercises do not contend. Requests are answered
//! directly: analysing a page takes well under a second, so nothing is served
//! while the caller waits.

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context as _, Result};
use poem::endpoint::StaticFilesEndpoint;
use poem::http::StatusCode;
use poem::http::header::CONTENT_TYPE;
use poem::middleware::SizeLimit;
use poem::web::{Data, Json, Multipart, Query};
use poem::{EndpointExt, IntoResponse, Response, Route, get, handler, post};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{info, warn};

use crate::context::Config;
use crate::server::activities::Activities;
use crate::server::processors::Processors;
use crate::server::upload::{self, MAX_UPLOAD_BYTES, Rejection, Upload};
use crate::types::Document;
use crate::util::html_enhancer::{HtmlEnhancer, sami_label};
use crate::util::json_enhancer::JsonEnhancer;
use crate::util::page_handler::PageHandler;

/// The pipeline language the shipped activity descriptors declare. Every
/// topic registers its pre- and postprocessor under `en` because no analysis
/// engine was ever registered under `sme`; the pipelines behind that key are
/// the North Sámi ones.
const PIPELINE_LANGUAGE: &str = "en";

/// How long a page fetch may take.
const FETCH_TIMEOUT: Duration = Duration::from_secs(20);

/// What the upload body may weigh, counting the multipart framing around the
/// file the cap in [`MAX_UPLOAD_BYTES`] applies to.
const MAX_UPLOAD_BODY: usize = MAX_UPLOAD_BYTES + 64 * 1024;

/// The exercise a request asks for.
// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.enhancement-type+1]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.enhancement-type+1]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Colorize,
    Click,
    Mc,
    Cloze,
}

impl Mode {
    pub const ALL: [Mode; 4] = [Mode::Colorize, Mode::Click, Mode::Mc, Mode::Cloze];

    pub fn parse(value: &str) -> Option<Mode> {
        Mode::ALL.into_iter().find(|mode| mode.name() == value)
    }

    pub fn name(self) -> &'static str {
        match self {
            Mode::Colorize => "colorize",
            Mode::Click => "click",
            Mode::Mc => "mc",
            Mode::Cloze => "cloze",
        }
    }
}

/// One topic the registry offers, with its North Sámi name.
#[derive(Debug, Clone, Serialize)]
pub struct Topic {
    pub name: String,
    pub label: Option<String>,
    pub enabled: bool,
}

/// Everything a request is served from: the deployment configuration, the
/// per-topic pipelines, and the topic list they were built from.
// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet+2]
pub struct AppState {
    pub config: Config,
    pub processors: Processors,
    pub topics: Vec<Topic>,
}

impl AppState {
    /// Scans the activity tree and builds every topic's pipeline pair once,
    /// so no request pays for a model load.
    // [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.init-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.init-fn+2]
    pub fn new(config: Config) -> Result<Self> {
        let started = Instant::now();
        let mut activities = Activities::new(&config.activities_dir).with_context(|| {
            format!(
                "scanning activities under {}",
                config.activities_dir.display()
            )
        })?;

        let names: Vec<String> = activities.iterator().cloned().collect();
        let topics = names
            .into_iter()
            .map(|name| {
                let enabled = activities
                    .get_activity(&name)
                    .is_some_and(|activity| activity.is_enabled());
                Topic {
                    label: sami_label(&name).map(str::to_string),
                    name,
                    enabled,
                }
            })
            .collect();

        let processors = Processors::new(&mut activities)?;
        info!("Loaded every topic pipeline ({:?})", started.elapsed());

        Ok(AppState {
            config,
            processors,
            topics,
        })
    }

    pub fn knows_topic(&self, name: &str) -> bool {
        self.topics.iter().any(|topic| topic.name == name)
    }

    /// Runs one topic pipeline over a page for the requested exercise and
    /// hands back the annotated document.
    fn analyse(&self, activity: &str, mode: Mode, page: &str, key: &str) -> Result<Document> {
        let cache = self.config.analysis_dir.to_string_lossy().into_owned();
        let handler = PageHandler::new(
            &self.processors,
            activity,
            key,
            &cache,
            page,
            PIPELINE_LANGUAGE,
            mode,
        );
        handler
            .process()?
            .with_context(|| format!("no pipeline is registered for topic {activity:?}"))
    }
}

/// The URL map. Every path a client may reach is here; nothing else answers.
///
/// A deployment carrying a built web client serves it from the root, with any
/// path the bundle has no file for answered by `index.html` so the client's
/// own router owns it. The `/api` paths are static routes and the client's is
/// a catch-all, so the API answers first whatever the client routes.
pub fn routes(config: &Config) -> Route {
    let api = Route::new()
        .at("/api/activities", get(registry))
        .at("/api/enhance", get(enhance_page).post(enhance_spans))
        .at(
            "/api/upload",
            post(upload_text).with(SizeLimit::new(MAX_UPLOAD_BODY)),
        );

    match &config.webapp_dist {
        Some(dist) => api.nest(
            "/",
            StaticFilesEndpoint::new(dist)
                .index_file("index.html")
                .fallback_to_index(),
        ),
        None => api.at("/", get(index)),
    }
}

/// The root of a deployment with no web client: the endpoint listing, so an
/// API-only deployment can be probed without one.
// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.index-fn+1]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.index-fn+1]
#[handler]
async fn index() -> Response {
    let body = concat!(
        "teaksta\n",
        "\n",
        "GET  /api/activities\n",
        "GET  /api/enhance?url=&activity=&mode=\n",
        "POST /api/enhance\n",
        "POST /api/upload\n",
    );
    Response::builder()
        .header(CONTENT_TYPE, "text/plain;charset=UTF-8")
        .body(body)
}

// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.activities-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.activities-fn]
#[handler]
async fn registry(state: Data<&Arc<AppState>>) -> Json<serde_json::Value> {
    let modes: Vec<serde_json::Value> = Mode::ALL
        .into_iter()
        .map(|mode| json!({ "name": mode.name(), "label": sami_label(mode.name()) }))
        .collect();

    Json(json!({ "activities": &state.0.topics, "modes": modes }))
}

/// The query the whole-page endpoint takes. A missing member is a malformed
/// request, which the extractor answers before the handler runs.
#[derive(Debug, Deserialize)]
struct PageQuery {
    url: String,
    activity: String,
    mode: String,
}

// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+2]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+2]
#[handler]
async fn enhance_page(
    Query(query): Query<PageQuery>,
    state: Data<&Arc<AppState>>,
) -> poem::Result<Response> {
    let state = state.0.clone();
    let mode = parse_mode(&query.mode)?;
    let activity = known_topic(&state, query.activity)?;
    let url = page_url(&query.url)?;
    let requested = url.to_string();

    let started = Instant::now();
    let page = blocking(move || {
        let source = fetch_page(&url)?;
        let document = state.analyse(&activity, mode, &source, &cache_key(url.as_str()))?;
        HtmlEnhancer::new(&document).enhance(Some(mode), url.as_str())
    })
    .await?;

    info!(
        "Enhanced {} as {} in {:?}",
        requested,
        mode.name(),
        started.elapsed()
    );
    Ok(Response::builder()
        .header(CONTENT_TYPE, "text/html;charset=UTF-8")
        .body(page))
}

/// The body the span endpoint takes: the page itself, or where to fetch it.
#[derive(Debug, Deserialize)]
struct SpanRequest {
    #[serde(default)]
    html: Option<String>,
    #[serde(default)]
    url: Option<String>,
    activity: String,
    mode: String,
}

// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn+5]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn+5]
#[handler]
async fn enhance_spans(
    Json(request): Json<SpanRequest>,
    state: Data<&Arc<AppState>>,
) -> poem::Result<Response> {
    let state = state.0.clone();
    let mode = parse_mode(&request.mode)?;
    let activity = known_topic(&state, request.activity)?;
    let source = match (request.html, request.url) {
        (Some(html), None) => Source::Inline(html),
        (None, Some(url)) => Source::Fetched(page_url(&url)?),
        _ => return Err(bad_request("give exactly one of \"html\" and \"url\"")),
    };

    let started = Instant::now();
    let spans = blocking(move || {
        let (page, key) = source.read()?;
        let document = state.analyse(&activity, mode, &page, &key)?;
        JsonEnhancer::new(&document, mode).enhance()
    })
    .await?;

    info!(
        "Enhanced spans as {} in {:?}",
        mode.name(),
        started.elapsed()
    );
    Ok(Response::builder()
        .header(CONTENT_TYPE, "application/json")
        .body(spans))
}

/// Where the page under analysis comes from.
enum Source {
    Inline(String),
    Fetched(Url),
}

impl Source {
    /// The page, paired with the key its analysis is cached under.
    fn read(self) -> Result<(String, String)> {
        match self {
            Source::Inline(html) => {
                let key = cache_key(&html);
                Ok((html, key))
            }
            Source::Fetched(url) => Ok((fetch_page(&url)?, cache_key(url.as_str()))),
        }
    }
}

// [spec:teaksta:def:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn+1]
// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn+1]
#[handler]
async fn upload_text(
    mut multipart: Multipart,
    state: Data<&Arc<AppState>>,
) -> poem::Result<Response> {
    let state = state.0.clone();
    let mut collected = Upload::default();

    while let Some(field) = multipart.next_field().await? {
        let name = field.name().unwrap_or_default().to_string();
        let file_name = field.file_name().map(str::to_string);
        let bytes = field.bytes().await?;
        match file_name {
            Some(file_name) => {
                collected.file_name = Some(file_name);
                collected.content = bytes;
            }
            None if name == "keep" => {
                collected.keep = String::from_utf8_lossy(&bytes).trim() == "true";
            }
            None => {}
        }
    }

    let stored = tokio::task::spawn_blocking(move || {
        let directory = state.config.upload_dir(collected.keep).to_path_buf();
        let stored = upload::store(&collected, &directory)?;
        upload::file_url(&stored)
    })
    .await;

    match stored {
        Ok(Ok(url)) => {
            info!("Stored an upload at {url}");
            Ok(Json(json!({ "url": url })).into_response())
        }
        Ok(Err(error)) => rejected(error),
        Err(join) => {
            warn!("An upload terminated: {join}");
            Err(poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))
        }
    }
}

/// A gate that closed is the client's business and names itself in the body;
/// anything else the upload path reports is ours.
fn rejected(error: anyhow::Error) -> poem::Result<Response> {
    let Some(rejection) = error.downcast_ref::<Rejection>() else {
        warn!("{error:?}");
        return Err(poem::Error::from_string(
            format!("{error:#}"),
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    };

    let status = match rejection {
        Rejection::TooLarge => StatusCode::PAYLOAD_TOO_LARGE,
        _ => StatusCode::BAD_REQUEST,
    };
    info!("Upload refused: {}", rejection.code());
    Ok(Json(json!({ "error": rejection.code() }))
        .with_status(status)
        .into_response())
}

/// Runs blocking work on a thread of its own and turns whatever it reports
/// into the status the client should see.
async fn blocking<T, F>(work: F) -> poem::Result<T>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T> + Send + 'static,
{
    match tokio::task::spawn_blocking(work).await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(error)) => Err(failure(error)),
        Err(join) => {
            warn!("A request handler terminated: {join}");
            Err(poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))
        }
    }
}

/// A page that could not be fetched is the far end's failure; anything else
/// is ours.
fn failure(error: anyhow::Error) -> poem::Error {
    if error.downcast_ref::<Unreachable>().is_some() {
        info!("{error:#}");
        return poem::Error::from_string(format!("{error:#}"), StatusCode::BAD_GATEWAY);
    }
    warn!("{error:?}");
    poem::Error::from_string(format!("{error:#}"), StatusCode::INTERNAL_SERVER_ERROR)
}

/// The page could not be read from where the request pointed.
#[derive(Debug, thiserror::Error)]
#[error("the page at {0} could not be fetched")]
struct Unreachable(String);

fn bad_request(message: &str) -> poem::Error {
    poem::Error::from_string(message.to_string(), StatusCode::BAD_REQUEST)
}

fn parse_mode(value: &str) -> poem::Result<Mode> {
    Mode::parse(value).ok_or_else(|| bad_request("mode must be colorize, click, mc or cloze"))
}

fn known_topic(state: &AppState, activity: String) -> poem::Result<String> {
    if state.knows_topic(&activity) {
        Ok(activity)
    } else {
        Err(bad_request("activity is not a registered topic"))
    }
}

/// Reads the address a request points at. A bare host with no scheme is
/// taken as `http`, which is what a learner types into an address field.
pub fn page_url(raw: &str) -> poem::Result<Url> {
    let raw = raw.trim();
    let absolute = if raw.contains("://") || raw.starts_with("file:") {
        raw.to_string()
    } else {
        format!("http://{raw}")
    };
    Url::parse(&absolute).map_err(|_| bad_request("url is not a valid address"))
}

/// Which encoding the cached analysis is written in. A build that changes the
/// document model raises this, so every file an earlier encoding wrote is
/// keyed somewhere the new one never looks: the cache goes cold rather than
/// sour, and no deployment has to be told to empty a directory.
pub const CACHE_FORMAT_VERSION: u32 = 1;

/// How much of the digest the key carries. 128 bits is past the reach of a
/// search for two subjects sharing one.
const CACHE_KEY_BYTES: usize = 16;

/// The analysed document is cached under this key. A fetched page is keyed by
/// its address and an inline one by its own content, so neither is answered
/// from the other's analysis.
///
/// The digest is cryptographic, and the encoding version is hashed into it as
/// well as written in front of it. A key that could be collided is a key a
/// cache file could be planted under: whoever controls one page's address
/// would choose what another page's readers are served. Nothing about the key
/// varies with the build either, so a toolchain upgrade leaves a deployment's
/// cache addressable rather than silently orphaning it.
pub fn cache_key(subject: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    // The version and the subject are separated, so no subject can present
    // itself as one keyed under another version.
    hasher.update(format!("teaksta-analysis-v{CACHE_FORMAT_VERSION}\0").as_bytes());
    hasher.update(subject.as_bytes());
    let digest = hasher.finalize();

    format!(
        "v{CACHE_FORMAT_VERSION}-{}",
        &digest.to_hex()[..CACHE_KEY_BYTES * 2]
    )
}

/// The page as fetched. A `file:` address is read from disk, which is how an
/// accepted upload is reached.
fn fetch_page(url: &Url) -> Result<String> {
    let unreachable = || Unreachable(url.to_string());

    if url.scheme() == "file" {
        let path = url.to_file_path().map_err(|_| unreachable())?;
        return std::fs::read_to_string(&path)
            .map_err(|error| anyhow::Error::new(error).context(unreachable()));
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(FETCH_TIMEOUT)
        .build()?;
    let response = client
        .get(url.clone())
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|error| anyhow::Error::new(error).context(unreachable()))?;
    response
        .text()
        .map_err(|error| anyhow::Error::new(error).context(unreachable()))
}

#[cfg(test)]
#[path = "api_tests.rs"]
mod tests;
