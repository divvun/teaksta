//! The upload endpoint: a teacher's own text in, the URL the stored copy is
//! read from out.
//!
//! It is the one endpoint that takes something other than JSON, and the one
//! that can turn a request away for a reason the teacher rather than the
//! programmer has to act on. So the multipart framing and the four gates the
//! backend can close on a text live here, apart from the three endpoints that
//! only ever ask for a page.

use serde::Deserialize;

use super::{ApiError, Backend};

/// The largest upload the backend accepts, in bytes. The backend holds this
/// cap itself; it is repeated here so a file already known to be over it is
/// turned away before it is sent rather than after.
pub const MAX_UPLOAD_BYTES: usize = 5 * 1024 * 1024;

/// The multipart part the upload endpoint reads the file from.
const UPLOAD_FIELD: &str = "file";

/// The multipart part the endpoint reads the keep-or-sweep choice from.
const KEEP_FIELD: &str = "keep";

/// What a multipart body's part delimiter is built from, before it is made
/// unique against the file's own bytes.
const BOUNDARY_SEED: &str = "teaksta-upload-boundary";

/// What the size limit in front of the upload handler answers with. It stands
/// ahead of the handler, so a body over the limit is refused without the JSON
/// the handler's own gates name themselves in.
const PAYLOAD_TOO_LARGE: u16 = 413;

/// One text a teacher offers, as the file picker handed it over.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UploadFile {
    /// The name the file carries on the teacher's own machine, which is the
    /// only hint the backend has about its format beyond the bytes.
    pub file_name: String,
    pub content: Vec<u8>,
    /// Whether the deployment should keep the text, so its link stays good, or
    /// sweep it away.
    pub keep: bool,
}

impl UploadFile {
    /// A text offered for this lesson only.
    pub fn new(file_name: impl Into<String>, content: Vec<u8>) -> Self {
        Self {
            file_name: file_name.into(),
            content,
            keep: false,
        }
    }

    /// The same text, kept or swept as the teacher asked.
    pub fn keeping(self, keep: bool) -> Self {
        Self { keep, ..self }
    }

    /// Whether there is a file here at all, which there is not while the
    /// picker stands empty.
    pub fn is_present(&self) -> bool {
        !self.file_name.trim().is_empty() && !self.content.is_empty()
    }
}

/// Why the backend turned a text away. Each variant is one code the upload
/// endpoint names a closed gate with, and carries the wording the teacher
/// reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rejection {
    NoFile,
    TooLarge,
    NotAPage,
    NotNorthSami,
}

impl Rejection {
    /// Every gate, in the order the endpoint runs them.
    pub const ALL: [Rejection; 4] = [
        Rejection::NoFile,
        Rejection::TooLarge,
        Rejection::NotAPage,
        Rejection::NotNorthSami,
    ];

    /// The code the endpoint writes into the reply.
    pub fn code(self) -> &'static str {
        match self {
            Rejection::NoFile => "no-file",
            Rejection::TooLarge => "too-large",
            Rejection::NotAPage => "not-a-page",
            Rejection::NotNorthSami => "not-north-sami",
        }
    }

    /// The gate a code names, or none for a code this client does not know.
    pub fn parse(code: &str) -> Option<Self> {
        Rejection::ALL
            .into_iter()
            .find(|rejection| rejection.code() == code)
    }

    /// What the teacher reads. The wording is the webapp's own, kept as it
    /// turned the same four gates away with it.
    pub fn message(self) -> &'static str {
        match self {
            Rejection::NoFile => "Vajálduhttet sáddet fiilla!",
            Rejection::TooLarge => "Fiila lea menddo stuoris! Lobálaš sturrodat: 5MB.",
            Rejection::NotAPage => "Fiilla formáhta ii leat html! Lobálaš formáhta: html.",
            Rejection::NotNorthSami => "Fiila ii sisttisdoala davvisámegiela!",
        }
    }

    /// The English gloss shown beside the message, as beside every other North
    /// Sámi line in the app.
    pub fn gloss(self) -> &'static str {
        match self {
            Rejection::NoFile => "No file was chosen",
            Rejection::TooLarge => "The file is over the 5 MB limit",
            Rejection::NotAPage => "The file is not an HTML page",
            Rejection::NotNorthSami => "Too little of the text reads as North Sámi",
        }
    }
}

/// The upload endpoint's reply, which names either where the text was stored
/// or the gate that closed. Both members are optional so one shape reads
/// either answer.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
struct UploadReply {
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

/// Decode an upload reply into the stored text's URL, or the gate that closed.
/// The status is read alongside the body because the size limit stands ahead
/// of the handler and answers with no body of the handler's shape.
pub fn parse_upload(status: u16, body: &str) -> Result<String, ApiError> {
    match serde_json::from_str::<UploadReply>(body) {
        Ok(UploadReply {
            error: Some(code), ..
        }) => Err(match Rejection::parse(&code) {
            Some(rejection) => ApiError::Rejected(rejection),
            None => ApiError::Malformed(format!("the reply names an unknown gate {code:?}")),
        }),
        Ok(UploadReply { url: Some(url), .. }) if (200..300).contains(&status) => Ok(url),
        _ if status == PAYLOAD_TOO_LARGE => Err(ApiError::Rejected(Rejection::TooLarge)),
        _ if !(200..300).contains(&status) => Err(ApiError::Status(status)),
        _ => Err(ApiError::Malformed(
            "the reply names neither a stored text nor a gate".to_string(),
        )),
    }
}

/// Frame one text as the multipart body the upload endpoint reads. Written out
/// here rather than left to the browser's own form encoding because the picker
/// has already handed the bytes over, and because a body built in one place is
/// a body that can be read back in a test.
pub fn multipart_body(file: &UploadFile, boundary: &str) -> Vec<u8> {
    let keep = file.keep;
    let name = quotable(&file.file_name);
    let mut body = Vec::with_capacity(file.content.len() + 2 * BOUNDARY_SEED.len() + 256);

    // No part carries a media type: the backend reads the bytes themselves,
    // and a type named here would only be this client's guess at them.
    body.extend_from_slice(
        format!(
            "--{boundary}\r\n\
             Content-Disposition: form-data; name=\"{UPLOAD_FIELD}\"; filename=\"{name}\"\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(&file.content);
    body.extend_from_slice(
        format!(
            "\r\n--{boundary}\r\n\
             Content-Disposition: form-data; name=\"{KEEP_FIELD}\"\r\n\r\n\
             {keep}\r\n\
             --{boundary}--\r\n"
        )
        .as_bytes(),
    );

    body
}

/// A delimiter the text itself cannot hold, since a boundary occurring in the
/// bytes would end the part early. Grown from the seed rather than drawn at
/// random, which leaves the body a function of what is being sent.
pub fn boundary_for(content: &[u8]) -> String {
    let mut boundary = BOUNDARY_SEED.to_string();

    while content
        .windows(boundary.len())
        .any(|window| window == boundary.as_bytes())
    {
        boundary.push('-');
    }

    boundary
}

/// A file name that cannot break the header quoting it sits in. A quote is
/// percent-escaped and a line break dropped, which is all a name can carry
/// that the framing would read as its own.
fn quotable(file_name: &str) -> String {
    file_name
        .trim()
        .replace('"', "%22")
        .replace(['\r', '\n'], "")
}

/// Offer one text, and read back the URL the enhancement endpoints reach the
/// stored copy at. The two gates that can be seen from here — an empty picker,
/// and a file already over the cap — are closed here rather than across the
/// wire, and read to the teacher as the backend's own would.
pub async fn upload(backend: &Backend, file: &UploadFile) -> Result<String, ApiError> {
    if !file.is_present() {
        return Err(ApiError::Rejected(Rejection::NoFile));
    }
    if file.content.len() >= MAX_UPLOAD_BYTES {
        return Err(ApiError::Rejected(Rejection::TooLarge));
    }

    let boundary = boundary_for(&file.content);
    let (status, body) = post_multipart(
        &backend.upload_url(),
        &boundary,
        multipart_body(file, &boundary),
    )
    .await?;

    parse_upload(status, &body)
}

/// The one request whose body is read whatever the status says, because a
/// closed gate names itself in the body of the refusal.
#[cfg(target_arch = "wasm32")]
async fn post_multipart(
    url: &str,
    boundary: &str,
    body: Vec<u8>,
) -> Result<(u16, String), ApiError> {
    let response = gloo_net::http::Request::post(url)
        .header(
            "Content-Type",
            &format!("multipart/form-data; boundary={boundary}"),
        )
        .body(js_sys::Uint8Array::from(body.as_slice()))
        .map_err(|error| ApiError::Network(error.to_string()))?
        .send()
        .await
        .map_err(|error| ApiError::Network(error.to_string()))?;
    let status = response.status();

    response
        .text()
        .await
        .map(|body| (status, body))
        .map_err(|error| ApiError::Network(error.to_string()))
}

#[cfg(not(target_arch = "wasm32"))]
async fn post_multipart(
    _url: &str,
    _boundary: &str,
    _body: Vec<u8>,
) -> Result<(u16, String), ApiError> {
    Err(ApiError::Unsupported)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offered_file() -> UploadFile {
        UploadFile::new("artihkal.html", b"<html>Mun oidnen viesu.</html>".to_vec())
    }

    #[test]
    fn an_upload_goes_to_the_upload_endpoint() {
        assert_eq!(Backend::default().upload_url(), "/api/upload");
        assert_eq!(
            Backend::at("https://gtweb.uit.no/teaksta/").upload_url(),
            "https://gtweb.uit.no/teaksta/api/upload"
        );
    }

    #[test]
    fn an_accepted_upload_answers_the_stored_url() {
        let url = parse_upload(200, r#"{"url":"file:///srv/teaksta/upload/aB3xY9zQ1w"}"#).unwrap();

        assert_eq!(url, "file:///srv/teaksta/upload/aB3xY9zQ1w");
    }

    #[test]
    fn every_gate_reads_back_as_itself() {
        for rejection in Rejection::ALL {
            let body = format!(r#"{{"error":"{}"}}"#, rejection.code());

            assert_eq!(
                parse_upload(400, &body).unwrap_err(),
                ApiError::Rejected(rejection),
                "{}",
                rejection.code()
            );
            assert_eq!(Rejection::parse(rejection.code()), Some(rejection));
        }

        assert_eq!(Rejection::parse("no-captcha"), None);
    }

    #[test]
    fn the_size_limit_answers_without_a_gate() {
        assert_eq!(
            parse_upload(413, r#"{"error":"too-large"}"#).unwrap_err(),
            ApiError::Rejected(Rejection::TooLarge)
        );
        assert_eq!(
            parse_upload(413, "payload too large").unwrap_err(),
            ApiError::Rejected(Rejection::TooLarge)
        );
    }

    #[test]
    fn an_unknown_gate_is_malformed_not_silent() {
        assert!(matches!(
            parse_upload(400, r#"{"error":"no-captcha"}"#).unwrap_err(),
            ApiError::Malformed(_)
        ));
        assert_eq!(
            parse_upload(500, "<html>").unwrap_err(),
            ApiError::Status(500)
        );
        assert!(matches!(
            parse_upload(200, "{}").unwrap_err(),
            ApiError::Malformed(_)
        ));
    }

    #[test]
    fn every_gate_is_named_in_sami() {
        for rejection in Rejection::ALL {
            assert!(!rejection.message().is_empty());
            assert!(!rejection.gloss().is_empty());
        }

        assert_eq!(
            Rejection::NotNorthSami.message(),
            "Fiila ii sisttisdoala davvisámegiela!"
        );
        assert_eq!(
            ApiError::Rejected(Rejection::NoFile).to_string(),
            "the text was turned away: No file was chosen"
        );
    }

    #[test]
    fn the_body_frames_file_and_keep_choice() {
        let file = offered_file().keeping(true);
        let boundary = boundary_for(&file.content);
        let body = String::from_utf8(multipart_body(&file, &boundary)).unwrap();

        assert_eq!(boundary, "teaksta-upload-boundary");
        assert_eq!(
            body,
            "--teaksta-upload-boundary\r\n\
             Content-Disposition: form-data; name=\"file\"; filename=\"artihkal.html\"\r\n\r\n\
             <html>Mun oidnen viesu.</html>\r\n\
             --teaksta-upload-boundary\r\n\
             Content-Disposition: form-data; name=\"keep\"\r\n\r\n\
             true\r\n\
             --teaksta-upload-boundary--\r\n"
        );
        assert!(
            String::from_utf8(multipart_body(&offered_file(), &boundary))
                .unwrap()
                .contains("\r\n\r\nfalse\r\n")
        );
    }

    #[test]
    fn a_held_boundary_is_grown_until_free() {
        let held = format!("<p>{BOUNDARY_SEED}</p>").into_bytes();

        let boundary = boundary_for(&held);

        assert_eq!(boundary, format!("{BOUNDARY_SEED}-"));
        assert!(
            !held
                .windows(boundary.len())
                .any(|window| window == boundary.as_bytes())
        );

        let body =
            String::from_utf8(multipart_body(&UploadFile::new("a.html", held), &boundary)).unwrap();

        // The seed the text holds is left where it stands, and only the three
        // delimiters the framing writes read as delimiters.
        assert!(body.contains(&format!("<p>{BOUNDARY_SEED}</p>")));
        assert_eq!(body.matches(&format!("--{boundary}")).count(), 3);
    }

    #[test]
    fn a_file_name_cannot_break_its_header() {
        let file = UploadFile::new("  \"odd\"\r\nname.html  ", b"<html>a</html>".to_vec());

        let body = String::from_utf8(multipart_body(&file, "b")).unwrap();

        assert!(body.contains("filename=\"%22odd%22name.html\""));
        // The name's own line break added no line: two parts and a close is
        // nine lines however the file was named.
        assert_eq!(body.lines().count(), 9);
    }

    #[test]
    fn an_empty_picker_never_reaches_the_wire() {
        let empty = UploadFile::default();
        let named = UploadFile::new("artihkal.html", Vec::new());

        assert!(!empty.is_present());
        assert!(!named.is_present());
        assert!(!UploadFile::new("   ", b"<html>a</html>".to_vec()).is_present());
        assert!(offered_file().is_present());
    }
}
