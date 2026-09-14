//! What the suites that read the server's own log lines share.
//!
//! A test that asks what was logged has to install a subscriber and then read
//! back what it wrote, and the reading half is the same wherever the question
//! is asked. It lives here so the two suites that ask are not two copies of
//! one writer.

use std::io::Write;
use std::sync::{Arc, Mutex};

use tracing_subscriber::fmt::MakeWriter;

/// Whatever the subscriber wrote, so a line can be read back rather than only
/// looked at.
#[derive(Clone, Default)]
pub struct Recorded(Arc<Mutex<Vec<u8>>>);

impl Recorded {
    pub fn read(&self) -> String {
        String::from_utf8(self.0.lock().expect("the log").clone()).expect("a readable log")
    }

    /// A subscriber writing here, installed for this thread until the guard
    /// is dropped. Info level, because that is where the server writes the
    /// lines an operator reads, and without ANSI, because what is wanted is
    /// the text and not a rendering of it.
    pub fn recording(&self) -> tracing::subscriber::DefaultGuard {
        let subscriber = tracing_subscriber::fmt()
            .with_writer(self.clone())
            .with_ansi(false)
            .with_max_level(tracing::Level::INFO)
            .finish();
        tracing::subscriber::set_default(subscriber)
    }
}

impl Write for Recorded {
    fn write(&mut self, written: &[u8]) -> std::io::Result<usize> {
        self.0.lock().expect("the log").extend_from_slice(written);
        Ok(written.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for Recorded {
    type Writer = Recorded;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}
