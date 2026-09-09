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
