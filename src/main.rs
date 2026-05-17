mod app;
mod config;
mod core;
mod api;
mod modules;

use std::net::SocketAddr;

use anyhow::Context;
use tracing_subscriber::EnvFilter;

use crate::app::build_app;
use crate::config::AppConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = AppConfig::from_env().context("failed to load configuration")?;
    init_tracing(&config.log_level)?;

    let app = build_app(config.clone()).await.context("failed to build app")?;
    let addr = SocketAddr::from(([0, 0, 0, 0], config.app_port));

    tracing::info!(%addr, "starting server");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("failed to bind tcp listener")?;

    axum::serve(listener, app)
        .await
        .map_err(|err| anyhow::anyhow!("server exited with error: {err}"))?;

    Ok(())
}

fn init_tracing(level: &str) -> anyhow::Result<()> {
    let filter = EnvFilter::try_new(level).unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .json()
        .with_env_filter(filter)
        .with_current_span(false)
        .with_target(true)
        .try_init()
        .map_err(|err| anyhow::anyhow!("failed to initialize tracing: {err}"))?;

    Ok(())
}
