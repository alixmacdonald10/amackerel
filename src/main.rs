mod api;
mod app;
mod cache;
mod components;
mod config;
mod middleware;
mod projects;
mod utils;

use std::time::Duration;

use redis::{AsyncConnectionConfig, ConnectionAddr};
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
    let cache_connection = match redis::Client::open(redis_connection_addr) {
        Ok(client) => {
            let config =
                AsyncConnectionConfig::new().set_connection_timeout(Some(Duration::from_secs(2)));
            match client
                .get_multiplexed_async_connection_with_config(&config)
                .await
            {
                Ok(conn) => Some(conn),
                Err(err) => {
                    tracing::warn!("redis unavailable, running without cache: {err}");
                    None
                }
            }
        }
        Err(err) => {
            tracing::warn!("invalid redis config, running without cache: {err}");
            None
        }
    };

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
