//! The HTTP surface: a topic registry, three enhancement endpoints, an
//! upload endpoint and the two the cluster's probes read, over one shared
//! analysis state, with the built web client under them when the deployment
//! carries one.
//!
//! The three enhancement endpoints answer the same analysis three ways: a
//! whole page for a caller that wants the document back, the per-token span
//! map the browser add-on spliced into a page it already held, and the
//! analysed text block by block for a client that renders the exercise
//! itself, which is what the web client asks for.
//!
//! Analysis is synchronous and blocking — the morpho seam owns a runtime of
//! its own — so every endpoint that analyses hands the work to a blocking
//! thread. The exercise travels with the request that asked for it, so two
//! requests wanting different exercises do not contend. Requests are answered
//! directly: analysing a page takes well under a second, so nothing is served
//! while the caller waits.
//!
//! An address a request carries is vetted by [`crate::server::fetch`] before
//! anything is opened, so what an endpoint here holds is already a target this
//! deployment is willing to read.

use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use anyhow::{Context as _, Result, bail};
use poem::endpoint::StaticFilesEndpoint;
use poem::error::ParseJsonError;
use poem::http::StatusCode;
use poem::http::header::CONTENT_TYPE;
use poem::middleware::{CatchPanic, SizeLimit};
use poem::web::{Data, Json, Multipart, Query, RequestBody};
use poem::{
    Endpoint, EndpointExt, FromRequest, IntoResponse, Request, Response, Route, get, handler, post,
};
use reqwest::Url;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{info, warn};

use crate::context::Config;
use crate::morpho::{BUNDLE_ENV, GENERATOR_ENV, MorphoPipeline};
use crate::server::access;
use crate::server::fetch::{self, Overloaded, Oversized, Refusal, Unreachable};
use crate::server::registry::Registry;
use crate::server::upload::{self, MAX_UPLOAD_BYTES, Rejection, Upload};
use crate::types::Document;
use crate::util::html_blocks;
use crate::util::html_enhancer::{HtmlEnhancer, mode_label};
use crate::util::json_enhancer::JsonEnhancer;
use crate::util::page_handler::PageHandler;

/// The exercise a request asks for. It is defined beside the document it is
/// analysed for, because the pipeline stages and every topic enhancer read
/// one and none of them knows this module; it is re-exported here because
/// the endpoints below are where a request's exercise is parsed.
pub use crate::types::Mode;

/// What the upload body may weigh, counting the multipart framing around the
/// file the cap in [`MAX_UPLOAD_BYTES`] applies to.
const MAX_UPLOAD_BODY: usize = MAX_UPLOAD_BYTES + 64 * 1024;

/// What a POST enhancement body may weigh, span map and blocks alike. A page
/// carried inline is the largest thing either holds, so it weighs what an
/// upload may, with the same room for the framing around it.
const MAX_ENHANCE_BODY: usize = MAX_UPLOAD_BYTES + 64 * 1024;

/// Everything a request is served from: the deployment configuration and the
/// topic registry, with the pipeline pair each topic runs.
// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet+4]
pub struct AppState {
    pub config: Config,
    pub registry: Registry,
    /// Whether the deep check has already analysed a sentence in this
    /// process. It is the one thing here that is written after startup, and
    /// it is written once and never back: see
    /// [`health_deep`] for why only a success is remembered.
    ///
    /// Relaxed is the whole ordering this needs. Nothing is published through
    /// the flag — the models it stands for are behind the morpho seam's own
    /// synchronisation — so it says only that some earlier request got an
    /// answer out of them, and a reader that briefly misses a store runs the
    /// check again and reaches the same verdict.
    models_proven: AtomicBool,
}

impl AppState {
    /// Reads the topic registry and builds every topic's pipeline pair once,
    /// so no request pays for a model load.
    // [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.init-fn+3]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.init-fn+3]
    pub fn new(config: Config) -> Result<Self> {
        let started = Instant::now();
        let loaded = Registry::from_config(config.topics.as_deref())?;
        info!(
            "Loaded {} topic pipelines ({:?})",
            loaded.topics().len(),
            started.elapsed()
        );

        Ok(AppState {
            config,
            registry: loaded,
            models_proven: AtomicBool::new(false),
        })
    }

    pub fn knows_topic(&self, name: &str) -> bool {
        self.registry.knows(name)
    }

    /// Runs one topic pipeline over a page for the requested exercise and
    /// hands back the annotated document.
    fn analyse(&self, activity: &str, mode: Mode, page: &str, key: &str) -> Result<Document> {
        let cache = self.config.analysis_dir.to_string_lossy().into_owned();
        let handler = PageHandler::new(&self.registry, activity, key, &cache, page, mode);
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
///
/// Three layers stand over it. The panic guard, so a handler that panics is
/// answered rather than dropping the connection under the caller. The access
/// log, outside the guard so that the 500 the guard writes is logged like any
/// other answer, and over the whole map so the static client is logged too.
/// And the per-client rate limit, over the four routes that analyse and not
/// over the registry, which is a read of state built at startup and costs
/// nothing worth counting.
///
/// The two health routes are registered outside the limiter for a different
/// reason: they are read by the cluster, not by a client, and a probe that
/// is answered 429 is a probe that failed. The kubelet asks from the pod
/// network, so every probe of every pod on a node presents as one address —
/// exactly the shape a per-address allowance is built to bound — and a busy
/// node would spend a pod's own allowance on the requests that decide whether
/// that pod lives.
///
/// The limiter is built here, so one map is one limiter and the four routes
/// it covers share it: a client's allowance is spent across the endpoints
/// that analyse together rather than four times over.
pub fn routes(config: &Config) -> impl Endpoint + use<> {
    let analysis = access::Limit::new(config);

    let api = Route::new()
        .at("/api/health", get(health))
        .at("/api/health/deep", get(health_deep))
        .at("/api/activities", get(registry))
        .at(
            "/api/enhance",
            get(enhance_page).post(enhance_spans).with(analysis.clone()),
        )
        .at(
            "/api/enhance/blocks",
            post(enhance_blocks).with(analysis.clone()),
        )
        .at(
            "/api/upload",
            post(upload_text)
                .with(SizeLimit::new(MAX_UPLOAD_BODY))
                .with(analysis.clone()),
        );

    #[cfg(test)]
    let api = api.at("/api/panic", get(panics));

    let map = match &config.webapp_dist {
        Some(dist) => api.nest(
            "/",
            StaticFilesEndpoint::new(dist)
                .index_file("index.html")
                .fallback_to_index(),
        ),
        None => api.at("/", get(index)),
    };

    map.with(CatchPanic::new().with_handler(panicked))
        .with(access::AccessLog::new(config.trust_proxy))
}

/// What a panicking handler is answered with. The guard unwinds the panic and
/// hands its payload here, so the failure is recorded and the caller is told
/// something rather than seeing the connection reset with nothing on it.
fn panicked(payload: Box<dyn Any + Send + 'static>) -> (StatusCode, &'static str) {
    let raised = payload
        .downcast_ref::<&str>()
        .map(|message| (*message).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "a payload of no known type".to_string());
    warn!("A request handler panicked: {raised}");
    (StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
}

/// A route that panics, so the guard the map is served behind can be shown to
/// answer. It is compiled only under test and reaches no deployment.
#[cfg(test)]
#[handler]
async fn panics() -> &'static str {
    panic!("the panic this route exists to raise")
}

/// The root of a deployment with no web client: the endpoint listing, so an
/// API-only deployment can be probed without one.
// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.index-fn+4]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.index-fn+4]
#[handler]
async fn index() -> Response {
    let body = concat!(
        "teaksta\n",
        "\n",
        "GET  /api/activities\n",
        "GET  /api/enhance?url=&activity=&mode=\n",
        "POST /api/enhance\n",
        "POST /api/enhance/blocks\n",
        "POST /api/upload\n",
        "GET  /api/health\n",
        "GET  /api/health/deep\n",
    );
    Response::builder()
        .header(CONTENT_TYPE, "text/plain;charset=UTF-8")
        .body(body)
}

/// The sentence the deep check analyses: four words of North Sámi, one of
/// them a noun the analyser has a reading for, which is the smallest input
/// that makes the tokeniser and the analyser do their real work.
const HEALTH_SENTENCE: &str = "Mun oidnen viesu ikte.";

/// The lookup the deep check asks the generator for. A form the generator
/// knows, so a transducer that loaded answers with one rather than with the
/// `+?` echo — though the echo would pass too: what is being proved is that
/// the transducer is there and can be queried, not what it knows.
const HEALTH_LOOKUP: &str = "viessu+N+Sg+Nom";

/// The liveness and readiness probe: is this process still answering?
// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.health-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.health-fn]
#[handler]
async fn health(state: Data<&Arc<AppState>>) -> Json<serde_json::Value> {
    // A slice read off state built at startup. Nothing here fetches, analyses,
    // opens a file or takes a lock, so a process spending every core on an
    // analysis answers this as fast as an idle one does — which is the whole
    // requirement, because a liveness probe that times out under load is a
    // liveness probe that kills the pods doing the most work.
    Json(json!({ "status": "ok", "topics": state.0.registry.topics().len() }))
}

/// The startup probe: do the models this deployment was given actually load
/// and answer?
///
/// The expensive path runs at most once per process. A startup probe asks
/// until it is answered and then stops asking, so a verdict that was reached
/// is the verdict for the life of the process, and remembering it is what
/// keeps a stranger who found the address from being able to ask for an
/// analysis, unmetered, as often as they like. A failure is not remembered:
/// the probe that asked is going to ask again, and the models may be a moment
/// from ready.
// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.health-deep-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.health-deep-fn]
#[handler]
async fn health_deep(state: Data<&Arc<AppState>>) -> Response {
    let state = state.0.clone();
    if state.models_proven.load(Ordering::Relaxed) {
        return loaded();
    }

    // Named before anything is spawned, so the deployment that has no models
    // at all — which is every deployment built without them — is told what is
    // missing without a thread being taken for it.
    if let Some(missing) = unset_model_variables() {
        return degraded(&missing);
    }

    // The morpho seam owns a runtime of its own and blocks on it, so the
    // analysis goes to a blocking thread exactly as every analysing endpoint's
    // does.
    let started = Instant::now();
    match tokio::task::spawn_blocking(analyse_one_sentence).await {
        Ok(Ok(())) => {
            state.models_proven.store(true, Ordering::Relaxed);
            info!("The models answered a sentence in {:?}", started.elapsed());
            loaded()
        }
        Ok(Err(error)) => {
            warn!("The models did not answer: {error:?}");
            degraded(&format!("{error:#}"))
        }
        Err(join) => {
            warn!("The deep health check terminated: {join}");
            degraded(&format!("the check terminated: {join}"))
        }
    }
}

/// The two variables naming this deployment's models, when either is unset.
/// A deployment without them serves the registry and refuses every analysis,
/// which is a state worth being told about by name rather than being left to
/// show up as a failed request.
fn unset_model_variables() -> Option<String> {
    let missing: Vec<&str> = [BUNDLE_ENV, GENERATOR_ENV]
        .into_iter()
        .filter(|name| std::env::var_os(name).is_none())
        .collect();
    if missing.is_empty() {
        return None;
    }
    Some(format!(
        "{} is not set; this deployment analyses nothing",
        missing.join(" and ")
    ))
}

/// One sentence through both models. The bundle is asked to tokenise and then
/// to analyse, which is the path every exercise is built on; the generator is
/// asked for one form, because a deployment whose generator will not load
/// answers the multiple-choice and cloze exercises with nothing and the probe
/// that exists to catch a model that is not there should catch that one too.
///
/// Both answers are weighed rather than merely awaited: a pipeline that
/// returned an empty stream has failed in the way that matters here, and a
/// check that only asked whether a call returned would pass on it.
fn analyse_one_sentence() -> Result<()> {
    let pipeline = MorphoPipeline::shared();

    let tokens = pipeline
        .tokenize(HEALTH_SENTENCE)
        .context("the tokenise pipeline")?;
    if tokens.is_empty() {
        bail!("the tokenise pipeline answered {HEALTH_SENTENCE:?} with no tokens");
    }

    let stream = pipeline
        .analyze_disambiguate(&tokens)
        .context("the analyse pipeline")?;
    if stream.trim().is_empty() {
        bail!("the analyse pipeline answered {HEALTH_SENTENCE:?} with an empty stream");
    }

    pipeline
        .generate(HEALTH_LOOKUP)
        .context("the normative generator")?;
    Ok(())
}

/// What a proved deployment answers, whether it was proved a moment ago or at
/// boot. The two are the same answer deliberately: which of them a caller got
/// is this process's business, and a body that told them apart would be a
/// thing to hold stable for a reader who started reading it.
fn loaded() -> Response {
    Json(json!({ "status": "ok", "models": "loaded" })).into_response()
}

/// What a deployment that cannot analyse answers: 503, with what went wrong
/// in the body, so an operator reading the probe's own reply is told rather
/// than being sent to the logs.
fn degraded(reason: &str) -> Response {
    Json(json!({ "status": "failed", "error": reason }))
        .with_status(StatusCode::SERVICE_UNAVAILABLE)
        .into_response()
}

// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.activities-fn+1]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.activities-fn+1]
#[handler]
async fn registry(state: Data<&Arc<AppState>>) -> Json<serde_json::Value> {
    let modes: Vec<serde_json::Value> = Mode::ALL
        .into_iter()
        .map(|mode| json!({ "name": mode.name(), "label": mode_label(mode) }))
        .collect();

    Json(json!({ "activities": state.0.registry.topics(), "modes": modes }))
}

/// The query the whole-page endpoint takes. A missing member is a malformed
/// request, which the extractor answers before the handler runs.
#[derive(Debug, Deserialize)]
struct PageQuery {
    url: String,
    activity: String,
    mode: String,
}

// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+6]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+6]
#[handler]
async fn enhance_page(
    Query(query): Query<PageQuery>,
    state: Data<&Arc<AppState>>,
) -> poem::Result<Response> {
    let state = state.0.clone();
    let mode = parse_mode(&query.mode)?;
    let activity = known_topic(&state, query.activity)?;
    let url = page_url(&query.url)?;
    let target = fetch::target(&url, &state.config).map_err(|refusal| failure(refusal.into()))?;
    let requested = target.address().to_string();
    let key = cache_key(url.as_str());

    let started = Instant::now();
    let source = fetch::fetch(target).await.map_err(failure)?;
    let page = blocking(move || {
        let document = state.analyse(&activity, mode, &source, &key)?;
        Ok(HtmlEnhancer::new(&document).enhance(Some(mode), url.as_str()))
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

/// A JSON body read under a cap, which is what the span endpoint takes
/// instead of poem's own `Json`.
///
/// The cap has to bound the read itself rather than a declared length.
/// poem's `SizeLimit` middleware weighs only the `Content-Length` header —
/// and refuses a request carrying none outright — while `Json` then buffers
/// however many bytes actually arrive. A chunked request declares no length,
/// so a body streamed forever would be read forever; here it is read up to
/// the cap and refused with 413 at it, declared or not.
struct CappedJson<T>(T);

impl<'a, T: DeserializeOwned> FromRequest<'a> for CappedJson<T> {
    async fn from_request(request: &'a Request, body: &mut RequestBody) -> poem::Result<Self> {
        let content_type = request
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .ok_or(ParseJsonError::ContentTypeRequired)?;
        if !is_json(content_type) {
            return Err(ParseJsonError::InvalidContentType(content_type.to_string()).into());
        }

        let bytes = body.take()?.into_bytes_limit(MAX_ENHANCE_BODY).await?;
        serde_json::from_slice(&bytes)
            .map(CappedJson)
            .map_err(|error| ParseJsonError::Parse(error).into())
    }
}

/// Whether a body announces itself as JSON. Requiring it is what keeps a
/// plain HTML form, which can announce nothing of the sort, from reaching an
/// endpoint that analyses whatever it is given.
fn is_json(content_type: &str) -> bool {
    let media = content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    media == "application/json" || (media.starts_with("application/") && media.ends_with("+json"))
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

// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn+7]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn+7]
#[handler]
async fn enhance_spans(
    CappedJson(request): CappedJson<SpanRequest>,
    state: Data<&Arc<AppState>>,
) -> poem::Result<Response> {
    let state = state.0.clone();
    let mode = parse_mode(&request.mode)?;
    let activity = known_topic(&state, request.activity)?;

    let started = Instant::now();
    let (page, key) = page_source(&state, request.html, request.url).await?;

    let spans = blocking(move || {
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

/// The page a request names, with the key its analysis is cached under.
///
/// The page is keyed by its address when it is fetched and by its own content
/// when it arrives inline, so neither is answered from the other's analysis.
async fn page_source(
    state: &AppState,
    html: Option<String>,
    url: Option<String>,
) -> poem::Result<(String, String)> {
    match (html, url) {
        (Some(html), None) => {
            let key = cache_key(&html);
            Ok((html, key))
        }
        (None, Some(raw)) => {
            let url = page_url(&raw)?;
            let target =
                fetch::target(&url, &state.config).map_err(|refusal| failure(refusal.into()))?;
            let key = cache_key(url.as_str());
            Ok((fetch::fetch(target).await.map_err(failure)?, key))
        }
        _ => Err(bad_request("give exactly one of \"html\" and \"url\"")),
    }
}

/// The body the block endpoint takes: the page itself, or where to fetch it,
/// with the topic and the exercise to analyse it for. It asks what the span
/// endpoint's body asks and is held apart from it all the same, because one
/// of the two mirrors a protocol that is finished and the other is ours.
#[derive(Debug, Deserialize)]
struct BlockRequest {
    #[serde(default)]
    html: Option<String>,
    #[serde(default)]
    url: Option<String>,
    activity: String,
    mode: String,
}

/// One block of the analysed text, as the client reads it.
#[derive(Debug, Serialize)]
struct TextBlock {
    html: String,
}

// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.blocks-fn+2]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.blocks-fn+2]
#[handler]
async fn enhance_blocks(
    CappedJson(request): CappedJson<BlockRequest>,
    state: Data<&Arc<AppState>>,
) -> poem::Result<Response> {
    let state = state.0.clone();
    let mode = parse_mode(&request.mode)?;
    let activity = known_topic(&state, request.activity)?;

    let started = Instant::now();
    let (page, key) = page_source(&state, request.html, request.url).await?;

    let blocks = blocking(move || {
        let document = state.analyse(&activity, mode, &page, &key)?;
        let blocks: Vec<TextBlock> =
            html_blocks::render_blocks(&document.page, &document, Some(mode))
                .into_iter()
                .map(|html| TextBlock { html })
                .collect();

        Ok(serde_json::to_string(&blocks)?)
    })
    .await?;

    info!(
        "Enhanced blocks as {} in {:?}",
        mode.name(),
        started.elapsed()
    );
    Ok(Response::builder()
        .header(CONTENT_TYPE, "application/json")
        .body(blocks))
}

// [spec:teaksta:def:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn+4]
// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn+4]
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

/// An address this deployment will not fetch is the caller's mistake, and is
/// named as such; a page that could not be fetched, or that weighs more than
/// this deployment reads, is the far end's failure; a fetch that found no slot
/// is a load the deployment is asked to shed; anything else is ours.
fn failure(error: anyhow::Error) -> poem::Error {
    if let Some(refusal) = error.downcast_ref::<Refusal>() {
        info!("Refused an address: {refusal}");
        return bad_request(&refusal.to_string());
    }
    if error.downcast_ref::<Overloaded>().is_some() {
        warn!("{error:#}");
        return poem::Error::from_string(format!("{error:#}"), StatusCode::SERVICE_UNAVAILABLE);
    }
    if error.downcast_ref::<Oversized>().is_some() {
        info!("{error:#}");
        return poem::Error::from_string(format!("{error:#}"), StatusCode::BAD_GATEWAY);
    }
    if error.downcast_ref::<Unreachable>().is_some() {
        info!("{error:#}");
        return poem::Error::from_string(format!("{error:#}"), StatusCode::BAD_GATEWAY);
    }
    warn!("{error:?}");
    poem::Error::from_string(format!("{error:#}"), StatusCode::INTERNAL_SERVER_ERROR)
}

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
///
/// This only parses. Whether the address is one this deployment will fetch is
/// [`fetch::target`]'s decision, and every caller makes it before reading
/// anything.
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
///
/// A build that changes what is analysed raises it for the same reason. It
/// went to 2 when fetched pages began to be reduced to their main content:
/// the key names an address, the page behind that address is now cut down
/// before it is analysed, and every analysis written before the cut is of a
/// document this build would never produce.
///
/// It went to 3 when the document dropped the language it carried. Every
/// file written before that holds a field this build's model has no home
/// for, and the cache files themselves are named differently now — so the
/// old files are neither read nor looked for.
pub const CACHE_FORMAT_VERSION: u32 = 3;

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

#[cfg(test)]
#[path = "api_tests.rs"]
mod tests;
