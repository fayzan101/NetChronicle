mod app;
mod config;
mod routes;
mod state;

use anyhow::Context;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::config::ApiConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = ApiConfig::from_env().context("invalid API configuration")?;
    let pool = netchronicle_db::create_pool(&config.database_url).await?;
    netchronicle_db::run_migrations(&pool).await?;

    let addr = config.socket_addr();
    let app = app::create_app(pool);

    info!(%addr, "NetChronicle API listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
