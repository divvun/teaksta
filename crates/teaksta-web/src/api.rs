//! Typed client for the Konteaksta backend.
//!
//! Two protocols share one backend: the servlet endpoint the entry form has
//! always used, which answers a whole enhanced page as HTML, and the JSON
//! protocol the browser add-on uses to enhance a page it already has.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

/// The endpoint the entry form submits to.
pub const SERVLET_PATH: &str = "/WERTiServlet";

/// The sme activities declare `<lang code="en">` in their `activity.xml`
/// because no tokenizer is registered under `sme`, so the form has always
/// sent `en` for every North Sámi topic.
pub const SERVLET_LANGUAGE: &str = "en";

/// Where the backend lives. The default base is empty, which makes every
/// request same-origin: what a deployment serving the app and the servlet
/// together wants.
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
    /// and exercise type.
    pub fn enhance_url(&self, request: &EnhanceRequest) -> String {
        format!(
            "{}{}?url={}&activity={}&client.enhancement={}&language={}",
            self.base,
            SERVLET_PATH,
            urlencoding::encode(&request.url),
            urlencoding::encode(&request.activity),
            urlencoding::encode(&request.enhancement),
            urlencoding::encode(&request.language),
        )
    }
}

/// One enhancement request against the servlet endpoint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnhanceRequest {
    /// The web page the learner wants to practise on.
    pub url: String,
    /// The activity directory name, e.g. `VerbConjugation`.
    pub activity: String,
    /// The `client.enhancement` value: colorize, click, mc or cloze.
    pub enhancement: String,
    /// The pipeline language code.
    pub language: String,
}

impl EnhanceRequest {
    pub fn new(url: impl Into<String>, activity: impl Into<String>, enhancement: &str) -> Self {
        Self {
            url: url.into(),
            activity: activity.into(),
            enhancement: enhancement.to_string(),
            language: SERVLET_LANGUAGE.to_string(),
        }
    }
}

/// The add-on's JSON request body: it ships the page it has already fetched
/// and asks only for the enhanced spans back.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct EnhanceJsonRequest {
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub url: Option<String>,
    pub language: Option<String>,
    pub topic: Option<String>,
    pub activity: Option<String>,
    pub document: Option<String>,
    pub version: Option<String>,
}

/// The add-on's JSON reply: enhancement id to the replacement markup for the
/// span it identifies. Ids are strings because JSON object keys are.
pub type EnhancedSpans = BTreeMap<String, String>;

/// Decode a JSON enhancement reply into its spans.
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
    /// The request lacks a parameter the servlet requires, so it was never
    /// sent.
    Incomplete,
    /// No fetch client exists outside the browser.
    Unsupported,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Network(detail) => write!(f, "network error: {detail}"),
            ApiError::Status(code) => write!(f, "backend answered {code}"),
            ApiError::Malformed(detail) => write!(f, "malformed response: {detail}"),
            ApiError::Incomplete => write!(f, "no topic, exercise type or page was given"),
            ApiError::Unsupported => write!(f, "no fetch client outside the browser"),
        }
    }
}

impl std::error::Error for ApiError {}

/// Fetch the enhanced page for a request, as HTML.
#[cfg(target_arch = "wasm32")]
pub async fn fetch_enhanced(
    backend: &Backend,
    request: &EnhanceRequest,
) -> Result<String, ApiError> {
    let response = gloo_net::http::Request::get(&backend.enhance_url(request))
        .send()
        .await
        .map_err(|error| ApiError::Network(error.to_string()))?;

    if !response.ok() {
        return Err(ApiError::Status(response.status()));
    }

    response
        .text()
        .await
        .map_err(|error| ApiError::Network(error.to_string()))
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_enhanced(
    _backend: &Backend,
    _request: &EnhanceRequest,
) -> Result<String, ApiError> {
    Err(ApiError::Unsupported)
}

#[cfg(test)]
mod tests {
    use super::*;

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

        assert!(url.starts_with("/WERTiServlet?"));
    }

    #[test]
    fn the_query_carries_the_servlet_parameter_names() {
        let url = Backend::default().enhance_url(&sample());

        assert_eq!(
            url,
            "/WERTiServlet?url=http%3A%2F%2Fexample.org%2Fartihkal%3Fid%3D7%26p%3D2\
             &activity=VerbConjugation&client.enhancement=cloze&language=en"
        );
    }

    #[test]
    fn a_remote_base_loses_its_trailing_slash() {
        let backend = Backend::at("https://gtweb.uit.no/teaksta/");

        assert_eq!(backend.base(), "https://gtweb.uit.no/teaksta");
        assert!(
            backend
                .enhance_url(&sample())
                .starts_with("https://gtweb.uit.no/teaksta/WERTiServlet?")
        );
    }

    #[test]
    fn requests_default_to_the_servlet_language() {
        assert_eq!(sample().language, "en");
    }

    #[test]
    fn absent_json_members_deserialise_to_none() {
        let request: EnhanceJsonRequest = serde_json::from_str(r#"{"url":"http://a.example"}"#)
            .expect("the add-on protocol tolerates absent members");

        assert_eq!(request.url.as_deref(), Some("http://a.example"));
        assert_eq!(request.topic, None);
        assert_eq!(request.document, None);
    }

    #[test]
    fn the_json_type_member_serialises_as_type() {
        let request = EnhanceJsonRequest {
            kind: Some("enhance".to_string()),
            ..EnhanceJsonRequest::default()
        };

        let json = serde_json::to_string(&request).unwrap();

        assert!(json.contains(r#""type":"enhance""#));
    }

    #[test]
    fn spans_parse_keyed_by_enhancement_id() {
        let spans = parse_spans(r#"{"1":"<span>boaris</span>","2":"<span>beana</span>"}"#).unwrap();

        assert_eq!(spans.len(), 2);
        assert_eq!(spans.get("1").unwrap(), "<span>boaris</span>");
    }

    #[test]
    fn a_non_json_body_reports_a_malformed_reply() {
        let error = parse_spans("<html>").unwrap_err();

        assert!(matches!(error, ApiError::Malformed(_)));
        assert!(error.to_string().starts_with("malformed response: "));
    }
}
