//! Typed client for the teaksta backend.
//!
//! Three endpoints answer this app: one names the topics and exercise modes on
//! offer, one hands back the analysed text of a page block by block, and one
//! takes a teacher's own text and hands back the URL the second reads it from.
//! Each is answered in a single request.
//!
//! The backend also answers a whole enhanced page and a per-token span map,
//! and this app asks for neither. The page is a foreign document this app has
//! no business rendering, and the span map is the browser add-on's protocol:
//! its entries are keyed by positions in an analysed document text the caller
//! never receives, and they carry the matched word forms alone — no prose, no
//! punctuation, no block structure — so no exercise can be built from them.

use std::fmt;

use serde::{Deserialize, Serialize};

mod upload;

pub use upload::{
    MAX_UPLOAD_BYTES, Rejection, UploadFile, boundary_for, multipart_body, parse_upload, upload,
};

/// Where the analysed text is asked for, block by block.
pub const BLOCKS_PATH: &str = "/api/enhance/blocks";

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

    /// Where the analysed text is asked for.
    pub fn blocks_url(&self) -> String {
        format!("{}{}", self.base, BLOCKS_PATH)
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

/// One request for the analysed text of a page: the page itself, or where the
/// backend fetches it. Exactly one of the two is sent, which is what the
/// endpoint accepts.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// The activity directory name, e.g. `VerbConjugation`.
    pub activity: String,
    /// The exercise asked for: colorize, click, mc or cloze.
    pub mode: String,
}

impl BlockRequest {
    /// Ask for the analysed text of a page the backend fetches itself, which
    /// is what a learner naming a web page asks for.
    pub fn fetched(url: impl Into<String>, activity: impl Into<String>, mode: &str) -> Self {
        Self {
            html: None,
            url: Some(url.into()),
            activity: activity.into(),
            mode: mode.to_string(),
        }
    }

    /// Ask for the analysed text of a page the caller already holds.
    pub fn inline(html: impl Into<String>, activity: impl Into<String>, mode: &str) -> Self {
        Self {
            html: Some(html.into()),
            url: None,
            activity: activity.into(),
            mode: mode.to_string(),
        }
    }

    /// Whether the backend has everything it needs, which it does not while a
    /// route is still being filled in. Exactly one source, and a source that
    /// holds something: a route carrying an empty `url` names no page.
    pub fn is_complete(&self) -> bool {
        let source = match (&self.html, &self.url) {
            (Some(html), None) => !html.is_empty(),
            (None, Some(url)) => !url.is_empty(),
            _ => false,
        };

        source && !self.activity.is_empty() && !self.mode.is_empty()
    }
}

/// One block of the analysed text: the prose of that block with the topic's
/// spans already standing in it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct TextBlock {
    pub html: String,
}

/// The block endpoint's reply: every block of the analysed text, in document
/// order.
pub type EnhancedText = Vec<TextBlock>;

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

/// Decode a block reply.
pub fn parse_blocks(body: &str) -> Result<EnhancedText, ApiError> {
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

/// The analysed text of one page, block by block, in a single request. The
/// endpoint takes exactly one of the two sources, so a body naming both or
/// neither is refused here instead of being sent for a refusal.
pub async fn fetch_blocks(
    backend: &Backend,
    request: &BlockRequest,
) -> Result<EnhancedText, ApiError> {
    if !request.is_complete() {
        return Err(ApiError::Incomplete);
    }

    parse_blocks(&post_json(&backend.blocks_url(), request).await?)
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
async fn post_json(url: &str, body: &BlockRequest) -> Result<String, ApiError> {
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
async fn post_json(_url: &str, _body: &BlockRequest) -> Result<String, ApiError> {
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

    fn sample() -> BlockRequest {
        BlockRequest::fetched(
            "http://example.org/artihkal?id=7&p=2",
            "VerbConjugation",
            "cloze",
        )
    }

    #[test]
    fn same_origin_builds_a_root_relative_url() {
        assert_eq!(Backend::default().blocks_url(), "/api/enhance/blocks");
        assert_eq!(Backend::default().activities_url(), "/api/activities");
        assert_eq!(Backend::default().upload_url(), "/api/upload");
    }

    #[test]
    fn a_remote_base_loses_its_trailing_slash() {
        let backend = Backend::at("https://gtweb.uit.no/teaksta/");

        assert_eq!(backend.base(), "https://gtweb.uit.no/teaksta");
        assert_eq!(
            backend.blocks_url(),
            "https://gtweb.uit.no/teaksta/api/enhance/blocks"
        );
    }

    #[test]
    fn a_block_request_sends_one_source() {
        let inline =
            serde_json::to_string(&BlockRequest::inline("<p>a</p>", "Object", "mc")).unwrap();
        let fetched =
            serde_json::to_string(&BlockRequest::fetched("http://a.example", "Object", "mc"))
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

    /// The endpoint takes exactly one of the two sources and needs both the
    /// topic and the exercise, so a request short of any of that never leaves
    /// the browser.
    #[test]
    fn a_request_missing_a_parameter_is_incomplete() {
        assert!(sample().is_complete());
        assert!(!BlockRequest::fetched("", "Subject", "click").is_complete());
        assert!(!BlockRequest::fetched("http://a.example", "", "click").is_complete());
        assert!(!BlockRequest::fetched("http://a.example", "Subject", "").is_complete());
        assert!(!BlockRequest::default().is_complete());

        let both = BlockRequest {
            html: Some("<p>a</p>".to_string()),
            url: Some("http://a.example".to_string()),
            activity: "Subject".to_string(),
            mode: "click".to_string(),
        };
        assert!(!both.is_complete());
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
    fn blocks_parse_in_document_order() {
        let blocks =
            parse_blocks(r#"[{"html":"<h1>Beana</h1>"},{"html":"<p>Boaris beana.</p>"}]"#).unwrap();

        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].html, "<h1>Beana</h1>");
        assert_eq!(blocks[1].html, "<p>Boaris beana.</p>");
        assert_eq!(parse_blocks("[]").unwrap(), Vec::new());
    }

    #[test]
    fn a_non_json_body_reports_a_malformed_reply() {
        let error = parse_blocks("<html>").unwrap_err();

        assert!(matches!(error, ApiError::Malformed(_)));
        assert!(error.to_string().starts_with("malformed response: "));
        assert!(matches!(
            parse_registry("<html>").unwrap_err(),
            ApiError::Malformed(_)
        ));
    }
}
