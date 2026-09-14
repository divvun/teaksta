//! Turning the address a request carried into a page, and refusing the
//! addresses a service reachable by strangers must not fetch on their behalf.
//!
//! The enhancement endpoints take a `url` from the caller, so this is the one
//! place where a caller's string becomes a file read or a connection. Every
//! decision about what may be reached lives here, and every one of them is
//! made before anything is opened: [`target`] either hands back a vetted
//! [`Target`] or refuses, and [`fetch`] can only be called with a `Target`.
//!
//! It is also where a page off the network is told apart from a page off the
//! disk, so it is where the reader-mode reduction in
//! [`crate::server::reader`] is applied — to the first and never the second.
//!
//! # Schemes
//!
//! `http`, `https` and `file`, and nothing else. A `data:`, `ftp:` or
//! `jar:` address is refused rather than handed to a library that might know
//! what to do with it.
//!
//! Beside the three schemes there is one address that is not an address at
//! all: a reference to a text this deployment stored, which is written
//! `/api/texts/<id>` and carries no scheme and no host.
//!
//! # Stored texts, and why they carry no host
//!
//! A kept upload is answered with `/api/texts/<id>`, and that is what the
//! enhancement endpoints are then handed back as their `url`. Fetching it
//! over HTTP would mean this deployment connecting to itself — which the
//! private-address policy refuses, correctly, and which would be a waste of a
//! socket even if it did not.
//!
//! So a stored-text reference is recognised here and read from
//! [`crate::server::texts::TextStore`] directly. What makes that safe is that
//! the reference has no host to be wrong about. A request cannot tell this
//! process its own name — `Host` is a header a caller writes — so an address
//! that had to be compared against this deployment's hostname would be an
//! address whose meaning a caller controls. A root-relative reference has
//! nothing to compare: `/api/texts/<id>` names this deployment because it
//! names no other, and `http://elsewhere.example/api/texts/<id>` is an
//! ordinary web address that is fetched, vetted and refused exactly as
//! `http://elsewhere.example/anything` is. The path shape opens no hole
//! because it is only ever read off a reference that has no authority
//! component for a caller to have chosen.
//!
//! The `id` is read by [`crate::server::texts::TextId::parse`], which admits
//! thirty-two hex characters and nothing else, so no traversal, no separator
//! and no encoded anything reaches an object key.
//!
//! # `file:` confinement
//!
//! The `file:` scheme exists because an accepted upload is handed back as a
//! `file:` URL and then fetched by the enhancement endpoints, so it cannot
//! simply be dropped. It is confined instead: the path is resolved and must
//! land inside one of the directories this deployment itself mints `file:`
//! URLs under, which [`served_roots`] derives from the deployment
//! configuration — the two upload directories, and nothing else. Anything
//! else — `/etc/passwd`, a key under `~/.ssh`, the analysis cache — is
//! refused with the same 400 the endpoint answers any other unusable address
//! with.
//!
//! Resolution walks up to the deepest ancestor that exists and canonicalises
//! that, then appends what is left verbatim. Every symlink on the existing
//! part is therefore resolved before the confinement test, so a symlink
//! planted inside an allowed directory that points outside it does not
//! escape; and a file that does not exist yet is still placed exactly where it
//! would be created, so a missing upload is reported as unreadable rather than
//! as an intrusion.
//!
//! # The private-network policy
//!
//! The service fetches public pages by design, so there is no host allowlist
//! to write. The policy is the other way round: every address a connection
//! would land on must be a public one. [`is_public`] rejects loopback,
//! `0.0.0.0/8`, RFC 1918 space, the `169.254.0.0/16` link-local range that
//! carries cloud instance-metadata services, carrier-grade NAT, benchmarking,
//! documentation and reserved space, and — on IPv6 — the unspecified and
//! loopback addresses, unique-local `fc00::/7`, link-local `fe80::/10`,
//! multicast, and documentation space. IPv6 forms that carry an IPv4 address
//! inside them (IPv4-mapped, the deprecated IPv4-compatible form, 6to4 and the
//! NAT64 well-known prefix) are judged by the IPv4 address they would deliver
//! to, so `::ffff:127.0.0.1` and `2002:7f00:1::` are refused as loopback.
//!
//! The test is applied at two points. An address that names an IP literally is
//! tested by [`target`], before any socket is opened, and refused with a 400 —
//! including the legacy integer and octal spellings, which the URL parser
//! normalises to dotted quads first. An address that names a host is tested
//! after resolution, by the client's own DNS resolver, which drops every
//! private address a name resolves to and fails the lookup when none is left.
//! Redirects are tested again on each hop.
//!
//! ## What this prevents
//!
//! Because the resolver hands the connector the vetted addresses themselves,
//! the connection lands on an address that was checked — there is no second
//! lookup between the check and the connect for a rebinding answer to win.
//! A name that resolves to both a public and a private address reaches the
//! public one only. A name resolving only to private addresses fails to
//! resolve at all, and is reported as unreachable (502) rather than refused
//! (400), because nothing about the address was knowable until it resolved.
//! A public page that redirects to `http://169.254.169.254/` is not followed.
//!
//! ## What this does not prevent
//!
//! A public host that is itself a proxy into a private network: the far end
//! is fetched as asked and whatever it returns is enhanced. Nothing here
//! inspects the response.
//!
//! An HTTP proxy configured in the process environment. reqwest honours
//! `HTTP_PROXY` and friends, and a proxied request is resolved and connected
//! by the proxy, so the address policy cannot see where it lands. A
//! deployment that configures a proxy has delegated this decision to it.
//!
//! Filesystem races. The `file:` path is resolved and then opened, so a
//! directory on the resolved path replaced by a symlink in between is
//! followed. Closing that needs `openat2`-style resolution the standard
//! library does not offer, and it requires write access to a directory this
//! deployment serves out of.
//!
//! A far end that answers as much as it likes within the time it has. The
//! body is read up to the cap the deployment configured and abandoned at it,
//! so what one fetch may hold is bounded; but [`MAX_CONCURRENT_FETCHES`] of
//! them may be reading at once, so what all of them may hold together is that
//! many caps, and the analysis each one then feeds costs several multiples of
//! its page again. The cap is what keeps one page from being a memory
//! problem, not what sizes the machine.
//!
//! A far end that answers slowly. The whole fetch is bounded by
//! [`FETCH_TIMEOUT`], which bounds the body's arrival as well as the
//! connection, so a page dribbled out a byte at a time ends at the deadline
//! rather than at the cap.
//!
//! # One client, bounded concurrency
//!
//! [`CLIENT`] is built once. A per-fetch client would carry its own
//! connection pool, TLS configuration and background runtime thread, so a few
//! hundred concurrent fetches of slow pages would cost a few hundred threads.
//!
//! The blocking client blocks the thread it is called on, so a fetch runs on
//! a blocking thread; [`MAX_CONCURRENT_FETCHES`] of those may be in flight at
//! once. A request that finds no slot waits for one as a task rather than as a
//! thread — waiting is free, so the blocking pool stays available to the
//! upload endpoint and to analysis — and gives up after [`FETCH_TIMEOUT`],
//! answering 503 rather than queueing without end.

use std::ffi::OsString;
use std::io::Read as _;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs as _};
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};
use std::time::Duration;

use anyhow::{Result, anyhow};
use encoding_rs::{Encoding, UTF_8};
use reqwest::Url;
use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use reqwest::header::{CONTENT_TYPE, HeaderMap};
use tokio::sync::{Semaphore, SemaphorePermit};

use crate::context::Config;
use crate::server::reader;
use crate::server::texts::{Missing, TEXTS_PATH, TextId, TextStore};

/// How long a page fetch may take, start to finished body.
pub const FETCH_TIMEOUT: Duration = Duration::from_secs(20);

/// How long the connection itself may take, inside that budget.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// How many redirects are followed before the chain is given up on.
const MAX_REDIRECTS: usize = 5;

/// How many page fetches may be in flight at once.
pub const MAX_CONCURRENT_FETCHES: usize = 16;

/// What the far end is told this is, so an operator reading their logs can
/// see who asked.
const USER_AGENT: &str = concat!("teaksta/", env!("CARGO_PKG_VERSION"));

/// An address the service will not fetch. Each variant reaches the client as
/// a 400 carrying its own message, because each is the caller's mistake.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Refusal {
    #[error("url is not a valid address")]
    Address,
    #[error("only http, https, file and stored-text addresses can be enhanced")]
    Scheme,
    #[error("addresses on the private network cannot be enhanced")]
    Private,
    #[error("that file is not one this deployment serves")]
    Confined,
}

/// The page could not be read from where the request pointed. The far end's
/// failure, not the caller's, so it reaches the client as a 502.
#[derive(Debug, thiserror::Error)]
#[error("the page at {0} could not be fetched")]
pub struct Unreachable(pub String);

/// The page is larger than this deployment reads, and the read was abandoned
/// at the cap rather than finished.
///
/// It reaches the client as a 502, beside [`Unreachable`], and deliberately
/// not as the 400 a [`Refusal`] answers with. A refusal is decided from the
/// address alone, before anything is opened, and is therefore something the
/// caller could have known; how many bytes are behind an address is not, any
/// more than whether a name resolves — which this module already reports as
/// unreachable for that same reason. Nor is it the 413 the endpoints answer
/// an oversized request body with: that body is the caller's, and this one is
/// a stranger's page the caller merely named.
// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.page-cap-fn+1]
#[derive(Debug, Clone, thiserror::Error)]
#[error("the page at {address} is larger than the {cap} byte limit")]
pub struct Oversized {
    pub address: String,
    pub cap: usize,
}

/// Every fetch slot is taken and one did not come free. Reaches the client as
/// a 503.
#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("too many pages are being fetched at once")]
pub struct Overloaded;

/// A vetted address, and the only thing [`fetch`] will read from. Held apart
/// from [`Url`] on purpose: a `Target` cannot be built except by [`target`],
/// so no path reaches a read without having passed the checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    /// The address the request carried, for the log line and for the failure
    /// the caller is told about. A confined path is never echoed back.
    address: String,
    read: Read,
    /// How much of it will be read. It travels with the vetted address rather
    /// than being passed to [`fetch`] beside it, because it is one more thing
    /// this deployment decided about this target before anything was opened,
    /// and a `Target` is where those decisions live.
    cap: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Read {
    /// A resolved path inside one of the directories this deployment serves.
    File(PathBuf),
    /// A text this deployment stored for a teacher who asked to keep it.
    Text(TextId),
    /// A page on the public network.
    Web(Url),
}

impl Target {
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Whether this target is read from this deployment's own storage rather
    /// than over the network — a file under an upload directory, or a text in
    /// the store.
    pub fn is_stored(&self) -> bool {
        matches!(self.read, Read::File(_) | Read::Text(_))
    }
}

/// Reads the address a request points at, refusing everything this deployment
/// will not fetch. Nothing is opened here: a refusal costs no connection and
/// no directory listing beyond resolving the path a `file:` address names.
///
/// The parsing is here rather than at the endpoints because what an address
/// means and what may be reached with it are one decision. A string with no
/// scheme is taken as `http`, so a learner may type a bare host; a string
/// beginning with `/` is not an address at all but a reference to something
/// this deployment holds, and is read as one before any absolutising is done
/// to it.
// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+8]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+8]
pub fn target(raw: &str, config: &Config) -> std::result::Result<Target, Refusal> {
    let raw = raw.trim();
    let cap = config.max_page_bytes;

    // A reference to something this deployment stored. It is recognised
    // before anything else because it has no scheme to recognise it by, and
    // it is recognised by its path alone because it carries no host for a
    // caller to have chosen — see the module documentation.
    if raw.starts_with('/') {
        let named = raw.strip_prefix(TEXTS_PATH).ok_or(Refusal::Scheme)?;
        let id = TextId::parse(named).ok_or(Refusal::Address)?;
        return Ok(Target {
            address: id.reference(),
            read: Read::Text(id),
            cap,
        });
    }

    let url = page_url(raw)?;
    let address = url.to_string();
    match url.scheme() {
        "file" => {
            let path = url.to_file_path().map_err(|()| Refusal::Confined)?;
            let confined = confine(&path, &served_roots(config)).ok_or(Refusal::Confined)?;
            Ok(Target {
                address,
                read: Read::File(confined),
                cap,
            })
        }
        "http" | "https" => {
            public_host(&url)?;
            Ok(Target {
                address,
                read: Read::Web(url),
                cap,
            })
        }
        _ => Err(Refusal::Scheme),
    }
}

/// An address a request carried, read as an absolute one. A string carrying
/// no scheme is taken as `http`, which is what a learner types into an
/// address field.
///
/// This only parses. Whether the address is one this deployment will read is
/// [`target`]'s decision, which is the only caller.
fn page_url(raw: &str) -> std::result::Result<Url, Refusal> {
    let absolute = if raw.contains("://") || raw.starts_with("file:") {
        raw.to_string()
    } else {
        format!("http://{raw}")
    };
    Url::parse(&absolute).map_err(|_| Refusal::Address)
}

/// Reads a vetted target. A `file:` target is read from disk, a stored-text
/// one from the store, and a web one is fetched through the shared client,
/// under the concurrency bound, and then reduced to its main content.
///
/// This is where the reduction is scoped, because this is where the
/// difference it turns on is known. A page off the network is a stranger's
/// whole document, menus and all, and is cut down to the part of it somebody
/// wrote; a page this deployment holds — an accepted upload, kept or
/// temporary — is one it was given, and is answered as it was written.
/// Neither endpoint chooses, and an inline body never arrives here.
///
/// The store is handed in rather than reached for, so a `Target` stays a
/// decision about an address and nothing else, and so the endpoints that read
/// one keep the store they were built with. A deployment that has none holds
/// no stored text, so a reference to one names nothing there — which is the
/// answer a name that was never stored gets, and is why the reference shape is
/// still recognised rather than refused: what the caller asked for is a text,
/// and this deployment does not have it.
// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+8]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+8]
// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn+8]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn+8]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.reader-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.page-cap-fn+1]
pub async fn fetch(target: Target, texts: Option<&TextStore>) -> Result<String> {
    let Target { address, read, cap } = target;
    match read {
        Read::File(path) => {
            let read = tokio::task::spawn_blocking(move || read_file(&path, &address, cap));
            read.await
                .map_err(|join| anyhow!("the read ended: {join}"))?
        }
        Read::Text(id) => {
            let Some(texts) = texts else {
                return Err(Missing(id.to_string()).into());
            };
            let bytes = texts.get(&id, cap).await?;
            String::from_utf8(bytes)
                .map_err(|error| anyhow::Error::new(error).context(Unreachable(address)))
        }
        Read::Web(url) => {
            let _slot = slot().await?;
            // The reduction runs on the same blocking thread the fetch did.
            // It parses the page twice over, which is work the reactor should
            // not be doing, and the extractor's own document is not `Send`,
            // so it must live and die inside one closure.
            let fetch = tokio::task::spawn_blocking(move || {
                let page = get(&url, &address, cap)?;
                Ok(reader::reduce(page, url.as_str()))
            });
            fetch
                .await
                .map_err(|join| anyhow!("the fetch ended: {join}"))?
        }
    }
}

/// The directories this deployment mints `file:` URLs under, canonicalised.
///
/// The two upload directories are the whole list, because an accepted upload
/// is the only `file:` URL a client is ever handed. There is no shipped page
/// tree to add to them: a topic is a handful of tag lists in `topics.toml`
/// now, not a directory of its own with files a deployment serves out of.
///
/// A directory that does not resolve is left out rather than compared
/// against unresolved, so a deployment naming a directory that is not there
/// confines more tightly rather than less — and a deployment that named no
/// keep directory at all confines to the temporary one alone. With neither
/// present nothing is servable and every `file:` address is refused.
fn served_roots(config: &Config) -> Vec<PathBuf> {
    config
        .upload_keep_dir
        .iter()
        .chain(std::iter::once(&config.upload_temp_dir))
        .filter_map(|directory| directory.canonicalize().ok())
        .collect()
}

/// The resolved path, if it lands inside one of the roots. `starts_with`
/// compares whole components, so a root of `/srv/upload` does not admit
/// `/srv/upload-elsewhere`.
fn confine(path: &Path, roots: &[PathBuf]) -> Option<PathBuf> {
    let resolved = resolve(path)?;
    roots
        .iter()
        .any(|root| resolved.starts_with(root))
        .then_some(resolved)
}

/// A path with every symlink on it resolved, without requiring that the path
/// itself exists.
///
/// Walks up to the deepest ancestor that does exist, canonicalises that, and
/// appends the components below it unchanged. A component that is not a plain
/// name — `..` above everything that exists, or a bare root — gives up rather
/// than guessing, because a `..` that cannot be resolved against a real
/// directory cannot be confined either.
fn resolve(path: &Path) -> Option<PathBuf> {
    let mut below: Vec<OsString> = Vec::new();
    let mut current = path.to_path_buf();

    loop {
        if let Ok(mut resolved) = current.canonicalize() {
            for name in below.iter().rev() {
                resolved.push(name);
            }
            return Some(resolved);
        }

        let name = current.file_name()?.to_os_string();
        below.push(name);
        current = current.parent()?.to_path_buf();
        if current.as_os_str().is_empty() {
            return None;
        }
    }
}

/// Whether a web address names somewhere on the public network, as far as can
/// be told without resolving it. A host that is an IP literal is judged now; a
/// host that is a name is judged by the resolver, except for the names RFC
/// 6761 reserves for the local host, which are known without a lookup.
fn public_host(url: &Url) -> std::result::Result<(), Refusal> {
    let Some(host) = url.host_str() else {
        return Err(Refusal::Scheme);
    };

    // `host_str` brackets an IPv6 literal, as the address itself is written.
    let literal = host.trim_start_matches('[').trim_end_matches(']');
    if let Ok(ip) = literal.parse::<IpAddr>() {
        return if is_public(ip) {
            Ok(())
        } else {
            Err(Refusal::Private)
        };
    }

    if names_the_local_host(host) {
        return Err(Refusal::Private);
    }

    Ok(())
}

/// `localhost` and anything under it, which RFC 6761 reserves for the local
/// host whatever a resolver would say.
fn names_the_local_host(host: &str) -> bool {
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    host == "localhost" || host.ends_with(".localhost")
}

/// Whether an address is one a stranger's page could legitimately live at.
pub fn is_public(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => is_public_v4(ip),
        IpAddr::V6(ip) => is_public_v6(ip),
    }
}

fn is_public_v4(ip: Ipv4Addr) -> bool {
    if ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_multicast()
        || ip.is_broadcast()
        || ip.is_documentation()
    {
        return false;
    }

    let [a, b, c, _] = ip.octets();
    // 0.0.0.0/8, "this network", which the local host answers on.
    if a == 0 {
        return false;
    }
    // 100.64.0.0/10, carrier-grade NAT.
    if a == 100 && (64..128).contains(&b) {
        return false;
    }
    // 192.0.0.0/24, IETF protocol assignments.
    if a == 192 && b == 0 && c == 0 {
        return false;
    }
    // 198.18.0.0/15, benchmarking.
    if a == 198 && (b == 18 || b == 19) {
        return false;
    }
    // 240.0.0.0/4, reserved.
    if a >= 240 {
        return false;
    }

    true
}

fn is_public_v6(ip: Ipv6Addr) -> bool {
    let segments = ip.segments();

    // The forms that carry an IPv4 address are judged by where that address
    // would deliver, since that is where the packet lands.
    if let Some(embedded) = ip.to_ipv4_mapped() {
        return is_public_v4(embedded);
    }
    // 2002::/16, 6to4: the address sits in the two segments after the prefix.
    if segments[0] == 0x2002 {
        return is_public_v4(embedded_v4(segments[1], segments[2]));
    }
    // 64:ff9b::/96, the NAT64 well-known prefix.
    if segments[..6] == [0x0064, 0xff9b, 0, 0, 0, 0] {
        return is_public_v4(embedded_v4(segments[6], segments[7]));
    }

    if ip.is_loopback() || ip.is_unspecified() || ip.is_multicast() {
        return false;
    }
    // ::a.b.c.d, the deprecated IPv4-compatible form.
    if let Some(embedded) = ip.to_ipv4() {
        return is_public_v4(embedded);
    }
    // fc00::/7, unique local.
    if segments[0] & 0xfe00 == 0xfc00 {
        return false;
    }
    // fe80::/10, link local.
    if segments[0] & 0xffc0 == 0xfe80 {
        return false;
    }
    // 2001:db8::/32, documentation.
    if segments[0] == 0x2001 && segments[1] == 0x0db8 {
        return false;
    }
    // 100::/64, discard-only.
    if segments[0] == 0x0100 && segments[1..4] == [0, 0, 0] {
        return false;
    }

    true
}

fn embedded_v4(high: u16, low: u16) -> Ipv4Addr {
    Ipv4Addr::new((high >> 8) as u8, high as u8, (low >> 8) as u8, low as u8)
}

/// The one client every fetch goes through.
static CLIENT: LazyLock<reqwest::blocking::Client> = LazyLock::new(|| {
    reqwest::blocking::Client::builder()
        .timeout(FETCH_TIMEOUT)
        .connect_timeout(CONNECT_TIMEOUT)
        .user_agent(USER_AGENT)
        .dns_resolver(Arc::new(PublicAddressesOnly))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= MAX_REDIRECTS {
                return attempt.error(anyhow!("the redirect chain is too long"));
            }
            match attempt.url().scheme() {
                "http" | "https" => {}
                _ => return attempt.error(Refusal::Scheme),
            }
            match public_host(attempt.url()) {
                Ok(()) => attempt.follow(),
                Err(refusal) => attempt.error(refusal),
            }
        }))
        .build()
        .expect("the fetch client is built from constants")
});

/// The slots a web fetch runs in.
static SLOTS: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(MAX_CONCURRENT_FETCHES));

/// Waits for a fetch slot, giving up after [`FETCH_TIMEOUT`].
async fn slot() -> Result<SemaphorePermit<'static>> {
    let slots: &'static Semaphore = &SLOTS;
    match tokio::time::timeout(FETCH_TIMEOUT, slots.acquire()).await {
        Ok(Ok(permit)) => Ok(permit),
        // The semaphore is a static that is never closed.
        Ok(Err(closed)) => Err(anyhow::Error::new(closed)),
        Err(_elapsed) => Err(Overloaded.into()),
    }
}

/// A page off the disk, under the same cap a fetched one is read under.
///
/// The cap applies here too. Every `file:` address this deployment will read
/// names something it stored itself, and it stores nothing over the upload
/// limit — so a larger file in a served directory is a deployment whose
/// directories hold something it did not put there, and reading it whole is
/// not the way to find that out.
///
/// The weight is decided before the bytes are read as text, so an oversized
/// page says it is oversized whatever encoding it is in rather than failing
/// as invalid UTF-8 at whichever byte the cap fell inside.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.page-cap-fn+1]
fn read_file(path: &Path, address: &str, cap: usize) -> Result<String> {
    let unreachable = || Unreachable(address.to_string());

    let file = std::fs::File::open(path)
        .map_err(|error| anyhow::Error::new(error).context(unreachable()))?;
    let bytes = capped(file, cap, address)?;
    String::from_utf8(bytes).map_err(|error| anyhow::Error::new(error).context(unreachable()))
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.page-cap-fn+1]
fn get(url: &Url, address: &str, cap: usize) -> Result<String> {
    let unreachable = || Unreachable(address.to_string());

    let response = CLIENT
        .get(url.clone())
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|error| anyhow::Error::new(error).context(unreachable()))?;

    // The charset is read off the headers before the body is taken, because
    // taking it consumes the response. This is what `Response::text` would
    // have done for us, and it is what is given up by reading the body
    // ourselves rather than letting it buffer however much arrives.
    let encoding = charset(response.headers());
    let bytes = capped(response, cap, address)?;
    let (page, _, _) = encoding.decode(&bytes);
    Ok(page.into_owned())
}

/// Reads a page up to `cap` bytes, refusing at it rather than truncating.
///
/// One byte past the cap is read and the read then stops, so nothing beyond
/// the cap is ever held and a far end streaming without end is abandoned at
/// the cap instead of filling memory until the deadline. A page that is
/// exactly the cap is read; a page that is one byte more is refused, and is
/// refused rather than cut down because half a document analysed as a whole
/// one is a worse answer than none.
// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.page-cap-fn+1]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.page-cap-fn+1]
fn capped(source: impl std::io::Read, cap: usize, address: &str) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    source
        .take(cap as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| anyhow::Error::new(error).context(Unreachable(address.to_string())))?;

    if bytes.len() > cap {
        return Err(Oversized {
            address: address.to_string(),
            cap,
        }
        .into());
    }
    Ok(bytes)
}

/// The encoding a response declares, as `Response::text` reads one: the
/// `charset` parameter of the content type when there is one and the label
/// names an encoding, and UTF-8 otherwise.
fn charset(headers: &HeaderMap) -> &'static Encoding {
    headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(charset_parameter)
        .and_then(|label| Encoding::for_label(label.as_bytes()))
        .unwrap_or(UTF_8)
}

/// The `charset` parameter of a content-type header value.
fn charset_parameter(content_type: &str) -> Option<&str> {
    content_type.split(';').skip(1).find_map(|parameter| {
        let (name, value) = parameter.split_once('=')?;
        name.trim()
            .eq_ignore_ascii_case("charset")
            .then(|| value.trim().trim_matches('"'))
    })
}

/// The client's DNS resolver: the system one, with every private address
/// dropped from the answer. The addresses it returns are the ones the
/// connector connects to, so what was checked here is what is reached.
struct PublicAddressesOnly;

impl Resolve for PublicAddressesOnly {
    fn resolve(&self, name: Name) -> Resolving {
        let host = name.as_str().to_string();
        Box::pin(async move {
            // The system resolver blocks, and the client's runtime is a single
            // thread shared by every fetch in flight.
            let resolved = tokio::task::spawn_blocking(move || public_addresses(&host))
                .await
                .map_err(|join| anyhow!("the lookup ended: {join}"))?;
            let addresses = resolved?;
            Ok(Box::new(addresses.into_iter()) as Addrs)
        })
    }
}

fn public_addresses(host: &str) -> Result<Vec<SocketAddr>> {
    // Port zero: reqwest fills in the scheme's port, or the one the address
    // named, once the answer is back.
    let resolved: Vec<SocketAddr> = (host, 0u16).to_socket_addrs()?.collect();
    let public: Vec<SocketAddr> = resolved
        .into_iter()
        .filter(|address| is_public(address.ip()))
        .collect();

    if public.is_empty() {
        return Err(anyhow!("{host} has no address on the public network"));
    }
    Ok(public)
}

#[cfg(test)]
#[path = "fetch_tests.rs"]
mod tests;
