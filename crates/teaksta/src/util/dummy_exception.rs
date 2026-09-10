//! A DummyException is used to throw an exception in order to move to a
//! certain point in the code, e.g., a finally block.
//!
//! Author: Marion Zepf

use std::error::Error;
use std::fmt;

// [spec:teaksta:def:sme.src.main.java.werti.util.dummy-exception.dummy-exception]
#[derive(Debug)]
pub struct DummyException {
    message: Option<String>,
    source: Option<Box<dyn Error + Send + Sync + 'static>>,
}

impl DummyException {
    // [spec:teaksta:def:sme.src.main.java.werti.util.dummy-exception.dummy-exception.dummy-exception-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.dummy-exception.dummy-exception.dummy-exception-fn]
    pub fn new() -> Self {
        DummyException {
            message: None,
            source: None,
        }
    }

    pub fn with_message(message: impl Into<String>) -> Self {
        DummyException {
            message: Some(message.into()),
            source: None,
        }
    }

    pub fn with_cause(cause: impl Error + Send + Sync + 'static) -> Self {
        // Exception(Throwable) adopts the cause's own string form as its
        // detail message.
        DummyException {
            message: Some(cause.to_string()),
            source: Some(Box::new(cause)),
        }
    }

    pub fn with_message_and_cause(
        message: impl Into<String>,
        cause: impl Error + Send + Sync + 'static,
    ) -> Self {
        DummyException {
            message: Some(message.into()),
            source: Some(Box::new(cause)),
        }
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

impl Default for DummyException {
    fn default() -> Self {
        DummyException::new()
    }
}

impl fmt::Display for DummyException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.message {
            Some(message) => write!(f, "DummyException: {message}"),
            None => write!(f, "DummyException"),
        }
    }
}

impl Error for DummyException {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_deref().map(|e| e as &(dyn Error + 'static))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // [spec:teaksta:sem:sme.src.main.java.werti.util.dummy-exception.dummy-exception.dummy-exception-fn/test]
    #[test]
    fn dummy_exception_new_carries_no_payload() {
        let e = DummyException::new();
        assert!(e.message().is_none());
        assert!(e.source().is_none());
        assert_eq!(e.to_string(), "DummyException");

        // The no-argument form is what `Default` produces too.
        let d = DummyException::default();
        assert!(d.message().is_none());
        assert!(d.source().is_none());
        assert_eq!(d.to_string(), "DummyException");

        // Usable purely as a control-flow signal that forces execution out of
        // a block and into the enclosing handler.
        let mut reached_cleanup = false;
        let outcome: Result<(), DummyException> = (|| {
            let result = Err(DummyException::new());
            reached_cleanup = true;
            result
        })();
        assert!(reached_cleanup);
        assert!(outcome.is_err());
        assert!(outcome.unwrap_err().message().is_none());
    }

    #[test]
    fn dummy_exception_alternate_forms_carry_message_and_cause() {
        let with_message = DummyException::with_message("stop here");
        assert_eq!(with_message.message(), Some("stop here"));
        assert!(with_message.source().is_none());
        assert_eq!(with_message.to_string(), "DummyException: stop here");

        let with_cause = DummyException::with_cause(DummyException::with_message("inner"));
        assert_eq!(with_cause.message(), Some("DummyException: inner"));
        assert_eq!(
            with_cause.source().map(|s| s.to_string()),
            Some("DummyException: inner".to_string())
        );

        let both = DummyException::with_message_and_cause("outer", DummyException::new());
        assert_eq!(both.message(), Some("outer"));
        assert_eq!(
            both.source().map(|s| s.to_string()),
            Some("DummyException".to_string())
        );
    }
}
