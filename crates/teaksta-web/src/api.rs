//! Typed client for the teaksta backend.
//!
//! Four endpoints answer this app: one names the topics and exercise modes on
//! offer, one hands back a whole enhanced page, one hands back the span map for
//! a page the caller already holds, and one takes a teacher's own text and hands
//! back the URL the other two read it from. Each is answered in a single
//! request.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

mod upload;

pub use upload::{
    MAX_UPLOAD_BYTES, Rejection, UploadFile, boundary_for, multipart_body, parse_upload, upload,
};

/// Where both enhancement endpoints live: the page one under GET, the span map
/// under POST.
pub const ENHANCE_PATH: &str = "/api/enhance";

/// Where the topic and mode registry lives.
pub const ACTIVITIES_PATH: &str = "/api/activities";

/// Where a teacher's own text is offered.
pub const UPLOAD_PATH: &str = "/api/upload";

/// The mode the entry form pre-selects. Every topic offers it, so it is always
/// a safe fallback when a request names a mode the backend does not know.
pub const DEFAULT_MODE: &str = "colorize";

/// Where the backend lives. The default base is empty, which makes every
/// request same-origin: what a deployment serving the app and the API together
/// wants.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Backend {
    base: String,
}

impl Backend {
    /// Point at a backend elsewhere, e.g. `https://gtweb.uit.no/teaksta`.
    /// A trailing slash is dropped so joining a path never doubles it.
    pub fn at(base: impl Into<String>) -> Self {
        let base = base.into();
        Self {
            base: base.trim_end_matches('/').to_string(),
        }
    }

    pub fn base(&self) -> &str {
        &self.base
    }

    /// The GET the entry form performs: the enhanced page for one URL, topic
    /// and mode.
    pub fn enhance_url(&self, request: &EnhanceRequest) -> String {
        format!(
            "{}{}?url={}&activity={}&mode={}",
            self.base,
            ENHANCE_PATH,
            urlencoding::encode(&request.url),
            urlencoding::encode(&request.activity),
            urlencoding::encode(&request.mode),
        )
    }

    /// Where the span map is asked for.
    pub fn spans_url(&self) -> String {
        format!("{}{}", self.base, ENHANCE_PATH)
    }

    /// Where the registry is read from.
    pub fn activities_url(&self) -> String {
        format!("{}{}", self.base, ACTIVITIES_PATH)
    }

    /// Where a text is offered.
    pub fn upload_url(&self) -> String {
        format!("{}{}", self.base, UPLOAD_PATH)
    }
}

/// One whole-page enhancement request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnhanceRequest {
    /// The web page the learner wants to practise on.
    pub url: String,
    /// The activity directory name, e.g. `VerbConjugation`.
    pub activity: String,
    /// The exercise asked for: colorize, click, mc or cloze.
    pub mode: String,
}

impl EnhanceRequest {
    pub fn new(url: impl Into<String>, activity: impl Into<String>, mode: &str) -> Self {
        Self {
            url: url.into(),
            activity: activity.into(),
            mode: mode.to_string(),
        }
    }

    /// Whether the backend has everything it needs, which it does not while a
    /// route is still being filled in.
    pub fn is_complete(&self) -> bool {
        !self.url.is_empty() && !self.activity.is_empty() && !self.mode.is_empty()
    }
}

/// The span endpoint's body: the page itself, or where the backend fetches it.
/// Exactly one of the two is sent, which is what the endpoint accepts.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpanRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub activity: String,
    pub mode: String,
}

impl SpanRequest {
    /// Ask for the spans of a page the caller already holds.
    pub fn inline(html: impl Into<String>, activity: impl Into<String>, mode: &str) -> Self {
        Self {
            html: Some(html.into()),
            url: None,
            activity: activity.into(),
            mode: mode.to_string(),
        }
    }

    /// Ask for the spans of a page the backend fetches itself.
    pub fn fetched(url: impl Into<String>, activity: impl Into<String>, mode: &str) -> Self {
        Self {
            html: None,
            url: Some(url.into()),
            activity: activity.into(),
            mode: mode.to_string(),
        }
    }
}

/// The span endpoint's reply: the position each enhanced fragment covers in the
/// page, to the markup that replaces it. Keys are strings because JSON object
/// keys are.
pub type EnhancedSpans = BTreeMap<String, String>;

/// One topic the backend offers.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct Activity {
    /// The activity directory name, e.g. `NegVerbs`.
    pub name: String,
    /// The North Sámi display name, absent for a topic with none.
    #[serde(default)]
    pub label: Option<String>,
    /// Whether the deployment has the topic switched on.
    #[serde(default)]
    pub enabled: bool,
}

/// One exercise mode the backend offers.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct Mode {
    /// The `mode` parameter value: colorize, click, mc or cloze.
    pub name: String,
    /// The North Sámi instruction shown to the learner.
    #[serde(default)]
    pub label: Option<String>,
}

/// What the backend offers. Modes are declared once for the whole registry,
/// so every topic is asked for with any of them.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct Registry {
    #[serde(default)]
    pub activities: Vec<Activity>,
    #[serde(default)]
    pub modes: Vec<Mode>,
}

impl Registry {
    pub fn activity(&self, name: &str) -> Option<&Activity> {
        self.activities.iter().find(|item| item.name == name)
    }

    pub fn mode(&self, name: &str) -> Option<&Mode> {
        self.modes.iter().find(|item| item.name == name)
    }

    /// A topic's North Sámi name, or the directory name when it has none.
    pub fn activity_label<'a>(&'a self, name: &'a str) -> &'a str {
        label_of(
            self.activity(name).and_then(|item| item.label.as_deref()),
            name,
        )
    }

    /// A mode's North Sámi instruction, or the parameter value itself.
    pub fn mode_label<'a>(&'a self, name: &'a str) -> &'a str {
        label_of(self.mode(name).and_then(|item| item.label.as_deref()), name)
    }
}

fn label_of<'a>(label: Option<&'a str>, name: &'a str) -> &'a str {
    label.filter(|label| !label.is_empty()).unwrap_or(name)
}

/// Decode a registry reply.
pub fn parse_registry(body: &str) -> Result<Registry, ApiError> {
    serde_json::from_str(body).map_err(|error| ApiError::Malformed(error.to_string()))
}

/// Decode a span-map reply.
pub fn parse_spans(body: &str) -> Result<EnhancedSpans, ApiError> {
    serde_json::from_str(body).map_err(|error| ApiError::Malformed(error.to_string()))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApiError {
    /// The request never produced a response.
    Network(String),
    /// The backend answered, but not with success.
    Status(u16),
    /// The body did not parse as the protocol says it should.
    Malformed(String),
    /// The request lacks a parameter the backend requires, so it was never
    /// sent.
    Incomplete,
    /// An offered text did not pass one of the upload endpoint's gates.
    Rejected(Rejection),
    /// No fetch client exists outside the browser.
    Unsupported,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Network(detail) => write!(f, "network error: {detail}"),
            ApiError::Status(code) => write!(f, "backend answered {code}"),
            ApiError::Malformed(detail) => write!(f, "malformed response: {detail}"),
            ApiError::Incomplete => write!(f, "no topic, exercise mode or page was given"),
            ApiError::Rejected(rejection) => {
                write!(f, "the text was turned away: {}", rejection.gloss())
            }
            ApiError::Unsupported => write!(f, "no fetch client outside the browser"),
        }
    }
}

impl std::error::Error for ApiError {}

/// The topics and modes the backend offers. A registry naming neither is no
/// answer the picker can be built from, so it is refused here rather than
/// shown as an empty form.
pub async fn fetch_registry(backend: &Backend) -> Result<Registry, ApiError> {
    let registry = parse_registry(&get_text(&backend.activities_url()).await?)?;

    if registry.activities.is_empty() || registry.modes.is_empty() {
        return Err(ApiError::Malformed(
            "the registry names no topics or no modes".to_string(),
        ));
    }

    Ok(registry)
}

/// One enhanced page, as HTML, in a single request.
pub async fn fetch_enhanced(
    backend: &Backend,
    request: &EnhanceRequest,
) -> Result<String, ApiError> {
    if !request.is_complete() {
        return Err(ApiError::Incomplete);
    }

    get_text(&backend.enhance_url(request)).await
}

/// The span map for a page the caller already holds. The endpoint takes
/// exactly one of the two sources, so a body naming both or neither is refused
/// here instead of being sent for a refusal.
pub async fn fetch_spans(
    backend: &Backend,
    request: &SpanRequest,
) -> Result<EnhancedSpans, ApiError> {
    if request.html.is_some() == request.url.is_some() {
        return Err(ApiError::Incomplete);
    }

    parse_spans(&post_json(&backend.spans_url(), request).await?)
}

#[cfg(target_arch = "wasm32")]
async fn get_text(url: &str) -> Result<String, ApiError> {
    let response = gloo_net::http::Request::get(url)
        .send()
        .await
        .map_err(|error| ApiError::Network(error.to_string()))?;

    read_body(response).await
}

#[cfg(target_arch = "wasm32")]
async fn post_json(url: &str, body: &SpanRequest) -> Result<String, ApiError> {
    let payload =
        serde_json::to_string(body).map_err(|error| ApiError::Malformed(error.to_string()))?;
    let response = gloo_net::http::Request::post(url)
        .header("Content-Type", "application/json")
        .body(payload)
        .map_err(|error| ApiError::Network(error.to_string()))?
        .send()
        .await
        .map_err(|error| ApiError::Network(error.to_string()))?;

    read_body(response).await
}

#[cfg(target_arch = "wasm32")]
async fn read_body(response: gloo_net::http::Response) -> Result<String, ApiError> {
    if !response.ok() {
        return Err(ApiError::Status(response.status()));
    }

    response
        .text()
        .await
        .map_err(|error| ApiError::Network(error.to_string()))
}

#[cfg(not(target_arch = "wasm32"))]
async fn get_text(_url: &str) -> Result<String, ApiError> {
    Err(ApiError::Unsupported)
}

#[cfg(not(target_arch = "wasm32"))]
async fn post_json(_url: &str, _body: &SpanRequest) -> Result<String, ApiError> {
    Err(ApiError::Unsupported)
}

#[cfg(test)]
mod tests {
    use super::*;

    const REGISTRY: &str = concat!(
        r#"{"activities":[{"name":"Substantive","label":"Substantiivvat","enabled":true},"#,
        r#"{"name":"Preps","label":null,"enabled":false}],"#,
        r#""modes":[{"name":"colorize","label":"Geahča ivdnejuvvon sániid."},"#,
        r#"{"name":"cloze","label":"Čále rivttes sániid!"}]}"#
    );

    fn sample() -> EnhanceRequest {
        EnhanceRequest::new(
            "http://example.org/artihkal?id=7&p=2",
            "VerbConjugation",
            "cloze",
        )
    }

    #[test]
    fn same_origin_builds_a_root_relative_url() {
        let url = Backend::default().enhance_url(&sample());

        assert!(url.starts_with("/api/enhance?"));
        assert_eq!(Backend::default().activities_url(), "/api/activities");
        assert_eq!(Backend::default().spans_url(), "/api/enhance");
    }

    #[test]
    fn the_query_carries_the_endpoint_parameters() {
        let url = Backend::default().enhance_url(&sample());

        assert_eq!(
            url,
            "/api/enhance?url=http%3A%2F%2Fexample.org%2Fartihkal%3Fid%3D7%26p%3D2\
             &activity=VerbConjugation&mode=cloze"
        );
    }

    #[test]
    fn a_remote_base_loses_its_trailing_slash() {
        let backend = Backend::at("https://gtweb.uit.no/teaksta/");

        assert_eq!(backend.base(), "https://gtweb.uit.no/teaksta");
        assert!(
            backend
                .enhance_url(&sample())
                .starts_with("https://gtweb.uit.no/teaksta/api/enhance?")
        );
    }

    #[test]
    fn a_request_missing_a_parameter_is_incomplete() {
        assert!(sample().is_complete());
        assert!(!EnhanceRequest::new("", "Subject", "click").is_complete());
        assert!(!EnhanceRequest::new("http://a.example", "", "click").is_complete());
        assert!(!EnhanceRequest::new("http://a.example", "Subject", "").is_complete());
    }

    #[test]
    fn a_span_request_sends_one_source() {
        let inline =
            serde_json::to_string(&SpanRequest::inline("<p>a</p>", "Object", "mc")).unwrap();
        let fetched =
            serde_json::to_string(&SpanRequest::fetched("http://a.example", "Object", "mc"))
                .unwrap();

        assert_eq!(
            inline,
            r#"{"html":"<p>a</p>","activity":"Object","mode":"mc"}"#
        );
        assert_eq!(
            fetched,
            r#"{"url":"http://a.example","activity":"Object","mode":"mc"}"#
        );
    }

    #[test]
    fn the_registry_parses_into_topics_and_modes() {
        let registry = parse_registry(REGISTRY).unwrap();

        assert_eq!(registry.activities.len(), 2);
        assert_eq!(registry.modes.len(), 2);
        assert!(registry.activity("Substantive").unwrap().enabled);
        assert!(!registry.activity("Preps").unwrap().enabled);
        assert_eq!(registry.activity("Nonesuch"), None);
    }

    #[test]
    fn a_label_falls_back_to_the_name() {
        let registry = parse_registry(REGISTRY).unwrap();

        assert_eq!(registry.activity_label("Substantive"), "Substantiivvat");
        assert_eq!(registry.activity_label("Preps"), "Preps");
        assert_eq!(registry.activity_label("Nonesuch"), "Nonesuch");
        assert_eq!(registry.mode_label("cloze"), "Čále rivttes sániid!");
        assert_eq!(registry.mode_label("click"), "click");
    }

    #[test]
    fn an_empty_registry_still_parses() {
        let registry = parse_registry("{}").unwrap();

        assert_eq!(registry, Registry::default());
        assert_eq!(registry.mode(DEFAULT_MODE), None);
    }

    #[test]
    fn spans_parse_keyed_by_document_position() {
        let spans =
            parse_spans(r#"{"11":"<span>boaris</span>","24":"<span>beana</span>"}"#).unwrap();

        assert_eq!(spans.len(), 2);
        assert_eq!(spans.get("11").unwrap(), "<span>boaris</span>");
    }

    #[test]
    fn a_non_json_body_reports_a_malformed_reply() {
        let error = parse_spans("<html>").unwrap_err();

        assert!(matches!(error, ApiError::Malformed(_)));
        assert!(error.to_string().starts_with("malformed response: "));
        assert!(matches!(
            parse_registry("<html>").unwrap_err(),
            ApiError::Malformed(_)
        ));
    }
}
