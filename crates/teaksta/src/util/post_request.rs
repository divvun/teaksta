//! Corresponds to the structure of the JSON AJAX requests sent by the add-on.
//!
//! Author: Adriane Boyd
//!
//! Gson leaves a member absent from the JSON payload as a null reference, so
//! every field is optional here and a missing member deserialises to `None`
//! rather than failing the parse.

use std::fmt;

use serde::{Deserialize, Serialize};

// [spec:teaksta:def:sme.src.main.java.werti.util.post-request.post-request]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PostRequest {
    #[serde(rename = "type")]
    pub r#type: Option<String>,
    pub url: Option<String>,
    pub language: Option<String>,
    pub topic: Option<String>,
    pub activity: Option<String>,
    pub document: Option<String>,
    pub version: Option<String>,
}

/// Java renders a null reference as the four characters `null` when it is
/// appended to a `StringBuilder`.
fn null_str(value: Option<&String>) -> &str {
    match value {
        Some(value) => value.as_str(),
        None => "null",
    }
}

// [spec:teaksta:def:sme.src.main.java.werti.util.post-request.post-request.to-string-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.util.post-request.post-request.to-string-fn]
impl fmt::Display for PostRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut sb = String::new();
        sb.push_str("PostRequest(");
        sb.push_str("\n  type = ");
        sb.push_str(null_str(self.r#type.as_ref()));
        sb.push_str("\n  url = ");
        sb.push_str(null_str(self.url.as_ref()));
        sb.push_str("\n  language = ");
        sb.push_str(null_str(self.language.as_ref()));
        sb.push_str("\n  topic = ");
        sb.push_str(null_str(self.topic.as_ref()));
        sb.push_str("\n  activity = ");
        sb.push_str(null_str(self.activity.as_ref()));
        sb.push_str("\n  document = ");
        sb.push_str(null_str(self.document.as_ref()));
        sb.push_str("\n  version = ");
        sb.push_str(null_str(self.version.as_ref()));
        sb.push_str("\n)");
        write!(f, "{}", sb)
    }
}
