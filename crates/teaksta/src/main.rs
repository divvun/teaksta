//! Server binary for the ported web application.
//!
//! The servlet container is replaced by a poem router carrying the same URL
//! mapping `web.xml` declared. Each endpoint collects the request into the
//! model structs the translated handlers already read, runs the handler on a
//! blocking thread — the morpho seam and the page fetch both block — and
//! converts the response model back.
//!
//! The deployment paths `web.xml` hard-coded under `/home/teaksta` are read
//! from the environment instead, defaulting under `./data/`.
//!
//! `TEAKSTA_WEBAPP_ROOT` names the expanded web application: the `/activities`
//! lookup, the `WERTi.properties` load and the descriptor classpath are all
//! resolved from it, and it defaults to the working directory for a server
//! started from the webapp root. `TEAKSTA_PROPERTIES` overrides the properties
//! file by path and `TEAKSTA_CLASSPATH` the descriptor root.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{Result, anyhow};
use poem::http::header::{CONTENT_TYPE, COOKIE, HOST, SET_COOKIE};
use poem::http::uri::Scheme;
use poem::listener::TcpListener;
use poem::web::{Data, Multipart};
use poem::{
    Body, EndpointExt, FromRequest, IntoResponse, Request, RequestBody, Response, Route, Server,
    get, handler, post,
};
use rand::Rng;
use rand::distr::Alphanumeric;
use tracing::{error, info, warn};

use teaksta::morpho::{BUNDLE_ENV, GENERATOR_ENV};
use teaksta::server::activities::{
    HttpServletRequest as SessionRequest, HttpSession, ServletContext as SessionContext,
};
use teaksta::server::activity_configuration::classpath_root;
use teaksta::server::servlet::{
    HttpServletRequest as WertiRequest, HttpServletResponse, ServletConfig,
    ServletContext as WertiContext, WertiServlet,
};
use teaksta::server::upload::{
    FileLocationContextListener, HttpServletRequest as UploadRequest,
    ServletContext as UploadContext, ServletContextEvent, UploadDownloadFileServlet,
    collect_multipart,
};

/// The name a servlet container hands its session cookie.
const SESSION_COOKIE: &str = "JSESSIONID";

/// `<display-name>` of the deployment, which the HTML enhancer prints.
const DISPLAY_NAME: &str = "WERTisme";

/// The `context-param` block of `web.xml` plus the listen address, each entry
/// overridable from the environment.
struct Config {
    listen: String,
    files_dir: String,
    files_prm_dir: String,
    files_tmp_dir: String,
    files_anl_dir: String,
    secret_key: String,
    webapp_root: PathBuf,
}

impl Config {
    /// The context parameters are declared once for the deployment, so both
    /// servlets read the same table.
    fn init_parameters(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("files_dir".to_string(), self.files_dir.clone()),
            ("files_prm_dir".to_string(), self.files_prm_dir.clone()),
            ("files_tmp_dir".to_string(), self.files_tmp_dir.clone()),
            ("files_anl_dir".to_string(), self.files_anl_dir.clone()),
            ("secret_key".to_string(), self.secret_key.clone()),
        ])
    }
}

fn env_or(name: &str, fallback: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| fallback.to_string())
}

fn config_from_env() -> Config {
    Config {
        listen: env_or("TEAKSTA_LISTEN", "127.0.0.1:8080"),
        files_dir: env_or("TEAKSTA_FILES_DIR", "./data/fileUpload"),
        files_prm_dir: env_or("TEAKSTA_FILES_PRM_DIR", "./data/fileUpload/prm"),
        files_tmp_dir: env_or("TEAKSTA_FILES_TMP_DIR", "./data/fileUpload/tmp"),
        files_anl_dir: env_or("TEAKSTA_FILES_ANL_DIR", "./data/analyzedTexts"),
        secret_key: env_or("TEAKSTA_SECRET_KEY", "./data/secret_key"),
        webapp_root: PathBuf::from(env_or("TEAKSTA_WEBAPP_ROOT", ".")),
    }
}

/// The servlet instances plus the session table. Both handler entry points
/// take `&mut self` and a session holds type-erased values, so one lock
/// guards the whole request path and requests are served one at a time.
struct Container {
    werti: WertiServlet,
    upload: UploadDownloadFileServlet,
    sessions: HashMap<String, SessionRequest>,
}

struct AppState {
    container: Mutex<Container>,
    webapp_root: PathBuf,
}

impl AppState {
    /// A poisoned lock still holds a usable container: the servlets carry no
    /// invariant a panicking request could have broken half-way.
    fn container(&self) -> std::sync::MutexGuard<'_, Container> {
        match self.container.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

/// Runs the startup sequence in the order a container does: the context
/// listener publishes the upload directories before any servlet's `init`,
/// whatever order `web.xml` lists the two declarations in.
fn boot_container(config: &Config) -> Result<Container> {
    let mut event = ServletContextEvent {
        servlet_context: UploadContext {
            init_parameters: config.init_parameters(),
            attributes: BTreeMap::new(),
        },
    };
    FileLocationContextListener::new().context_initialized(&mut event)?;

    let mut upload = UploadDownloadFileServlet::new();
    upload.init(event.servlet_context.clone())?;

    let mut werti = WertiServlet::new();
    werti.init(ServletConfig {
        servlet_context: WertiContext {
            init_parameters: config.init_parameters(),
            servlet_context_name: Some(DISPLAY_NAME.to_string()),
        },
    })?;

    Ok(Container {
        werti,
        upload,
        sessions: HashMap::new(),
    })
}

fn new_session(webapp_root: &Path) -> SessionRequest {
    SessionRequest::new(HttpSession::new(SessionContext::new(Some(
        webapp_root.to_path_buf(),
    ))))
}

fn new_session_id() -> String {
    rand::rng()
        .sample_iter(Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

/// The session the request belongs to, and whether it had to be minted here.
fn session_key(req: &Request) -> (String, bool) {
    match cookie_value(req, SESSION_COOKIE) {
        Some(id) => (id, false),
        None => (new_session_id(), true),
    }
}

fn cookie_value(req: &Request, name: &str) -> Option<String> {
    let header = req.headers().get(COOKIE)?.to_str().ok()?;
    header.split(';').find_map(|entry| {
        let (key, value) = entry.trim().split_once('=')?;
        (key == name).then(|| value.to_string())
    })
}

fn with_session_cookie(mut response: Response, id: &str, fresh: bool) -> Response {
    if fresh && let Ok(value) = format!("{SESSION_COOKIE}={id}; Path=/; HttpOnly").parse() {
        response.headers_mut().insert(SET_COOKIE, value);
    }
    response
}

fn header_value(req: &Request, name: poem::http::HeaderName) -> Option<String> {
    req.headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
}

/// `getServerName`/`getServerPort` are read off the `Host` header, with the
/// port the scheme implies when the header carries none.
fn host_parts(req: &Request) -> (String, u16) {
    let default_port = if req.scheme() == &Scheme::HTTPS {
        443
    } else {
        80
    };
    let host = header_value(req, HOST).unwrap_or_default();
    match host.rsplit_once(':') {
        // A bracketed IPv6 literal with no port ends in `]`, and its colons
        // belong to the address.
        Some((name, port)) if !port.contains(']') => {
            (name.to_string(), port.parse().unwrap_or(default_port))
        }
        _ => (host, default_port),
    }
}

/// Splits an `application/x-www-form-urlencoded` string into its fields.
fn form_params(encoded: &str) -> BTreeMap<String, String> {
    let mut params = BTreeMap::new();
    for pair in encoded.split('&').filter(|pair| !pair.is_empty()) {
        let (name, value) = match pair.split_once('=') {
            Some((name, value)) => (name, value),
            None => (pair, ""),
        };
        params.insert(decode_form(name), decode_form(value));
    }
    params
}

fn decode_form(value: &str) -> String {
    // `+` stands for a space before percent-decoding, so an escaped `%2B`
    // survives as a literal plus.
    let spaced = value.replace('+', " ");
    let decoded = urlencoding::decode(&spaced)
        .map(|decoded| decoded.into_owned())
        .ok();
    decoded.unwrap_or(spaced)
}

/// Collects the request head — and a form-encoded body — into the model the
/// WERTi servlet reads. `getParameter` covers the query string and the posted
/// fields together, with the query string taking precedence over a field of
/// the same name.
fn werti_request_model(req: &Request, body: String) -> WertiRequest {
    let form_encoded = header_value(req, CONTENT_TYPE)
        .is_some_and(|content_type| content_type.starts_with("application/x-www-form-urlencoded"));
    let mut parameters = if form_encoded {
        form_params(&body)
    } else {
        BTreeMap::new()
    };
    parameters.extend(form_params(req.uri().query().unwrap_or_default()));

    let (server_name, server_port) = host_parts(req);
    let scheme = req.scheme().to_string();
    WertiRequest {
        request_url: format!(
            "{}://{}:{}{}",
            scheme,
            server_name,
            server_port,
            req.uri().path()
        ),
        parameters,
        body,
        scheme,
        server_name,
        server_port,
        // The application is served from the server root rather than from a
        // deployment context.
        context_path: String::new(),
        query_string: req.uri().query().map(str::to_string),
        character_encoding: None,
    }
}

/// The container splits a multipart body into parts before the servlet method
/// is entered. A body that is not multipart arrives with no parts, and the
/// servlet's own content-type check is what rejects it.
async fn upload_request_model(req: &Request, body: Body) -> UploadRequest {
    let content_type = header_value(req, CONTENT_TYPE);
    let empty = UploadRequest {
        method: req.method().to_string(),
        content_type: content_type.clone(),
        items: Vec::new(),
    };

    let Ok(multipart) = Multipart::from_request(req, &mut RequestBody::new(body)).await else {
        return empty;
    };
    match collect_multipart(content_type, multipart).await {
        Ok(model) => model,
        Err(error) => {
            warn!("Multipart body could not be collected: {}", error);
            empty
        }
    }
}

async fn read_body(body: Body) -> String {
    match body.into_bytes().await {
        Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        Err(error) => {
            warn!("Request body could not be read: {}", error);
            String::new()
        }
    }
}

/// What the container serves for a `ServletException`: the handler's partial
/// output is discarded and the failure is reported instead. The whole chain
/// is rendered, because several of the messages the handlers raise with are
/// the empty string and carry everything in the cause.
fn error_response(error: anyhow::Error) -> HttpServletResponse {
    error!("{:?}", error);
    let chain: Vec<String> = error
        .chain()
        .map(ToString::to_string)
        .filter(|link| !link.is_empty())
        .collect();
    HttpServletResponse {
        status: 500,
        content_type: Some("text/plain".to_string()),
        character_encoding: Some("UTF-8".to_string()),
        body: format!("{}\n", chain.join(": ")),
        ..Default::default()
    }
}

fn panic_response(join: tokio::task::JoinError) -> HttpServletResponse {
    error_response(anyhow!("request handler terminated: {join}"))
}

async fn run_werti(
    state: Arc<AppState>,
    mut model: WertiRequest,
    session_id: String,
    post_request: bool,
) -> HttpServletResponse {
    let joined = tokio::task::spawn_blocking(move || {
        let mut guard = state.container();
        let Container {
            werti, sessions, ..
        } = &mut *guard;
        let session = sessions
            .entry(session_id)
            .or_insert_with(|| new_session(&state.webapp_root));

        let mut response = HttpServletResponse::default();
        let handled = if post_request {
            werti.handle_post(&model, session, &mut response)
        } else {
            werti.handle_get(&mut model, session, &mut response)
        };
        match handled {
            Ok(()) => response,
            Err(error) => error_response(error),
        }
    })
    .await;

    joined.unwrap_or_else(panic_response)
}

async fn run_upload(state: Arc<AppState>, model: UploadRequest) -> HttpServletResponse {
    let joined = tokio::task::spawn_blocking(move || {
        let guard = state.container();
        let mut response = HttpServletResponse::default();
        match guard.upload.handle_post(&model, &mut response) {
            Ok(()) => response,
            Err(error) => error_response(error),
        }
    })
    .await;

    joined.unwrap_or_else(panic_response)
}

#[handler]
async fn index() -> Response {
    let body = concat!(
        "teaksta\n",
        "\n",
        "GET  /WERTiServlet\n",
        "POST /WERTiServlet\n",
        "POST /UploadDownloadFileServlet\n",
    );
    Response::builder()
        .header(CONTENT_TYPE, "text/plain;charset=UTF-8")
        .body(body)
}

#[handler]
async fn werti_get(req: &Request, state: Data<&Arc<AppState>>) -> Response {
    let (session_id, fresh) = session_key(req);
    let model = werti_request_model(req, String::new());
    let response = run_werti(state.0.clone(), model, session_id.clone(), false).await;
    with_session_cookie(response.into_response(), &session_id, fresh)
}

#[handler]
async fn werti_post(req: &Request, body: Body, state: Data<&Arc<AppState>>) -> Response {
    let (session_id, fresh) = session_key(req);
    let model = werti_request_model(req, read_body(body).await);
    let response = run_werti(state.0.clone(), model, session_id.clone(), true).await;
    with_session_cookie(response.into_response(), &session_id, fresh)
}

#[handler]
async fn upload_post(req: &Request, body: Body, state: Data<&Arc<AppState>>) -> Response {
    let model = upload_request_model(req, body).await;
    run_upload(state.0.clone(), model).await.into_response()
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = config_from_env();
    for name in [BUNDLE_ENV, GENERATOR_ENV] {
        if std::env::var(name).is_err() {
            warn!("{name} is not set; every analysis request will fail");
        }
    }
    info!("Webapp root {}", config.webapp_root.display());
    let classpath = classpath_root();
    if classpath.join("operators").is_dir() {
        info!("Descriptors under {}", classpath.display());
    } else {
        warn!(
            "No descriptor tree under {}; every topic will report itself unavailable",
            classpath.display()
        );
    }

    let state = Arc::new(AppState {
        container: Mutex::new(boot_container(&config)?),
        webapp_root: config.webapp_root.clone(),
    });

    let app = Route::new()
        .at("/", get(index))
        .at("/WERTiServlet", get(werti_get).post(werti_post))
        .at("/UploadDownloadFileServlet", post(upload_post))
        .data(state);

    info!("Listening on {}", config.listen);
    Server::new(TcpListener::bind(config.listen.clone()))
        .run(app)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use poem::http::Method;

    use super::*;

    fn get_request(uri: &str) -> Request {
        Request::builder()
            .uri_str(uri)
            .header(HOST, "example.org:8080")
            .finish()
    }

    #[test]
    fn form_params_decode_plus_and_escapes() {
        let params = form_params("activity=Substantive&url=a+b&client.enhancement=%C3%A1%2Bb&bare");

        assert_eq!(params["activity"], "Substantive");
        assert_eq!(params["url"], "a b");
        assert_eq!(params["client.enhancement"], "á+b");
        assert_eq!(params["bare"], "");
        assert_eq!(params.len(), 4);
    }

    #[test]
    fn request_model_reads_query_and_host() {
        let req = get_request("/WERTiServlet?url=example.org%2Fa&language=sme");
        let model = werti_request_model(&req, String::new());

        assert_eq!(model.get_parameter("url"), Some("example.org/a"));
        assert_eq!(model.get_parameter("language"), Some("sme"));
        assert_eq!(model.get_server_name(), "example.org");
        assert_eq!(model.get_server_port(), 8080);
        assert_eq!(model.get_scheme(), "http");
        assert_eq!(model.get_context_path(), "");
        assert_eq!(
            model.get_query_string(),
            Some("url=example.org%2Fa&language=sme")
        );
        assert_eq!(
            model.get_request_url(),
            "http://example.org:8080/WERTiServlet"
        );
    }

    #[test]
    fn request_model_merges_form_encoded_body() {
        let req = Request::builder()
            .method(Method::POST)
            .uri_str("/WERTiServlet?word=query")
            .header(HOST, "example.org")
            .content_type("application/x-www-form-urlencoded")
            .finish();
        let model = werti_request_model(&req, "word=body&correct=1".to_string());

        // The query string is read before the posted fields, so a name in
        // both is answered from the query.
        assert_eq!(model.get_parameter("word"), Some("query"));
        assert_eq!(model.get_parameter("correct"), Some("1"));
        assert_eq!(model.get_server_port(), 80);
    }

    #[test]
    fn request_model_ignores_body_without_form_type() {
        let req = Request::builder()
            .method(Method::POST)
            .uri_str("/WERTiServlet")
            .header(HOST, "example.org")
            .content_type("application/json")
            .finish();
        let model = werti_request_model(&req, "{\"version\":\"0.10\"}".to_string());

        assert!(model.get_parameter_names().next().is_none());
        assert_eq!(model.get_reader(), "{\"version\":\"0.10\"}");
    }

    #[test]
    fn session_key_reuses_the_request_cookie() {
        let with_cookie = Request::builder()
            .uri_str("/WERTiServlet")
            .header(COOKIE, "other=1; JSESSIONID=abc123; last=2")
            .finish();
        assert_eq!(session_key(&with_cookie), ("abc123".to_string(), false));

        let (minted, fresh) = session_key(&get_request("/WERTiServlet"));
        assert!(fresh);
        assert_eq!(minted.len(), 32);
    }

    #[test]
    fn error_response_is_plain_text_500() {
        let response = error_response(anyhow!("Webpage retrieval failed.")).into_response();

        assert_eq!(response.status(), 500);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            "text/plain;charset=UTF-8"
        );
    }

    #[tokio::test]
    async fn error_response_skips_empty_chain_links() {
        let inner = anyhow!("descriptor URL was not found");
        let response = error_response(inner.context("")).into_response();

        let body = response
            .into_body()
            .into_string()
            .await
            .expect("the reported chain");
        assert_eq!(body, "descriptor URL was not found\n");
    }

    #[test]
    fn session_cookie_is_set_only_when_fresh() {
        let fresh = with_session_cookie(Response::builder().body(()), "abc123", true);
        assert_eq!(
            fresh.headers().get(SET_COOKIE).unwrap(),
            "JSESSIONID=abc123; Path=/; HttpOnly"
        );

        let existing = with_session_cookie(Response::builder().body(()), "abc123", false);
        assert!(existing.headers().get(SET_COOKIE).is_none());
    }
}
