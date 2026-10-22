//! Verifying a site's certificate, and completing a chain the site sent short.
//!
//! A page fetched over `https` is accepted on a certificate that leads to a
//! root in Mozilla's store, compiled in through `webpki-roots`. What this
//! module adds to the plain verification is the repair browsers make for a
//! site that sends its certificate without the intermediate that issued it:
//! the certificate says where its issuer can be fetched, and it is fetched.
//! Such a site opens in every browser, so nothing tells its owners it is
//! broken, and a fetcher that refused it would refuse pages that work
//! everywhere else.
//!
//! # No fetching in the handshake
//!
//! [`CompletingVerifier`] runs inside the TLS handshake, on the one runtime
//! thread every fetch in flight shares, so it fetches nothing: a slow issuer
//! address there would stall every other page being read. When no path is
//! found it tries once more with the intermediates already [`Intermediates`]
//! holds, and when that finds none either it fails the handshake carrying a
//! [`MissingIssuer`]. The fetch that failed reads it back off the error with
//! [`missing_issuer`], runs [`complete`] on its own blocking thread, and asks
//! for the page again.
//!
//! # A fetched intermediate grants nothing
//!
//! It is a candidate for path building, beside the ones the site sent, and a
//! chain through it is accepted only when every signature on the way verifies
//! and it ends at a trusted root. It is held only once it has completed such
//! a chain, so what is held is real CA certificates: a site cannot fill the
//! store with certificates named like a popular issuer to spend every later
//! verification's path-building budget on them.

use std::error::Error as StdError;
use std::sync::{Arc, LazyLock, RwLock};
use std::time::Duration;

use rustls::client::WebPkiServerVerifier;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::CryptoProvider;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{
    CertificateError, ClientConfig, DigitallySignedStruct, Error as TlsError, OtherError,
    RootCertStore, SignatureScheme,
};
use x509_parser::extensions::{GeneralName, ParsedExtension};
use x509_parser::oid_registry::OID_PKIX_ACCESS_DESCRIPTOR_CA_ISSUERS;
use x509_parser::pem::parse_x509_pem;
use x509_parser::prelude::{FromDer, X509Certificate};

/// How long fetching one issuer may take.
pub const ISSUER_TIMEOUT: Duration = Duration::from_secs(5);

/// How much of an issuer address's answer is read. An intermediate is a
/// kilobyte or two; anything near this is not one.
pub const ISSUER_CAP: usize = 16 * 1024;

/// How many links of a chain are followed upward from the site's certificate.
const MAX_LINKS: usize = 3;

/// How many addresses are fetched to complete one chain, all links together.
const MAX_FETCHES: usize = 4;

/// How many intermediates are held at once. A site met again is verified from
/// these without fetching anything; past the bound the oldest goes first.
const MAX_HELD: usize = 64;

/// Intermediates fetched from issuer addresses that completed a chain, held
/// for the life of the process.
#[derive(Debug, Default)]
pub struct Intermediates {
    held: RwLock<Vec<CertificateDer<'static>>>,
}

impl Intermediates {
    fn all(&self) -> Vec<CertificateDer<'static>> {
        self.held
            .read()
            .map(|held| held.clone())
            .unwrap_or_default()
    }

    fn hold(&self, certificates: &[CertificateDer<'static>]) {
        let Ok(mut held) = self.held.write() else {
            return;
        };
        for certificate in certificates {
            if held.contains(certificate) {
                continue;
            }
            if held.len() >= MAX_HELD {
                held.remove(0);
            }
            held.push(certificate.clone());
        }
    }
}

/// The site's certificate leads to no trusted root through what it sent and
/// what is held. Carried out of the handshake as the failure's cause, so the
/// fetch that failed can complete the chain where fetching is allowed.
#[derive(Debug, Clone, thiserror::Error)]
#[error("the certificate for {server_name:?} was sent without the intermediate that issued it")]
pub struct MissingIssuer {
    pub leaf: CertificateDer<'static>,
    pub presented: Vec<CertificateDer<'static>>,
    pub server_name: ServerName<'static>,
}

/// Mozilla's verification, which tries the held intermediates as well before
/// it gives up, and gives up with a [`MissingIssuer`] when no issuer is found.
// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.issuer-fn]
#[derive(Debug)]
pub struct CompletingVerifier {
    inner: Arc<WebPkiServerVerifier>,
    held: Arc<Intermediates>,
}

impl CompletingVerifier {
    pub fn new(
        roots: RootCertStore,
        provider: Arc<CryptoProvider>,
        held: Arc<Intermediates>,
    ) -> Self {
        let inner = WebPkiServerVerifier::builder_with_provider(Arc::new(roots), provider)
            .build()
            .expect("a verifier over a non-empty root store");
        Self { inner, held }
    }

    /// The verification, with `extra` offered beside what the site sent and
    /// what is held.
    // [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.issuer-fn]
    fn verify_with(
        &self,
        leaf: &CertificateDer<'_>,
        presented: &[CertificateDer<'_>],
        extra: &[CertificateDer<'static>],
        server_name: &ServerName<'_>,
        ocsp: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, TlsError> {
        let first = self
            .inner
            .verify_server_cert(leaf, presented, server_name, ocsp, now);
        if !is_unknown_issuer(&first) {
            return first;
        }

        let mut offered: Vec<CertificateDer<'_>> = presented.to_vec();
        offered.extend(self.held.all());
        offered.extend(extra.iter().cloned());
        let second = self
            .inner
            .verify_server_cert(leaf, &offered, server_name, ocsp, now);
        if !is_unknown_issuer(&second) {
            return second;
        }

        let missing = MissingIssuer {
            leaf: leaf.clone().into_owned(),
            presented: presented
                .iter()
                .map(|certificate| certificate.clone().into_owned())
                .collect(),
            server_name: server_name.to_owned(),
        };
        Err(TlsError::InvalidCertificate(CertificateError::Other(
            OtherError(Arc::new(missing)),
        )))
    }
}

fn is_unknown_issuer(verified: &Result<ServerCertVerified, TlsError>) -> bool {
    matches!(
        verified,
        Err(TlsError::InvalidCertificate(
            CertificateError::UnknownIssuer
        ))
    )
}

impl ServerCertVerifier for CompletingVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, TlsError> {
        self.verify_with(
            end_entity,
            intermediates,
            &[],
            server_name,
            ocsp_response,
            now,
        )
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TlsError> {
        self.inner.verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TlsError> {
        self.inner.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.inner.supported_verify_schemes()
    }
}

/// The roots every fetch is verified against.
pub fn mozilla_roots() -> RootCertStore {
    RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned())
}

/// The verifier the shared client's handshakes run, and that [`complete`]
/// verifies a completed chain with, over one store of held intermediates.
pub static VERIFIER: LazyLock<Arc<CompletingVerifier>> = LazyLock::new(|| {
    Arc::new(CompletingVerifier::new(
        mozilla_roots(),
        Arc::new(rustls::crypto::ring::default_provider()),
        Arc::new(Intermediates::default()),
    ))
});

/// The TLS configuration of the shared client. No ALPN is offered, so every
/// page is read over HTTP/1.1, one request to a connection's turn.
pub fn client_config() -> ClientConfig {
    ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
        .with_safe_default_protocol_versions()
        .expect("ring supports the default protocol versions")
        .dangerous()
        .with_custom_certificate_verifier(VERIFIER.clone())
        .with_no_client_auth()
}

/// The TLS failure behind a failed request, wherever in its chain of causes it
/// sits. A TLS failure reaches the client as an I/O error carrying it, inside
/// another I/O error, and an I/O error does not give what it carries as its
/// source — its source is the carried error's own source, one link further
/// on — so at each I/O error the carried error is followed instead.
fn tls_error<'a>(error: &'a (dyn StdError + 'static)) -> Option<&'a TlsError> {
    let mut current = Some(error);
    while let Some(error) = current {
        if let Some(found) = error.downcast_ref::<TlsError>() {
            return Some(found);
        }
        current = match error
            .downcast_ref::<std::io::Error>()
            .and_then(std::io::Error::get_ref)
        {
            Some(carried) => Some(carried as &(dyn StdError + 'static)),
            None => error.source(),
        };
    }
    None
}

/// The missing issuer a request failed for, if that is what it failed for.
pub fn missing_issuer(error: &(dyn StdError + 'static)) -> Option<MissingIssuer> {
    match tls_error(error)? {
        TlsError::InvalidCertificate(CertificateError::Other(OtherError(cause))) => {
            cause.downcast_ref::<MissingIssuer>().cloned()
        }
        _ => None,
    }
}

/// Whether a request failed because the site's certificate was not accepted.
pub fn certificate_refused(error: &(dyn StdError + 'static)) -> bool {
    matches!(tls_error(error), Some(TlsError::InvalidCertificate(_)))
}

/// Where a certificate says its issuer can be fetched: the CA Issuers
/// addresses of its authority information access extension, in the order it
/// lists them.
pub fn issuer_addresses(certificate: &CertificateDer<'_>) -> Vec<String> {
    let Ok((_, parsed)) = X509Certificate::from_der(certificate.as_ref()) else {
        return Vec::new();
    };
    parsed
        .extensions()
        .iter()
        .filter_map(|extension| match extension.parsed_extension() {
            ParsedExtension::AuthorityInfoAccess(access) => Some(access),
            _ => None,
        })
        .flat_map(|access| access.accessdescs.iter())
        .filter(|description| description.access_method == OID_PKIX_ACCESS_DESCRIPTOR_CA_ISSUERS)
        .filter_map(|description| match description.access_location {
            GeneralName::URI(address) => Some(address.to_string()),
            _ => None,
        })
        .collect()
}

/// What an issuer address answered, read as one certificate: DER, which is
/// what issuer addresses serve, or PEM. Anything else is no issuer.
pub fn read_certificate(bytes: &[u8]) -> Option<CertificateDer<'static>> {
    let der = if bytes.starts_with(b"-----BEGIN") {
        parse_x509_pem(bytes).ok()?.1.contents
    } else {
        bytes.to_vec()
    };
    X509Certificate::from_der(&der).ok()?;
    Some(CertificateDer::from(der))
}

/// Completes a chain the site sent short, fetching issuers upward from its
/// certificate with `fetch` until the chain verifies, and holds what completed
/// it. Answers whether the chain verifies with what was fetched.
///
/// `fetch` is handed each issuer address and answers its bytes or nothing;
/// the address is a stranger's, and vetting it is the caller's, which knows
/// the policy every other address is read under.
// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.issuer-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.issuer-fn]
pub fn complete(
    missing: &MissingIssuer,
    verifier: &CompletingVerifier,
    fetch: impl Fn(&str) -> Option<Vec<u8>>,
    now: UnixTime,
) -> bool {
    let mut found: Vec<CertificateDer<'static>> = Vec::new();
    let mut frontier = vec![missing.leaf.clone()];
    let mut fetches = 0;

    for _ in 0..MAX_LINKS {
        let mut fetched = Vec::new();
        for certificate in &frontier {
            for address in issuer_addresses(certificate) {
                if fetches == MAX_FETCHES {
                    break;
                }
                fetches += 1;
                if let Some(issuer) = fetch(&address).as_deref().and_then(read_certificate)
                    && !found.contains(&issuer)
                {
                    fetched.push(issuer);
                }
            }
        }
        if fetched.is_empty() {
            return false;
        }
        found.extend(fetched.iter().cloned());

        let verified = verifier.verify_with(
            &missing.leaf,
            &missing.presented,
            &found,
            &missing.server_name,
            &[],
            now,
        );
        if verified.is_ok() {
            verifier.held.hold(&found);
            return true;
        }
        frontier = fetched;
    }
    false
}

#[cfg(test)]
mod tests;
