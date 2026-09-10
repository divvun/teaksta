//! Server binary.
//!
//! Everything the deployment needs is read from the environment: where the
//! expanded web application lives, where uploads and analysed documents are
//! kept, and what address to listen on. The models are named by the two
//! variables the morpho seam reads.

use anyhow::Result;
use poem::listener::TcpListener;
use poem::{EndpointExt, Server};
use std::sync::Arc;
use tracing::{info, warn};

use teaksta::context::Config;
use teaksta::morpho::{BUNDLE_ENV, GENERATOR_ENV};
use teaksta::server::activity_configuration::classpath_root;
use teaksta::server::api::{AppState, routes};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::from_env()?;
    for name in [BUNDLE_ENV, GENERATOR_ENV] {
        if std::env::var(name).is_err() {
            warn!("{name} is not set; every analysis request will fail");
        }
    }
    info!("Webapp root {}", config.webapp_root.display());
    let classpath = classpath_root();
    if classpath.join("operators").is_dir() {
        info!("Descriptors under {}", classpath.display());
    } else {
        warn!(
            "No descriptor tree under {}; every topic will report itself unavailable",
            classpath.display()
        );
    }

    let listen = config.listen.clone();
    let state = Arc::new(AppState::new(config)?);
    let app = routes().data(state);

    info!("Listening on {listen}");
    Server::new(TcpListener::bind(listen)).run(app).await?;
    Ok(())
}
