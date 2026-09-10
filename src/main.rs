mod app;
mod cache;
mod components;
mod config;
mod middleware;
mod projects;
mod utils;

use std::sync::Arc;

use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    router::{LayerFn, Path, Router, RouterBuilderDiscoverExt},
};

use crate::{cache::TTLCache, config::AppConfig, middleware::security_headers, utils::telemetry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = telemetry::setup_tracing()?;

    let http_client = reqwest::Client::new();
    let app_config = AppConfig::load()?;
    tracing::debug!("{app_config:#?}");
    let ttl_cache = TTLCache::new();

    let router = Router::builder()
        .discover()
        .app_context(http_client)
        .app_context(app_config)
        .app_context(Arc::new(ttl_cache))
        .assets(AssetBundle::load().unwrap())
        .layer(LayerFn::new(None::<&Path>, security_headers))
        .build();

    topcoat::start(router).await?;
    Ok(())
}
