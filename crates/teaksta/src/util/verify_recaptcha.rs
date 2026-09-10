//! Server-side validation of a Google reCAPTCHA response token.
//!
//! The `HttpsURLConnection` the original drives maps onto a blocking reqwest
//! client. The body is written as raw bytes rather than through an encoder so
//! that the byte-truncating write of the original is preserved exactly.

use std::time::Duration;

use anyhow::{Result, anyhow};

// [spec:teaksta:def:sme.src.main.java.werti.util.verify-recaptcha.verify-recaptcha]
pub const URL: &str = "https://www.google.com/recaptcha/api/siteverify";
const USER_AGENT: &str = "Mozilla/5.0";

/// `DataOutputStream#writeBytes`: each character contributes only its low
/// byte, so anything outside Latin-1 is mangled on the wire.
fn write_bytes(text: &str) -> Vec<u8> {
    text.chars().map(|c| (c as u32 & 0xFF) as u8).collect()
}

// [spec:teaksta:def:sme.src.main.java.werti.util.verify-recaptcha.verify-recaptcha.verify-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.util.verify-recaptcha.verify-recaptcha.verify-fn]
pub fn verify(g_recaptcha_response: &str, secret: &str) -> Result<bool> {
    // The original also rejected a null token; a `&str` cannot be null, so
    // only the empty-string half of that test is representable.
    if g_recaptcha_response.is_empty() {
        return Ok(false);
    }

    let attempt = (|| -> Result<bool> {
        // No connect or read timeout is configured, so a hung endpoint blocks
        // the calling request thread; the client's own default is disabled to
        // keep that.
        let con = reqwest::blocking::Client::builder()
            .timeout(None::<Duration>)
            .build()?;

        let post_params = format!("secret={}&response={}", secret, g_recaptcha_response);

        // Send post request
        let con = con
            .post(URL)
            // add reuqest header
            .header("User-Agent", USER_AGENT)
            .header("Accept-Language", "en-US,en;q=0.5")
            // `HttpURLConnection` supplies this header itself once output is
            // enabled and none was set; reqwest does not, and the endpoint
            // rejects a body without it.
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(write_bytes(&post_params))
            .send()?;

        let response_code = con.status().as_u16();
        println!("\nSending 'POST' request to URL : {}", URL);
        println!("Post parameters : {}", post_params);
        println!("Response Code : {}", response_code);

        // `getInputStream` throws on an error status rather than handing back
        // the error body.
        if response_code >= 400 {
            return Err(anyhow!(
                "IOException: Server returned HTTP response code: {} for URL: {}",
                response_code,
                URL
            ));
        }

        // the reader appends each line with no separator between them
        let response: String = con.text()?.lines().collect();

        // print result
        println!("{}", response);

        // parse JSON response and return 'success' value
        let json_object: serde_json::Value = serde_json::from_str(&response)?;

        json_object
            .get("success")
            .and_then(serde_json::Value::as_bool)
            .ok_or_else(|| anyhow!("no boolean member \"success\" in {}", response))
    })();

    match attempt {
        Ok(success) => Ok(success),
        Err(e) => {
            eprintln!("{:?}", e);
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // [spec:teaksta:sem:sme.src.main.java.werti.util.verify-recaptcha.verify-recaptcha.verify-fn/test]
    #[test]
    fn empty_token_is_rejected_before_the_endpoint() {
        assert!(!verify("", "a-shared-secret").unwrap());
    }

    // [spec:teaksta:sem:sme.src.main.java.werti.util.verify-recaptcha.verify-recaptcha.verify-fn/test]
    #[test]
    fn secret_is_not_checked_when_token_is_empty() {
        assert!(!verify("", "").unwrap());
    }

    #[test]
    fn request_body_keeps_only_each_low_byte() {
        assert_eq!(
            write_bytes("secret=abc&response=xyz"),
            b"secret=abc&response=xyz".to_vec()
        );
        assert_eq!(write_bytes("á"), vec![0xE1]);
        assert_eq!(write_bytes("š"), b"a".to_vec());
        assert_eq!(write_bytes("ŋ"), b"K".to_vec());
    }

    #[test]
    fn the_endpoint_is_the_google_siteverify_url() {
        assert_eq!(URL, "https://www.google.com/recaptcha/api/siteverify");
    }
}
