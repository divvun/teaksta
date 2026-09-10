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

#[cfg(test)]
mod tests {
    use super::*;

    // [spec:teaksta:sem:sme.src.main.java.werti.util.post-request.post-request.to-string-fn/test]
    #[test]
    fn display_renders_absent_members_as_the_literal_null() {
        let request = PostRequest::default();

        assert_eq!(
            request.to_string(),
            "PostRequest(\n  type = null\n  url = null\n  language = null\n  topic = null\n  activity = null\n  document = null\n  version = null\n)"
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.post-request.post-request.to-string-fn/test]
    #[test]
    fn display_renders_the_members_in_the_declared_order() {
        let request = PostRequest {
            r#type: Some("enhance".to_string()),
            url: Some("http://example.org/page?a=b&c=d".to_string()),
            language: Some("sme".to_string()),
            topic: Some("Nouns".to_string()),
            activity: Some("click".to_string()),
            document: Some("<p>&amp; raw\n  body</p>".to_string()),
            version: Some("0.9.1".to_string()),
        };

        assert_eq!(
            request.to_string(),
            "PostRequest(\n  type = enhance\n  url = http://example.org/page?a=b&c=d\n  language = sme\n  topic = Nouns\n  activity = click\n  document = <p>&amp; raw\n  body</p>\n  version = 0.9.1\n)"
        );
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.post-request.post-request.to-string-fn/test]
    #[test]
    fn display_embeds_a_large_document_verbatim() {
        let document = "á".repeat(4096);
        let request = PostRequest {
            document: Some(document.clone()),
            ..PostRequest::default()
        };

        let rendered = request.to_string();

        assert!(rendered.contains(&format!("\n  document = {}\n  version = null", document)));
        assert!(rendered.starts_with("PostRequest(\n  type = null\n"));
        assert!(rendered.ends_with("\n)"));
    }
}
