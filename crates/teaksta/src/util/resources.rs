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

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Write;

    use tempfile::{NamedTempFile, TempDir};

    /// The four-byte serialization stream header: magic 0xACED plus version 5.
    const STREAM_HEADER: [u8; 4] = [0xac, 0xed, 0x00, 0x05];

    fn write_temp(bytes: &[u8]) -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("temp file");
        file.write_all(bytes).expect("write");
        file.flush().expect("flush");
        file
    }

    fn url_for(file: &NamedTempFile) -> String {
        file_url(file.path().to_str().expect("utf-8 path")).expect("file url")
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.resources.resources.no-access-fn/test]
    #[test]
    fn no_access_builds_error_without_raising_it() {
        let built = no_access("/opt/smi/model.bin", anyhow!("disk on fire"));

        // The message key is the standard one and `reason` is its single
        // substitution argument.
        assert_eq!(
            built.to_string(),
            "COULD_NOT_ACCESS_DATA: /opt/smi/model.bin"
        );

        // The supplied exception is kept as the chained cause.
        assert_eq!(built.root_cause().to_string(), "disk on fire");
        assert!(built.chain().any(|c| c.to_string() == "disk on fire"));

        // It is returned rather than raised, so the reason can be varied
        // freely and nothing happens until a caller propagates it.
        let empty_reason = no_access("", anyhow!("boom"));
        assert_eq!(empty_reason.to_string(), "COULD_NOT_ACCESS_DATA: ");
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.resources.resources.get-resource-fn/test]
    #[test]
    fn get_resource_reads_payload_or_fails_no_access() {
        let mut payload = STREAM_HEADER.to_vec();
        payload.extend_from_slice(b"\x73\x72\x00\x05model");
        let file = write_temp(&payload);
        let url = url_for(&file);

        // The payload is handed back whole for the caller to decode.
        assert_eq!(get_resource(&url).expect("read"), payload);

        // A bad stream header surfaces as a failure named by the path
        // component of the URL, not the whole URL.
        let bad = write_temp(b"\x00\x01\x02\x03not-serialised");
        let bad_url = url_for(&bad);
        let err = get_resource(&bad_url).unwrap_err();
        assert_eq!(
            err.to_string(),
            format!("COULD_NOT_ACCESS_DATA: {}", url_path(&bad_url))
        );

        // A stream too short to carry a header fails the same way.
        let short = write_temp(&[0xac, 0xed]);
        assert!(
            get_resource(&url_for(&short))
                .unwrap_err()
                .to_string()
                .starts_with("COULD_NOT_ACCESS_DATA: ")
        );

        // A stream that cannot be opened at all fails the same way, with the
        // underlying I/O error kept as the cause.
        let dir = TempDir::new().expect("temp dir");
        let missing = file_url(dir.path().join("absent.bin").to_str().expect("utf-8 path"))
            .expect("file url");
        let open_err = get_resource(&missing).unwrap_err();
        assert_eq!(
            open_err.to_string(),
            format!("COULD_NOT_ACCESS_DATA: {}", url_path(&missing))
        );
        assert!(
            open_err
                .root_cause()
                .to_string()
                .to_lowercase()
                .contains("no such file")
        );

        // A URL whose protocol the entry points cannot construct is rejected
        // before any read is attempted.
        assert!(
            get_resource("http://example.invalid/model.bin")
                .unwrap_err()
                .to_string()
                .starts_with("COULD_NOT_ACCESS_DATA: ")
        );
    }

    #[test]
    fn public_entry_points_resolve_paths_before_delegating() {
        let mut payload = STREAM_HEADER.to_vec();
        payload.extend_from_slice(b"model");

        let dir = TempDir::new().expect("temp dir");
        let model = dir.path().join("model.bin");
        std::fs::write(&model, &payload).expect("write");

        let from_jvm =
            get_resource_from_jvm(model.to_str().expect("utf-8 path")).expect("jvm read");
        assert_eq!(from_jvm, payload);

        let servlet = ServletContext {
            root: dir.path().to_path_buf(),
        };
        let from_servlet = get_resource_from_servlet("/model.bin", &servlet).expect("servlet read");
        assert_eq!(from_servlet, payload);
    }
}
