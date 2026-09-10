//! The exercise the enhancement pass is producing: colorize, click, mc or
//! cloze.
//!
//! The postprocessing enhancers decide what to attach to a token by reading
//! this, and they are reached through the analysis flow rather than called
//! directly, so the choice travels beside the request rather than through it.
//! One value is shared by the whole process: concurrent requests asking for
//! different exercises would see each other's, which is why the handler layer
//! serialises analysis.

use std::sync::RwLock;

use anyhow::{Result, anyhow};

/// The exercise the enhancers read. Unset until a request publishes one.
// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.enhancement-type]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.enhancement-type]
pub static SELECTED: RwLock<Option<String>> = RwLock::new(None);

/// Hands the requested exercise to the enhancers.
pub fn publish(exercise: Option<&str>) -> Result<()> {
    *SELECTED
        .write()
        .map_err(|_| anyhow!("enhancement_type lock poisoned"))? = exercise.map(str::to_string);
    Ok(())
}

/// What the enhancers read. An unpublished exercise reads as the empty
/// string, which matches none of the four and therefore attaches nothing.
pub fn selected() -> String {
    SELECTED
        .read()
        .ok()
        .and_then(|exercise| exercise.clone())
        .unwrap_or_default()
}
