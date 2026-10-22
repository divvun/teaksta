//! Verification and chain completion over the certificates of the case that
//! was reported: kursa.oahpa.no sending its `*.oahpa.no` certificate, issued
//! by GÉANT TLS ECC 1, beside the intermediates of the chain it replaced.
//! Nothing here opens a socket; an issuer address is answered by a closure.

use super::*;

use std::cell::RefCell;

const LEAF: &[u8] = include_bytes!("../../../tests/fixtures/certificates/oahpa-leaf.der");
const GEANT: &[u8] = include_bytes!("../../../tests/fixtures/certificates/geant-tls-ecc-1.der");
const GEANT_PEM: &[u8] = include_bytes!("../../../tests/fixtures/certificates/geant-tls-ecc-1.pem");

/// Where the leaf says its issuer is.
const GEANT_ADDRESS: &str = "http://crt.harica.gr/HARICA-GEANT-TLS-E1.cer";

/// 2026-06-01, inside the leaf's year of validity, so the tests verify the
/// same chain after the certificate itself has expired.
fn while_valid() -> UnixTime {
    UnixTime::since_unix_epoch(Duration::from_secs(1_780_272_000))
}

fn verifier() -> CompletingVerifier {
    CompletingVerifier::new(
        mozilla_roots(),
        Arc::new(rustls::crypto::ring::default_provider()),
        Arc::new(Intermediates::default()),
    )
}

fn kursa() -> ServerName<'static> {
    ServerName::try_from("kursa.oahpa.no").expect("a host name")
}

fn leaf() -> CertificateDer<'static> {
    CertificateDer::from(LEAF.to_vec())
}

/// A TLS failure as a request carries it: inside an I/O error, inside a
/// second I/O error, under a cause of the client's own — the shape reqwest's
/// blocking client answers a refused certificate with.
#[derive(Debug, thiserror::Error)]
#[error("the request failed")]
struct Request(#[source] std::io::Error);

fn as_a_request_fails(error: TlsError) -> Request {
    let handshake = std::io::Error::new(std::io::ErrorKind::InvalidData, error);
    Request(std::io::Error::other(handshake))
}

#[test]
fn the_reported_certificate_names_its_issuer() {
    assert_eq!(issuer_addresses(&leaf()), [GEANT_ADDRESS]);
}

/// The leaf alone leads to no root, and the handshake fails carrying it, so
/// the fetch that failed can complete the chain.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.issuer-fn/test]
#[test]
fn a_short_chain_fails_carrying_its_certificate() {
    let refused = verifier()
        .verify_server_cert(&leaf(), &[], &kursa(), &[], while_valid())
        .expect_err("the leaf alone leads to no root");
    let failed = as_a_request_fails(refused);

    let missing = missing_issuer(&failed).expect("the failure carries the certificate");
    assert_eq!(missing.leaf, leaf());
    assert_eq!(missing.server_name, kursa());
    assert!(certificate_refused(&failed));
}

/// The issuer is fetched from the address the certificate names, the chain
/// then verifies, and the issuer is held so the next handshake needs no fetch.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.issuer-fn/test]
#[test]
fn a_short_chain_is_completed_from_its_issuer() {
    let verifier = verifier();
    let refused = verifier
        .verify_server_cert(&leaf(), &[], &kursa(), &[], while_valid())
        .expect_err("the leaf alone leads to no root");
    let missing = missing_issuer(&as_a_request_fails(refused)).expect("a missing issuer");

    let asked = RefCell::new(Vec::new());
    let completed = complete(
        &missing,
        &verifier,
        |address| {
            asked.borrow_mut().push(address.to_string());
            (address == GEANT_ADDRESS).then(|| GEANT.to_vec())
        },
        while_valid(),
    );

    assert!(completed);
    assert_eq!(*asked.borrow(), [GEANT_ADDRESS]);
    verifier
        .verify_server_cert(&leaf(), &[], &kursa(), &[], while_valid())
        .expect("the held issuer completes the chain");
}

/// What an issuer address answers is held only once it has completed a
/// chain, and a chain that leads nowhere stops rather than looping.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.issuer-fn/test]
#[test]
fn an_answer_that_completes_nothing_is_not_held() {
    let verifier = verifier();
    let missing = MissingIssuer {
        leaf: leaf(),
        presented: Vec::new(),
        server_name: kursa(),
    };

    let asked = RefCell::new(0);
    let completed = complete(
        &missing,
        &verifier,
        |_| {
            *asked.borrow_mut() += 1;
            Some(LEAF.to_vec())
        },
        while_valid(),
    );

    assert!(!completed);
    assert!(*asked.borrow() <= MAX_FETCHES);
    assert!(verifier.held.all().is_empty());
}

/// A certificate for another host is refused as such, with the issuer at hand:
/// completing a chain does nothing for a certificate that is wrong.
#[test]
fn another_hosts_certificate_is_no_missing_issuer() {
    let verifier = verifier();
    let elsewhere = ServerName::try_from("example.org").expect("a host name");
    let geant = CertificateDer::from(GEANT.to_vec());

    let refused = verifier
        .verify_server_cert(&leaf(), &[geant], &elsewhere, &[], while_valid())
        .expect_err("the certificate names oahpa.no");
    let failed = as_a_request_fails(refused);

    assert!(missing_issuer(&failed).is_none());
    assert!(certificate_refused(&failed));
}

#[test]
fn an_issuer_is_read_as_der_or_pem() {
    let der = read_certificate(GEANT).expect("DER");
    assert_eq!(read_certificate(GEANT_PEM), Some(der));
    assert_eq!(read_certificate(b"<html>not found</html>"), None);
    assert_eq!(read_certificate(b""), None);
}

#[test]
fn another_failure_is_not_the_certificate() {
    let failed = Request(std::io::Error::new(
        std::io::ErrorKind::ConnectionRefused,
        "refused",
    ));

    assert!(missing_issuer(&failed).is_none());
    assert!(!certificate_refused(&failed));
}
