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
