mod app;
mod cache;
mod config;
mod middleware;
mod projects;
mod utils;

use std::sync::Arc;

use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    router::{LayerFn, Path, Router, RouterBuilderDiscoverExt},
};
use tracing_subscriber::EnvFilter;

use crate::{cache::TTLCache, config::AppConfig, middleware::security_headers};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

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
