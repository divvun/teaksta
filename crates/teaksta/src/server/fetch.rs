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
//! # `file:` confinement
//!
//! The `file:` scheme exists because an accepted upload is handed back as a
//! `file:` URL and then fetched by the enhancement endpoints, so it cannot
//! simply be dropped. It is confined instead: the path is resolved and must
//! land inside one of the directories this deployment itself mints `file:`
//! URLs under, which [`served_roots`] derives from the deployment
//! configuration — the two upload directories, the activity tree and the
//! expanded web application root. Anything else — `/etc/passwd`, a key under
//! `~/.ssh`, the analysis cache — is refused with the same 400 the endpoint
//! answers any other unusable address with.
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
//! A far end that answers slowly and forever. The whole fetch is bounded by
//! [`FETCH_TIMEOUT`], which bounds the body too, but not by a byte count.
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
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs as _};
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};
use std::time::Duration;

use anyhow::{Result, anyhow};
use reqwest::Url;
use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use tokio::sync::{Semaphore, SemaphorePermit};

use crate::context::Config;
use crate::server::reader;

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
    #[error("only http, https and file addresses can be enhanced")]
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Read {
    /// A resolved path inside one of the directories this deployment serves.
    File(PathBuf),
    /// A page on the public network.
    Web(Url),
}

impl Target {
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Whether this target is read from disk rather than over the network.
    pub fn is_file(&self) -> bool {
        matches!(self.read, Read::File(_))
    }
}

/// Reads the address a request points at, refusing everything this deployment
/// will not fetch. Nothing is opened here: a refusal costs no connection and
/// no directory listing beyond resolving the path a `file:` address names.
// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+4]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+4]
pub fn target(url: &Url, config: &Config) -> std::result::Result<Target, Refusal> {
    let address = url.to_string();
    match url.scheme() {
        "file" => {
            let path = url.to_file_path().map_err(|()| Refusal::Confined)?;
            let confined = confine(&path, &served_roots(config)).ok_or(Refusal::Confined)?;
            Ok(Target {
                address,
                read: Read::File(confined),
            })
        }
        "http" | "https" => {
            public_host(url)?;
            Ok(Target {
                address,
                read: Read::Web(url.clone()),
            })
        }
        _ => Err(Refusal::Scheme),
    }
}

/// Reads a vetted target. A `file:` target is read from disk; a web one is
/// fetched through the shared client, under the concurrency bound, and then
/// reduced to its main content.
///
/// This is where the reduction is scoped, because this is where the
/// difference it turns on is known. A page off the network is a stranger's
/// whole document, menus and all, and is cut down to the part of it somebody
/// wrote; a page off the disk is one this deployment was given — an accepted
/// upload, or a page shipped with an activity — and is answered as it was
/// written. Neither endpoint chooses, and an inline body never arrives here.
// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+4]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+4]
// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn+6]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn+6]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.reader-fn]
pub async fn fetch(target: Target) -> Result<String> {
    let Target { address, read } = target;
    match read {
        Read::File(path) => {
            let read = tokio::task::spawn_blocking(move || read_file(&path, &address));
            read.await
                .map_err(|join| anyhow!("the read ended: {join}"))?
        }
        Read::Web(url) => {
            let _slot = slot().await?;
            // The reduction runs on the same blocking thread the fetch did.
            // It parses the page twice over, which is work the reactor should
            // not be doing, and the extractor's own document is not `Send`,
            // so it must live and die inside one closure.
            let fetch = tokio::task::spawn_blocking(move || {
                let page = get(&url, &address)?;
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
/// The two upload directories are where an accepted upload is stored, which
/// is the only `file:` URL a client is ever handed. The activity tree and the
/// web application root are where the shipped pages an activity points at
/// live; the activity tree is named separately because a deployment may put
/// it outside the web application root.
///
/// A directory that does not resolve is left out rather than compared
/// against unresolved, so a deployment naming a directory that is not there
/// confines more tightly rather than less. With none of them present nothing
/// is servable and every `file:` address is refused.
fn served_roots(config: &Config) -> Vec<PathBuf> {
    [
        &config.upload_keep_dir,
        &config.upload_temp_dir,
        &config.activities_dir,
        &config.webapp_root,
    ]
    .into_iter()
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

fn read_file(path: &Path, address: &str) -> Result<String> {
    std::fs::read_to_string(path)
        .map_err(|error| anyhow::Error::new(error).context(Unreachable(address.to_string())))
}

fn get(url: &Url, address: &str) -> Result<String> {
    let unreachable = || Unreachable(address.to_string());

    let response = CLIENT
        .get(url.clone())
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|error| anyhow::Error::new(error).context(unreachable()))?;
    response
        .text()
        .map_err(|error| anyhow::Error::new(error).context(unreachable()))
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
