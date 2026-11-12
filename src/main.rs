mod api;
mod app;
mod cache;
mod components;
mod config;
mod middleware;
mod projects;
mod utils;

use redis::ConnectionAddr;
use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    router::{LayerFn, Path, Router, RouterBuilderDiscoverExt},
};

use crate::{config::AppConfig, middleware::security_headers, utils::telemetry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = telemetry::setup_tracing()?;

    let http_client = reqwest::Client::new();
    let app_config = AppConfig::load()?;
    tracing::debug!("{app_config:#?}");

    let redis_connection_addr =
        ConnectionAddr::Tcp(app_config.redis_host.clone(), app_config.redis_port);
    let cache_client = redis::Client::open(redis_connection_addr)?;
    let cache_connection = cache_client.get_multiplexed_async_connection().await?;

    // TODO: add a tracing middleware
    let router = Router::builder()
        .discover()
        .app_context(http_client)
        .app_context(cache_connection)
        .app_context(app_config)
        .assets(AssetBundle::load().unwrap())
        .layer(LayerFn::new(None::<&Path>, security_headers))
        .build();

    topcoat::start(router).await?;
    Ok(())
}
