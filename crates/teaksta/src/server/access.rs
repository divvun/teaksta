//! The front door: who is asking, how often they may ask, and what is written
//! down about it.
//!
//! Every endpoint here is anonymous and three of them are expensive — a page
//! the analysis cache has never seen costs seconds of CPU across a pool of
//! language-technology handles — so a service reachable by strangers has to
//! be able to say no to one of them without saying no to the rest. That needs
//! one thing the HTTP layer does not otherwise have: a name for the client.
//!
//! # Who the client is
//!
//! By default, the peer that opened the connection, and nothing else. A
//! request's `X-Forwarded-For` and `X-Real-IP` are written by whoever sent
//! the request unless something in front of this process overwrote them, so
//! reading one on a bare deployment would let a single caller present itself
//! as a different client on every request — the limiter would count carefully
//! and bound nothing.
//!
//! `TEAKSTA_TRUST_PROXY=1` is the operator stating the other arrangement:
//! that exactly one hop they control sits in front of this process, and that
//! the hop appends the peer it saw to `X-Forwarded-For`. Under that
//! arrangement the **last** entry of the header is the one the trusted hop
//! wrote, and everything to its left is whatever the hop before it was
//! willing to believe — at the far left, whatever the client typed. So the
//! last entry is read and the header is never walked leftwards. A client that
//! sends `X-Forwarded-For: 9.9.9.9` has its own address appended after that
//! by the hop, and is counted as itself.
//!
//! Two hops is a misconfiguration this cannot detect, and its failure mode is
//! chosen deliberately: the last entry is then the inner hop's own address,
//! every client lands in one bucket, and the deployment answers 429 to
//! everybody. That is loud and is fixed by putting one hop in front. The
//! alternative — searching leftwards for the first entry that looks like a
//! public address — would survive two hops and would also let any client pick
//! its own bucket, which is a silent hole rather than a loud fault.
//!
//! `X-Real-IP` is read only when there is no `X-Forwarded-For` at all, since
//! the same hop writes both and a deployment behind a proxy that sets only
//! the one is ordinary.
//!
//! # What the limiter holds
//!
//! One [`governor`] GCRA cell per client, in memory, keyed by address. The
//! store is swept every [`SWEEP_EVERY`] checks: `retain_recent` drops every
//! key whose allowance has fully replenished, which is every client that is
//! no longer being counted. What is left is therefore the clients still
//! inside their window plus at most one sweep interval of new ones — a bound
//! that does not grow with uptime or with how many distinct addresses have
//! ever been seen, only with how many are active at once.
//!
//! Nothing is persisted and nothing is shared between processes. A restart
//! forgives everybody, and two replicas each count their own share of the
//! traffic; both are fine for a limit whose job is to keep one caller from
//! taking the machine, and neither is a quota anybody is billed against.

use std::net::{IpAddr, SocketAddr};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use governor::clock::{Clock, DefaultClock};
use governor::state::keyed::DefaultKeyedStateStore;
use governor::{Quota, RateLimiter};
use poem::http::StatusCode;
use poem::http::header::RETRY_AFTER;
use poem::web::Json;
use poem::{Endpoint, IntoResponse, Middleware, Request, Response};
use serde_json::json;
use tracing::{info, warn};

use crate::context::{Config, RateLimit};

/// How many checks pass between sweeps of the keyed store. Small enough that
/// the residue between sweeps is negligible beside the clients being counted,
/// large enough that walking the store is not per-request work.
const SWEEP_EVERY: u64 = 512;

/// How many clients may be inside their window before the operator is told.
/// Nothing is dropped at it — the store is bounded by the sweep either way —
/// but a deployment counting this many callers at once is being used by
/// something other than a classroom, and that is worth a line.
const CROWDED: usize = 100_000;

/// What the body of a refused request says. It is a code rather than a
/// sentence for the same reason an upload's rejection is one: a client shows
/// the learner its own words, in their language, and reads this to decide
/// which words.
const RATE_LIMITED: &str = "rate-limited";

/// Which client a request is from.
///
/// `None` is a request whose peer is not an internet address — a Unix socket,
/// or the in-process test transport. Every such request counts as one client,
/// which is the conservative reading: a caller nobody can tell apart from
/// another is not thereby unlimited.
// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.client-ip-fn]
pub type Client = Option<IpAddr>;

/// The address a request is counted and logged against, by whichever of the
/// two rules this deployment configured.
// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.client-ip-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.client-ip-fn]
pub fn client(request: &Request, trust_proxy: bool) -> Client {
    if trust_proxy {
        if let Some(forwarded) = forwarded(request) {
            return Some(forwarded);
        }
    }
    request
        .remote_addr()
        .as_socket_addr()
        .map(std::net::SocketAddr::ip)
}

/// What the trusted hop in front of this process said, if it said anything.
///
/// The last `X-Forwarded-For` entry, and only the last: see the module
/// documentation for why the header is not walked leftwards. An entry that is
/// not an address means the header is unusable rather than that the next one
/// along should be tried, so `X-Real-IP` is consulted and then the peer.
fn forwarded(request: &Request) -> Option<IpAddr> {
    let headers = request.headers();
    headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|chain| chain.rsplit(',').next())
        .and_then(address)
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|value| value.to_str().ok())
                .and_then(address)
        })
}

/// One forwarding-header entry read as an address. A proxy may write the bare
/// address, an address with the port it was seen on, or a bracketed IPv6
/// literal, and all three name the same client.
fn address(entry: &str) -> Option<IpAddr> {
    let entry = entry.trim().trim_matches('"').trim();
    if let Ok(ip) = entry.parse::<IpAddr>() {
        return Some(ip);
    }
    if let Ok(socket) = entry.parse::<SocketAddr>() {
        return Some(socket.ip());
    }
    entry
        .trim_start_matches('[')
        .trim_end_matches(']')
        .parse()
        .ok()
}

/// A client as a log line names it.
fn named(client: Client) -> String {
    match client {
        Some(ip) => ip.to_string(),
        None => "unknown".to_string(),
    }
}

/// The limiter itself, and the sweep that keeps its keys from accumulating.
struct Gate {
    limiter: RateLimiter<Client, DefaultKeyedStateStore<Client>, DefaultClock>,
    trust_proxy: bool,
    /// How many checks have been made, which is what the sweep is paced by.
    checks: AtomicU64,
}

impl Gate {
    fn new(limit: RateLimit, trust_proxy: bool) -> Self {
        // A zero anywhere would mean a deployment that answers nobody, which
        // is never what was meant; `TEAKSTA_RATE_LIMIT=off` is how a limit is
        // turned off. The environment cannot produce one, so this only holds
        // the line for a configuration built in code.
        let burst = NonZeroU32::new(limit.burst).unwrap_or(NonZeroU32::MIN);
        let count = NonZeroU32::new(limit.count).unwrap_or(NonZeroU32::MIN);
        let quota = Quota::with_period(limit.period / count.get())
            .unwrap_or_else(|| Quota::per_second(count))
            .allow_burst(burst);

        Gate {
            limiter: RateLimiter::keyed(quota),
            trust_proxy,
            checks: AtomicU64::new(0),
        }
    }

    /// Whether this client may be served now, and how long until it may be if
    /// not.
    fn admits(&self, client: Client) -> Result<(), Duration> {
        self.sweep();
        self.limiter
            .check_key(&client)
            .map_err(|until| until.wait_time_from(self.limiter.clock().now()))
    }

    /// Drops the clients that are no longer being counted, every
    /// [`SWEEP_EVERY`] checks. The work is done by whichever request happens
    /// to land on the interval rather than by a task of its own, so a router
    /// that is built and dropped — as every test's is — leaves nothing
    /// running behind it.
    fn sweep(&self) {
        if self.checks.fetch_add(1, Ordering::Relaxed) % SWEEP_EVERY != 0 {
            return;
        }
        self.limiter.retain_recent();
        self.limiter.shrink_to_fit();

        let tracked = self.limiter.len();
        if tracked > CROWDED {
            warn!("{tracked} clients are inside their rate-limit window at once");
        }
    }
}

/// Per-client rate limiting, for the routes that carry it.
///
/// One value covers every route it is applied to, so a client's allowance is
/// spent across the analysis endpoints together rather than four times over:
/// what is being protected is one pool of language-technology handles, and
/// which endpoint asked it to work is not the pool's concern.
///
/// A deployment with no configured limit builds one of these carrying no
/// gate, which passes every request through. That is the one shape the type
/// has for "not limited", so no route has to be registered two ways.
// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.rate-limit-fn+1]
#[derive(Clone)]
pub struct Limit {
    gate: Option<Arc<Gate>>,
}

impl Limit {
    /// The limiter this deployment's analysis endpoints share.
    pub fn new(config: &Config) -> Self {
        Limit {
            gate: config
                .rate_limit
                .map(|limit| Arc::new(Gate::new(limit, config.trust_proxy))),
        }
    }
}

impl<E: Endpoint> Middleware<E> for Limit {
    type Output = Limited<E>;

    fn transform(&self, inner: E) -> Self::Output {
        Limited {
            inner,
            gate: self.gate.clone(),
        }
    }
}

/// One route behind the limiter.
pub struct Limited<E> {
    inner: E,
    gate: Option<Arc<Gate>>,
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.rate-limit-fn+1]
impl<E: Endpoint> Endpoint for Limited<E> {
    type Output = Response;

    async fn call(&self, request: Request) -> poem::Result<Response> {
        let Some(gate) = &self.gate else {
            return self
                .inner
                .call(request)
                .await
                .map(IntoResponse::into_response);
        };

        let client = client(&request, gate.trust_proxy);
        if let Err(wait) = gate.admits(client) {
            info!(
                "Rate limited {} on {}",
                named(client),
                request.original_uri().path()
            );
            return Ok(too_many(wait));
        }

        self.inner
            .call(request)
            .await
            .map(IntoResponse::into_response)
    }
}

/// What a client over its allowance is answered with: the same
/// single-`error`-member JSON body an upload's closed gate answers with, and
/// the wait rounded up to whole seconds so a client told to come back in a
/// second does not come back a fraction of one too early.
fn too_many(wait: Duration) -> Response {
    let seconds = (wait.as_secs() + u64::from(wait.subsec_nanos() > 0)).max(1);
    Json(json!({ "error": RATE_LIMITED }))
        .with_status(StatusCode::TOO_MANY_REQUESTS)
        .with_header(RETRY_AFTER, seconds)
        .into_response()
}

/// One line for every request, whatever answered it.
///
/// It carries the client as the limiter counts it, the method, the path, the
/// status and how long the answer took. The endpoints that analyse log a line
/// of their own naming the address and the exercise; this one is the layer
/// under that, and covers the static client and every refusal that never
/// reached a handler as well.
///
/// The path is logged without its query. A learner's request carries the
/// address of whatever they are reading in it, and an access log is a file
/// that is kept, copied and read by people with no business knowing what any
/// particular learner was practising on.
// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.access-log-fn]
#[derive(Debug, Clone, Copy)]
pub struct AccessLog {
    trust_proxy: bool,
}

impl AccessLog {
    pub fn new(trust_proxy: bool) -> Self {
        AccessLog { trust_proxy }
    }
}

impl<E: Endpoint> Middleware<E> for AccessLog {
    type Output = Logged<E>;

    fn transform(&self, inner: E) -> Self::Output {
        Logged {
            inner,
            trust_proxy: self.trust_proxy,
        }
    }
}

/// The whole map behind the access log.
pub struct Logged<E> {
    inner: E,
    trust_proxy: bool,
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.access-log-fn]
impl<E: Endpoint> Endpoint for Logged<E> {
    type Output = Response;

    async fn call(&self, request: Request) -> poem::Result<Response> {
        let client = named(client(&request, self.trust_proxy));
        let method = request.method().clone();
        // The address as it arrived. This layer stands outside the map, so
        // nothing has rewritten it yet for a nested route to match against,
        // and the path here is the one the request was sent to.
        let path = request.uri().path().to_string();

        let started = Instant::now();
        let answered = self
            .inner
            .call(request)
            .await
            .map(IntoResponse::into_response);
        let elapsed = started.elapsed();

        let status = match &answered {
            Ok(response) => response.status(),
            Err(error) => error.status(),
        };
        info!(
            client = %client,
            method = %method,
            path = %path,
            status = status.as_u16(),
            duration_ms = elapsed.as_millis() as u64,
            "access"
        );

        answered
    }
}

#[cfg(test)]
#[path = "access_tests.rs"]
mod tests;
