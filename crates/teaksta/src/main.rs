//! Server binary.
//!
//! Everything the deployment needs is read from the environment: where the
//! built web client lives, which topics file to read if not the compiled-in
//! one, where uploads and analysed documents are kept, what address to listen
//! on, and what a service reachable by strangers will do for one of them. The
//! models are named by the two variables the morpho seam reads.

use anyhow::Result;
use poem::listener::TcpListener;
use poem::{EndpointExt, Server};
use std::sync::Arc;
use tracing::{info, warn};

use teaksta::context::{
    ANALYSIS_DIR_ENV, AZURE_ACCESS_KEY_ENV, AZURE_ACCOUNT_ENV, AZURE_CONTAINER_ENV, Config,
    LISTEN_ENV, MAX_PAGE_BYTES_ENV, RATE_LIMIT_BURST_ENV, RATE_LIMIT_ENV, TOPICS_ENV,
    TRUST_PROXY_ENV, UPLOAD_KEEP_DIR_ENV, UPLOAD_TEMP_DIR_ENV, WEBAPP_DIST_ENV,
};
use teaksta::morpho::{BUNDLE_ENV, GENERATOR_ENV, WORKERS_ENV, analysis_workers};
use teaksta::server::api::{AppState, routes};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::from_env()?;
    report(&config);

    let listen = config.listen.clone();
    let state = Arc::new(AppState::new(config)?);
    let app = routes(&state.config).data(state);

    info!("Listening on {listen}");
    Server::new(TcpListener::bind(listen)).run(app).await?;
    Ok(())
}

/// What this deployment was configured to be, logged before it serves
/// anything. Every variable the server reads is reported, so an operator
/// reading the first screenful sees the whole configured surface rather than
/// the two or three that happened to be worth a warning. A value that is
/// merely defaulted is a log line; one whose absence will fail requests, or
/// one naming something that is not there, is a warning.
fn report(config: &Config) {
    info!("{LISTEN_ENV}: listening on {}", config.listen);
    info!(
        "{ANALYSIS_DIR_ENV}: analysed documents cached under {}",
        config.analysis_dir.display()
    );
    info!(
        "{UPLOAD_KEEP_DIR_ENV}/{UPLOAD_TEMP_DIR_ENV}: uploads under {} and {}",
        config.upload_keep_dir.display(),
        config.upload_temp_dir.display()
    );

    // Which of the two the deployment keeps texts in is the difference between
    // a kept text that is there next term and one that goes with the pod, so
    // it is said before anything is served rather than left to be inferred
    // from three variables. The account key is never among what is printed.
    match &config.azure {
        Some(azure) => info!(
            "{AZURE_ACCOUNT_ENV}/{AZURE_CONTAINER_ENV}/{AZURE_ACCESS_KEY_ENV}: kept texts stored \
             in the container {} on the account {}",
            azure.container, azure.account
        ),
        None => warn!(
            "{AZURE_ACCOUNT_ENV} is not set; kept texts are stored under {} and last exactly as \
             long as that directory does",
            config.upload_keep_dir.display()
        ),
    }

    info!(
        "{MAX_PAGE_BYTES_ENV}: a page is read up to {} bytes and abandoned there",
        config.max_page_bytes
    );

    // A limit that is off is a warning, because a deployment anyone can reach
    // with the analysis endpoints unmetered is one request away from spending
    // its machine on whoever asked first.
    match config.rate_limit {
        Some(limit) => info!(
            "{RATE_LIMIT_ENV}/{RATE_LIMIT_BURST_ENV}: one client may make {} requests per {:?} of \
             the endpoints that analyse, {} of them at once",
            limit.count, limit.period, limit.burst
        ),
        None => warn!("{RATE_LIMIT_ENV} is off; the endpoints that analyse are not rate limited"),
    }

    // Trusting the headers is the unusual setting, but neither state is a
    // warning: one is wrong behind a proxy and the other is wrong without
    // one, and which is which is the operator's to know.
    if config.trust_proxy {
        info!(
            "{TRUST_PROXY_ENV}: a client is the last X-Forwarded-For entry, so exactly one \
             trusted hop must sit in front of this process"
        );
    } else {
        info!(
            "{TRUST_PROXY_ENV} is not set; a client is the peer address and forwarding headers \
             are ignored"
        );
    }

    // An unset topics file is not a warning: the registry compiled into this
    // binary is the deployment's, and a named one is the exception.
    match &config.topics {
        Some(topics) => info!("{TOPICS_ENV}: topics read from {}", topics.display()),
        None => info!("{TOPICS_ENV} is not set; the compiled-in topics are served"),
    }

    match &config.webapp_dist {
        Some(dist) => info!("{WEBAPP_DIST_ENV}: web client under {}", dist.display()),
        None => warn!("{WEBAPP_DIST_ENV} is not set; only the API is served"),
    }

    for name in [BUNDLE_ENV, GENERATOR_ENV] {
        match std::env::var(name) {
            Ok(path) => info!("{name}: {path}"),
            Err(_) => warn!("{name} is not set; every analysis request will fail"),
        }
    }

    // The effective count rather than the variable, because a value that
    // could not be read as a count is replaced by the derived one (which the
    // seam warns about where it happens) and the deployment is whichever of
    // the two is actually in force.
    let workers = analysis_workers();
    match std::env::var(WORKERS_ENV) {
        Ok(_) => info!("{WORKERS_ENV}: {workers} chunks of a document are analysed at once"),
        Err(_) => info!(
            "{WORKERS_ENV} is not set; {workers} chunks of a document are analysed at once, \
             derived from the machine"
        ),
    }
}
