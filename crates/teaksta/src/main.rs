//! Server binary.
//!
//! Everything the deployment needs is read from the environment: where the
//! expanded web application lives, where the built web client lives, where the
//! activity descriptors live, where uploads and analysed documents are kept,
//! and what address to listen on. The models are named by the two variables
//! the morpho seam reads.

use anyhow::Result;
use poem::listener::TcpListener;
use poem::{EndpointExt, Server};
use std::sync::Arc;
use tracing::{info, warn};

use teaksta::context::{
    ACTIVITIES_DIR_ENV, ANALYSIS_DIR_ENV, CLASSPATH_ENV, Config, LISTEN_ENV, UPLOAD_KEEP_DIR_ENV,
    UPLOAD_TEMP_DIR_ENV, WEBAPP_DIST_ENV, WEBAPP_ROOT_ENV,
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
        "{WEBAPP_ROOT_ENV}: webapp root {}",
        config.webapp_root.display()
    );
    info!(
        "{ACTIVITIES_DIR_ENV}: activities under {}",
        config.activities_dir.display()
    );
    info!(
        "{ANALYSIS_DIR_ENV}: analysed documents cached under {}",
        config.analysis_dir.display()
    );
    info!(
        "{UPLOAD_KEEP_DIR_ENV}/{UPLOAD_TEMP_DIR_ENV}: uploads under {} and {}",
        config.upload_keep_dir.display(),
        config.upload_temp_dir.display()
    );

    // The descriptor root always has a value, so an absent tree is what is
    // worth saying: without one every topic loads but answers unavailable.
    let classpath = &config.classpath_root;
    if classpath.join("operators").is_dir() {
        info!("{CLASSPATH_ENV}: descriptors under {}", classpath.display());
    } else {
        warn!(
            "{CLASSPATH_ENV}: no descriptor tree under {}; every topic will report itself unavailable",
            classpath.display()
        );
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
