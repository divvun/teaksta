//! Methods needed for processing practice response.
//!
//! Author: Adriane Boyd

use crate::util::post_request::PostRequest;

// [spec:teaksta:def:sme.src.main.java.werti.util.practice-handler.practice-handler]
pub struct PracticeHandler<'a> {
    /// The request is retained by reference, as in the original, so a later
    /// mutation by the caller is visible through the handler. The field is
    /// never read — `process` does not look at it — and is public only so the
    /// unread field is reachable rather than suppressed.
    pub request_info: &'a PostRequest,
}

impl<'a> PracticeHandler<'a> {
    // [spec:teaksta:def:sme.src.main.java.werti.util.practice-handler.practice-handler.practice-handler-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.practice-handler.practice-handler.practice-handler-fn]
    pub fn new(a_request_info: &'a PostRequest) -> Self {
        PracticeHandler {
            request_info: a_request_info,
        }
    }

    // [spec:teaksta:def:sme.src.main.java.werti.util.practice-handler.practice-handler.process-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.practice-handler.practice-handler.process-fn]
    pub fn process(&self) -> String {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn populated_request() -> PostRequest {
        PostRequest {
            r#type: Some("practice".to_string()),
            url: Some("http://example.org/page".to_string()),
            language: Some("sme".to_string()),
            topic: Some("Nouns".to_string()),
            activity: Some("click".to_string()),
            document: Some("<html><body>Sápmi</body></html>".to_string()),
            version: Some("0.9.1".to_string()),
        }
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.practice-handler.practice-handler.practice-handler-fn/test]
    #[test]
    fn constructor_retains_the_request_by_reference() {
        let request = populated_request();

        let handler = PracticeHandler::new(&request);

        assert!(std::ptr::eq(handler.request_info, &request));
        assert_eq!(handler.request_info.topic.as_deref(), Some("Nouns"));
        assert_eq!(handler.request_info.language.as_deref(), Some("sme"));
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.practice-handler.practice-handler.practice-handler-fn/test]
    #[test]
    fn constructor_accepts_a_request_with_every_member_absent() {
        let request = PostRequest::default();

        let handler = PracticeHandler::new(&request);

        assert!(std::ptr::eq(handler.request_info, &request));
        assert!(handler.request_info.activity.is_none());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.practice-handler.practice-handler.process-fn/test]
    #[test]
    fn process_returns_empty_string_for_any_request() {
        let populated = populated_request();
        let empty = PostRequest::default();

        assert_eq!(PracticeHandler::new(&populated).process(), "");
        assert_eq!(PracticeHandler::new(&empty).process(), "");
    }
}
