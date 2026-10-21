//! Why a page did not load, as the three page endpoints name it.
//!
//! A page is read from wherever the learner pointed, and most of the ways that
//! can fail are not teaksta's: the site is down, the address is one nothing
//! can reach, the page is too large. The endpoints answer each with a code
//! rather than leaving the status to stand for all of them, because a 502
//! from a site that answered with an error and a 502 from one that never
//! answered are different news to the learner, and both are news about the
//! site.

use serde::Deserialize;

/// Why a page could not be read. Each variant is one code the page endpoints
/// write into the reply.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageFailure {
    BadAddress,
    BadScheme,
    PrivateAddress,
    NotServed,
    NoSuchText,
    /// The site answered, with this 4xx or 5xx status.
    SiteError(u16),
    Unreachable,
    TooLarge,
    Busy,
}

/// The body a page endpoint fails with. `status` is written only beside
/// `site-error`.
#[derive(Debug, Deserialize)]
struct FailureReply {
    error: String,
    #[serde(default)]
    status: Option<u16>,
}

impl PageFailure {
    /// The code the endpoint wrote into the reply.
    pub fn code(self) -> &'static str {
        match self {
            PageFailure::BadAddress => "bad-address",
            PageFailure::BadScheme => "bad-scheme",
            PageFailure::PrivateAddress => "private-address",
            PageFailure::NotServed => "not-served",
            PageFailure::NoSuchText => "no-such-text",
            PageFailure::SiteError(_) => "site-error",
            PageFailure::Unreachable => "page-unreachable",
            PageFailure::TooLarge => "page-too-large",
            PageFailure::Busy => "busy",
        }
    }

    /// The failure a reply names, or none for a body that names no code this
    /// client knows — a 500 of the backend's own, or a refusal of a request
    /// the app itself got wrong.
    pub fn parse(body: &str) -> Option<Self> {
        let reply: FailureReply = serde_json::from_str(body).ok()?;
        Some(match reply.error.as_str() {
            "bad-address" => PageFailure::BadAddress,
            "bad-scheme" => PageFailure::BadScheme,
            "private-address" => PageFailure::PrivateAddress,
            "not-served" => PageFailure::NotServed,
            "no-such-text" => PageFailure::NoSuchText,
            "site-error" => PageFailure::SiteError(reply.status?),
            "page-unreachable" => PageFailure::Unreachable,
            "page-too-large" => PageFailure::TooLarge,
            "busy" => PageFailure::Busy,
            _ => return None,
        })
    }

    /// What the learner reads, naming the site when there is one to name.
    ///
    /// English only: the webapp this replaces had no North Sámi wording for
    /// any of these to carry over, and an invented one would be worse than
    /// none.
    pub fn explain(self, site: Option<&str>) -> String {
        let named = site.unwrap_or("The site");
        match self {
            PageFailure::SiteError(status) => format!(
                "{named} answered with an error ({status}). The site itself is failing, \
                 not teaksta: try again later, or choose another page."
            ),
            PageFailure::Unreachable => match site {
                Some(site) => format!(
                    "{site} could not be reached, or did not answer in time. Check the \
                     address, or try again later."
                ),
                None => "The text could not be read. Try again later.".to_string(),
            },
            PageFailure::TooLarge => {
                "The page is too large to make an exercise from. Choose a shorter page.".to_string()
            }
            PageFailure::Busy => {
                "teaksta is reading too many pages at once. Try again in a moment.".to_string()
            }
            PageFailure::BadAddress => {
                "That is not a web address teaksta can read. Check it and try again.".to_string()
            }
            PageFailure::BadScheme => {
                "Only web pages, starting http:// or https://, can be used.".to_string()
            }
            PageFailure::PrivateAddress => {
                "That address is on a private network, which teaksta cannot reach.".to_string()
            }
            PageFailure::NotServed => "That file is not one teaksta can read.".to_string(),
            PageFailure::NoSuchText => {
                "The text this link pointed to is no longer stored.".to_string()
            }
        }
    }
}

/// The host a page address names, for telling the learner which site failed.
/// None for a stored text, which is no site's, and for anything that names no
/// host. An address without a scheme is read as the backend reads one, as
/// `http`.
pub fn site_of(url: &str) -> Option<&str> {
    let url = url.trim();
    if url.starts_with('/') {
        return None;
    }
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    let authority = rest.split(['/', '?', '#']).next()?;
    let host = authority
        .rsplit_once('@')
        .map_or(authority, |(_, host)| host);
    let host = match host.strip_prefix('[') {
        Some(bracketed) => bracketed.split(']').next()?,
        None => host.split(':').next()?,
    };
    (!host.is_empty()).then_some(host)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [PageFailure; 9] = [
        PageFailure::BadAddress,
        PageFailure::BadScheme,
        PageFailure::PrivateAddress,
        PageFailure::NotServed,
        PageFailure::NoSuchText,
        PageFailure::SiteError(502),
        PageFailure::Unreachable,
        PageFailure::TooLarge,
        PageFailure::Busy,
    ];

    /// Every code reads back as the failure it names.
    #[test]
    fn every_code_round_trips() {
        for failure in ALL {
            let body = match failure {
                PageFailure::SiteError(status) => {
                    format!(r#"{{"error":"site-error","status":{status}}}"#)
                }
                other => format!(r#"{{"error":"{}"}}"#, other.code()),
            };
            assert_eq!(PageFailure::parse(&body), Some(failure), "{body}");
        }
    }

    /// A body that names nothing this client knows is not mistaken for a
    /// failure it does.
    #[test]
    fn an_unknown_body_names_nothing() {
        assert_eq!(PageFailure::parse("no stored text is named x"), None);
        assert_eq!(PageFailure::parse(r#"{"error":"kitchens"}"#), None);
        assert_eq!(PageFailure::parse(r#"{"error":"site-error"}"#), None);
        assert_eq!(PageFailure::parse(""), None);
    }

    /// The case that was reported: a site down where it lives is named, with
    /// what it answered, and the learner is told it is not teaksta.
    #[test]
    fn a_failing_site_is_named() {
        let said = PageFailure::SiteError(502).explain(Some("kursa.oahpa.no"));

        assert!(said.starts_with("kursa.oahpa.no answered with an error (502)."));
        assert!(said.contains("not teaksta"));
    }

    #[test]
    fn every_failure_is_explained() {
        for failure in ALL {
            assert!(!failure.explain(Some("example.org")).is_empty());
            assert!(!failure.explain(None).is_empty());
        }
    }

    #[test]
    fn the_site_is_read_off_the_address() {
        for (url, site) in [
            (
                "https://kursa.oahpa.no/2013/12/30/ruovttus-leat-ollu-barggut/",
                Some("kursa.oahpa.no"),
            ),
            ("http://example.org:8080/a?b#c", Some("example.org")),
            ("example.org/a", Some("example.org")),
            ("https://user@example.org/", Some("example.org")),
            ("http://[2001:db8::1]:80/", Some("2001:db8::1")),
            ("/api/texts/00000000000000000000000000000000", None),
            ("file:///home/a.html", None),
            ("", None),
        ] {
            assert_eq!(site_of(url), site, "{url}");
        }
    }
}
