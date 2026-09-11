//! Server binary.
//!
//! Everything the deployment needs is read from the environment: where the
//! built web client lives, which topics file to read if not the compiled-in
//! one, where uploads and analysed documents are kept, and what address to
//! listen on. The models are named by the two variables the morpho seam
//! reads.

use anyhow::Result;
use poem::listener::TcpListener;
use poem::{EndpointExt, Server};
use std::sync::Arc;
use tracing::{info, warn};

use teaksta::context::{
    ANALYSIS_DIR_ENV, Config, LISTEN_ENV, TOPICS_ENV, UPLOAD_KEEP_DIR_ENV, UPLOAD_TEMP_DIR_ENV,
    WEBAPP_DIST_ENV,
};
use teaksta::morpho::{BUNDLE_ENV, GENERATOR_ENV};
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
}
