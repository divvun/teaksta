//! A small helper class to help WERTi communicate with the outside world.
//!
//! Author: Aleksandar Dimitrov

use std::path::PathBuf;

use anyhow::{Result, anyhow};
use tracing::debug;

// [spec:teaksta:def:sme.src.main.java.werti.util.resources.resources]

/// Stand-in for the `javax.servlet.ServletContext` resource lookup: resolves a
/// context-root-relative path to a `file:` URL.
pub struct ServletContext {
    pub root: PathBuf,
}

impl ServletContext {
    pub fn get_resource(&self, path: &str) -> Result<String> {
        let resolved = self.root.join(path.trim_start_matches('/'));
        let resolved_str = resolved
            .to_str()
            .ok_or_else(|| anyhow!("path is not valid UTF-8: {}", resolved.display()))?;
        file_url(resolved_str)
    }
}

// The extra 6 bytes per method call don't really matter all that much and we
// reduce boiler plating for exception throwing in getModel(String).
// [spec:teaksta:def:sme.src.main.java.werti.util.resources.resources.no-access-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.util.resources.resources.no-access-fn]
fn no_access(reason: &str, exception: anyhow::Error) -> anyhow::Error {
    exception.context(format!("COULD_NOT_ACCESS_DATA: {reason}"))
}

/// Counterpart of the `MALFORMED_URL` throw sites in the two public
/// `getResource` entry points.
fn malformed_url(path: &str, murle: anyhow::Error) -> anyhow::Error {
    murle.context(format!("MALFORMED_URL: {path}"))
}

/// `java.net.URL#getPath()`: the path component of the URL, i.e. everything
/// from the first slash after the (optional) authority.
fn url_path(url: &str) -> &str {
    let after_scheme = match url.find("://") {
        Some(i) => &url[i + 3..],
        None => match url.find(':') {
            Some(i) => &url[i + 1..],
            None => url,
        },
    };
    match after_scheme.find('/') {
        Some(i) => &after_scheme[i..],
        None => "",
    }
}

/// `new URL(new URL("file://"), path)`: the context URL carries an empty
/// authority and an empty path, so a relative spec is resolved against the
/// root and gains a leading slash.
fn file_url(path: &str) -> Result<String> {
    if path.starts_with('/') {
        Ok(format!("file://{path}"))
    } else {
        Ok(format!("file:///{path}"))
    }
}

/// `java.net.URL#openStream()`, restricted to the `file:` protocol — the only
/// one the two entry points below can construct.
fn open_stream(url: &str) -> Result<Vec<u8>> {
    if !url.starts_with("file:") {
        return Err(anyhow!("unsupported protocol: {url}"));
    }
    Ok(std::fs::read(url_path(url))?)
}

/// Fetch a model from the servlet context.
///
/// `path` is the full path to the model (including file name) based on the
/// servlet context's root.
pub fn get_resource_from_servlet(path: &str, servlet: &ServletContext) -> Result<Vec<u8>> {
    // to find the correct location of the resource
    let m_path = servlet
        .get_resource(path)
        .map_err(|murle| malformed_url(path, murle))?;
    debug!("Retrieving from {}", url_path(&m_path));
    get_resource(&m_path)
}

/// Fetch a model from the JVM context.
///
/// `path` is the full path to the model (including file name) based on the
/// JVM's root.
pub fn get_resource_from_jvm(path: &str) -> Result<Vec<u8>> {
    // to find the correct location of the resource
    let m_path = file_url(path).map_err(|murle| malformed_url(path, murle))?;
    debug!("Retrieving from {}", url_path(&m_path));
    get_resource(&m_path)
}

/*
 * This is a super-safe method that shuoldn't leak any dangling references.
 * Thanks to dmlloyd at ##java.
 */
// [spec:teaksta:def:sme.src.main.java.werti.util.resources.resources.get-resource-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.util.resources.resources.get-resource-fn]
fn get_resource(path: &str) -> Result<Vec<u8>> {
    // to open a connection to the resource
    let is = open_stream(path).map_err(|ioe| no_access(url_path(path), ioe))?;

    // to connect to the object input stream of the resource: constructing an
    // ObjectInputStream reads and validates the serialization stream header
    // (0xACED plus a two-byte version) before anything else happens, and a bad
    // header surfaces as an IOException.
    if is.len() < 4 || is[0] != 0xac || is[1] != 0xed {
        return Err(no_access(
            url_path(path),
            anyhow!("invalid stream header in {path}"),
        ));
    }

    // to actually read it in and return it. Java object deserialisation has no
    // counterpart on this platform, so the serialised payload is handed back
    // whole and the caller decodes the model type it expects.
    Ok(is)
}
