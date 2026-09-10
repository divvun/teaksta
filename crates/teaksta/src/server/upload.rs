//! Teacher text upload: an uploaded page in, a `file:` URL the enhancement
//! endpoints can be pointed at out.
//!
//! Three gates stand between the two. Size, so one request cannot fill the
//! disk; media type, because the enhancement pass wants a page rather than a
//! spreadsheet; and language, because a text the analyser does not recognise
//! yields an exercise with no exercises in it.
//!
//! Nothing here authenticates the uploader. The endpoint is reachable only
//! where the operator's own deployment puts it.

use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use rand::Rng;
use rand::distr::Alphanumeric;
use reqwest::Url;
use tracing::info;

use crate::morpho::MorphoPipeline;
use crate::util::html_utils;

/// The largest upload accepted, in bytes.
pub const MAX_UPLOAD_BYTES: usize = 5 * 1024 * 1024;

/// The share of a text's alphabetic tokens that must carry a North Sámi
/// reading for the text to count as North Sámi. Set well below one because
/// names, loanwords and typos are normal in classroom material, and far
/// enough above chance that a page in another language cannot clear it: the
/// only forms the analyser recognises in an English or Norwegian page are
/// shared proper nouns, which leaves such a page in single-digit percentages.
pub const SME_READING_SHARE: f64 = 0.6;

/// The two media types a page may arrive as.
const PAGE_TYPES: [&str; 2] = ["text/html", "application/xhtml+xml"];

/// The length of the stored file's name.
const STORED_NAME_LEN: usize = 10;

/// Owner-read-only, so a stored text can be neither executed nor rewritten.
const STORED_MODE: u32 = 0o400;

/// Why an upload was turned away. Each variant reaches the client as its own
/// code, so a frontend can say which gate closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Rejection {
    #[error("the request carried no file")]
    NoFile,
    #[error("the file is larger than the {MAX_UPLOAD_BYTES} byte limit")]
    TooLarge,
    #[error("the file is not an HTML or XHTML page")]
    NotAPage,
    #[error("too little of the text reads as North Sámi")]
    NotNorthSami,
}

impl Rejection {
    pub fn code(self) -> &'static str {
        match self {
            Rejection::NoFile => "no-file",
            Rejection::TooLarge => "too-large",
            Rejection::NotAPage => "not-a-page",
            Rejection::NotNorthSami => "not-north-sami",
        }
    }
}

/// One uploaded text, as the request carried it.
// [spec:teaksta:def:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet+1]
#[derive(Debug, Clone, Default)]
pub struct Upload {
    pub file_name: Option<String>,
    pub content: Vec<u8>,
    /// Whether the teacher asked for the text to be kept.
    pub keep: bool,
}

/// Runs the three gates over an upload and stores what passes them, handing
/// back the path it was stored at. A closed gate is a [`Rejection`] carried
/// by the error; anything else is a deployment failure.
// [spec:teaksta:def:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn+2]
// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn+2]
pub fn store(upload: &Upload, directory: &Path) -> Result<PathBuf> {
    let Some(file_name) = upload.file_name.as_deref() else {
        return Err(Rejection::NoFile.into());
    };
    if upload.content.is_empty() {
        return Err(Rejection::NoFile.into());
    }
    if upload.content.len() >= MAX_UPLOAD_BYTES {
        return Err(Rejection::TooLarge.into());
    }
    if !is_page(&upload.content, file_name) {
        return Err(Rejection::NotAPage.into());
    }

    let page = String::from_utf8_lossy(&upload.content);
    let share = sme_share(&page)?;
    info!("Upload reads {:.0}% North Sámi", share * 100.0);
    if share < SME_READING_SHARE {
        return Err(Rejection::NotNorthSami.into());
    }

    let stored = directory.join(random_name());
    std::fs::write(&stored, &upload.content)?;
    set_read_only(&stored);
    Ok(stored)
}

/// Whether the bytes are a page, by the two types the enhancement pass can
/// read. The uploaded name is a hint for a page opening with neither a
/// doctype nor a root element; the bytes decide otherwise.
// [spec:teaksta:def:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.check-meta-data-fn+1]
// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.check-meta-data-fn+1]
pub fn is_page(content: &[u8], file_name: &str) -> bool {
    match detect_content_type(content, file_name) {
        Some(detected) => PAGE_TYPES.iter().any(|kind| detected.contains(kind)),
        None => false,
    }
}

fn detect_content_type(content: &[u8], resource_name: &str) -> Option<String> {
    if content.is_empty() {
        return None;
    }

    let head_len = content.len().min(8192);
    let head = String::from_utf8_lossy(&content[..head_len]).to_lowercase();
    let name = resource_name.to_lowercase();

    if head.contains("<?xml") && head.contains("xhtml") {
        return Some("application/xhtml+xml".to_string());
    }
    if head.contains("<!doctype html")
        || head.contains("<html")
        || head.contains("<head")
        || head.contains("<body")
    {
        return Some("text/html; charset=UTF-8".to_string());
    }
    if name.ends_with(".xhtml") {
        return Some("application/xhtml+xml".to_string());
    }
    if name.ends_with(".html") || name.ends_with(".htm") {
        return Some("text/html".to_string());
    }
    if head.starts_with("<?xml") {
        return Some("application/xml".to_string());
    }
    if std::str::from_utf8(content).is_ok() {
        return Some("text/plain; charset=UTF-8".to_string());
    }

    Some("application/octet-stream".to_string())
}

/// The share of a page's alphabetic tokens the analyser gives a North Sámi
/// reading. A page with no alphabetic token scores zero rather than dividing
/// by none.
// [spec:teaksta:def:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.language-gate-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.language-gate-fn]
pub fn sme_share(page: &str) -> Result<f64> {
    let (document, _) = html_utils::extract(page);
    if document.text.trim().is_empty() {
        return Ok(0.0);
    }

    let pipeline = MorphoPipeline::shared();
    let tokens = pipeline.tokenize(&document.text)?;
    if tokens.is_empty() {
        return Ok(0.0);
    }
    let stream = pipeline.analyze_disambiguate(&tokens)?;

    let (alphabetic, recognised) = count_recognised(&stream);
    if alphabetic == 0 {
        return Ok(0.0);
    }
    Ok(recognised as f64 / alphabetic as f64)
}

/// Walks a CG stream and counts its alphabetic cohorts, and how many of them
/// carry a reading the analyser produced rather than guessed. An unrecognised
/// form comes back with a bare `?` tag, so a cohort counts as recognised when
/// at least one of its readings carries no such tag.
pub fn count_recognised(stream: &str) -> (usize, usize) {
    let mut alphabetic = 0usize;
    let mut recognised = 0usize;
    let mut counting = false;
    let mut known = false;

    for line in stream.lines() {
        if let Some(form) = cohort_form(line) {
            if counting && known {
                recognised += 1;
            }
            counting = form.chars().any(char::is_alphabetic);
            known = false;
            if counting {
                alphabetic += 1;
            }
        } else if counting && line.starts_with([' ', '\t']) && is_known_reading(line) {
            known = true;
        }
    }
    if counting && known {
        recognised += 1;
    }

    (alphabetic, recognised)
}

/// The surface form of a cohort line, which opens `"<` and closes `>"`.
fn cohort_form(line: &str) -> Option<&str> {
    line.strip_prefix("\"<")?.strip_suffix(">\"")
}

fn is_known_reading(line: &str) -> bool {
    !line.split_whitespace().any(|tag| tag == "?")
}

fn random_name() -> String {
    rand::rng()
        .sample_iter(Alphanumeric)
        .take(STORED_NAME_LEN)
        .map(char::from)
        .collect()
}

fn set_read_only(file: &Path) {
    use std::os::unix::fs::PermissionsExt;

    if let Ok(metadata) = std::fs::metadata(file) {
        let mut permissions = metadata.permissions();
        permissions.set_mode(STORED_MODE);
        let _ = std::fs::set_permissions(file, permissions);
    }
}

/// The `file:` URL an accepted upload is reachable at.
///
/// Built from the path rather than written around it, so a deployment whose
/// upload directory carries a space or a non-ASCII character hands back an
/// address that parses back to the path it names. The enhancement endpoints
/// read it with `Url::to_file_path`, which is the same encoding read the
/// other way.
pub fn file_url(stored: &Path) -> Result<String> {
    let absolute = std::path::absolute(stored)?;
    let url = Url::from_file_path(&absolute)
        .map_err(|()| anyhow!("{} is not an absolute path", absolute.display()))?;
    Ok(url.to_string())
}

#[cfg(test)]
#[path = "upload_tests.rs"]
mod tests;
