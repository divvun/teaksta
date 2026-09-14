//! Who a request is from, as the two rules resolve it. What the limiter does
//! with that is exercised over the whole map in `api_tests.rs`, where the
//! routes it covers and the ones it does not are both reachable.

use super::*;

use std::net::Ipv4Addr;

use poem::Request;

/// One request carrying the named headers and no peer address, which is what
/// every request the in-process transport carries looks like.
fn forwarded_request(headers: &[(&str, &str)]) -> Request {
    let mut builder = Request::builder();
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }
    builder.finish()
}

fn ip(literal: &str) -> IpAddr {
    literal.parse().expect("an address")
}

/// Without the flag, a forwarding header is a string the caller wrote and is
/// read by nobody: every request is the same client, which is what stops a
/// caller from being as many clients as it can invent addresses.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.client-ip-fn/test]
#[test]
fn a_forwarding_header_is_ignored_unless_trusted() {
    for headers in [
        &[("x-forwarded-for", "203.0.113.1")][..],
        &[("x-real-ip", "203.0.113.2")][..],
        &[("x-forwarded-for", "203.0.113.3, 198.51.100.4")][..],
        &[("forwarded", "for=203.0.113.5")][..],
    ] {
        assert_eq!(client(&forwarded_request(headers), false), None);
    }
}

/// With it, the last entry is the client — the one the trusted hop appended,
/// which is the only entry in the header the caller could not have chosen.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.client-ip-fn/test]
#[test]
fn the_last_forwarded_entry_is_the_client() {
    let cases = [
        ("203.0.113.1", "203.0.113.1"),
        // What the caller claimed is to the left of what the hop saw.
        ("9.9.9.9, 203.0.113.1", "203.0.113.1"),
        ("9.9.9.9,203.0.113.1", "203.0.113.1"),
        ("  9.9.9.9 ,  203.0.113.1  ", "203.0.113.1"),
        // A hop that wrote the port it saw the client on, or bracketed an
        // IPv6 literal, still named the same client.
        ("203.0.113.1:41234", "203.0.113.1"),
        ("9.9.9.9, [2001:db8::1]:41234", "2001:db8::1"),
        ("[2001:db8::1]", "2001:db8::1"),
        ("2001:db8::1", "2001:db8::1"),
    ];

    for (header, expected) in cases {
        assert_eq!(
            client(&forwarded_request(&[("x-forwarded-for", header)]), true),
            Some(ip(expected)),
            "{header}"
        );
    }
}

/// `X-Real-IP` is the fallback for a proxy that writes only that one, and is
/// read only when there is no chain to read the last entry of.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.client-ip-fn/test]
#[test]
fn the_real_ip_header_is_the_fallback() {
    assert_eq!(
        client(&forwarded_request(&[("x-real-ip", "203.0.113.7")]), true),
        Some(ip("203.0.113.7"))
    );

    // A chain that is there and readable wins, whatever the other header says.
    assert_eq!(
        client(
            &forwarded_request(&[
                ("x-forwarded-for", "9.9.9.9, 203.0.113.8"),
                ("x-real-ip", "203.0.113.7"),
            ]),
            true
        ),
        Some(ip("203.0.113.8"))
    );

    // A last entry that is not an address makes the chain unusable rather
    // than making the entry before it the client: walking leftwards is
    // walking towards what the caller wrote.
    assert_eq!(
        client(
            &forwarded_request(&[
                ("x-forwarded-for", "203.0.113.9, not-an-address"),
                ("x-real-ip", "203.0.113.7"),
            ]),
            true
        ),
        Some(ip("203.0.113.7"))
    );
}

/// With the flag set and nothing forwarded, the peer is the client — which on
/// this transport is nobody, and a request from nobody is still one client.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.client-ip-fn/test]
#[test]
fn a_request_naming_nobody_is_one_client() {
    assert_eq!(client(&forwarded_request(&[]), true), None);
    assert_eq!(client(&forwarded_request(&[]), false), None);
    assert_eq!(named(None), "unknown");
    assert_eq!(
        named(Some(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1)))),
        "203.0.113.1"
    );
}

/// The bucket is emptied at the burst and refills at the sustained rate, and
/// the wait it reports is that rate's replenishment rather than nothing.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.rate-limit-fn+1/test]
#[test]
fn the_burst_is_spent_before_the_rate_binds() {
    let gate = Gate::new(
        RateLimit {
            burst: 3,
            // One an hour, so nothing replenishes while this test runs and
            // the outcome is decided by the requests made rather than by how
            // long the machine took to make them.
            count: 1,
            period: Duration::from_secs(60 * 60),
        },
        false,
    );
    let client = Some(ip("203.0.113.1"));

    for asked in 1..=3 {
        assert!(gate.admits(client).is_ok(), "request {asked} of the burst");
    }

    let Err(wait) = gate.admits(client) else {
        panic!("a fourth request must be over a burst of three");
    };
    assert!(wait > Duration::from_secs(60), "{wait:?}");

    // Another client has its own allowance and is untouched by the first
    // having spent theirs.
    assert!(gate.admits(Some(ip("203.0.113.2"))).is_ok());
}

/// The sweep drops the clients that are no longer being counted, so what the
/// store holds is the clients inside their window rather than every client
/// the deployment has ever seen.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.rate-limit-fn+1/test]
#[test]
fn the_store_holds_only_the_clients_being_counted() {
    // One request a nanosecond: a client's allowance is replenished before
    // the next one is counted, so every key is stale by the time it is swept.
    let gate = Gate::new(
        RateLimit {
            burst: 1,
            count: 1,
            period: Duration::from_nanos(1),
        },
        false,
    );

    // Two sweeps' worth of distinct clients and then one more, so the last
    // request made is the one a sweep ran on and what is left afterwards is
    // the residue rather than the interval since the last sweep.
    let asked = SWEEP_EVERY * 2;
    for octet in 0..=asked {
        let client = Some(IpAddr::V4(Ipv4Addr::from(octet as u32 + 1)));
        assert!(gate.admits(client).is_ok());
    }

    let held = gate.limiter.len();
    assert!(
        held <= 2,
        "{asked} distinct clients have been counted and {held} keys are still held"
    );
}

/// A wait is rounded up, so a client told to come back in a second does not
/// come back a fraction of one too early, and is never told to come back in
/// no time at all.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.rate-limit-fn+1/test]
#[test]
fn the_retry_after_is_whole_seconds_never_zero() {
    for (wait, expected) in [
        (Duration::ZERO, "1"),
        (Duration::from_millis(1), "1"),
        (Duration::from_secs(1), "1"),
        (Duration::from_millis(1001), "2"),
        (Duration::from_secs(120), "120"),
    ] {
        let response = too_many(wait);

        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(
            response
                .headers()
                .get(RETRY_AFTER)
                .and_then(|value| value.to_str().ok()),
            Some(expected),
            "{wait:?}"
        );
    }
}
