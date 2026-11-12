use std::{sync::LazyLock, time::Duration};

use anyhow::Result;
use redis::aio::MultiplexedConnection;

use crate::{projects::RepositoryMeta, utils::io::github::CURATED_REPOS};

pub static REPO_CACHE_TTL: LazyLock<Duration> = LazyLock::new(|| Duration::from_mins(5));

pub fn default_redis_port() -> u16 {
    6379
}

pub async fn cache_repos(
    cache_connection: MultiplexedConnection,
    repos: Vec<RepositoryMeta>,
) -> Result<()> {
    if repos.is_empty() {
        return Ok(());
    }

    let mut cache_connection = cache_connection.clone();

    // MSET takes key value key value as args
    let mut command = redis::cmd("MSETEX");
    command.arg(repos.len()); // num keys
    for repo in repos {
        command.arg(generate_repo_cache_key(&repo.name));
        command.arg(repo);
    }
    // set expiry
    command.arg("EX");
    command.arg(REPO_CACHE_TTL.as_secs());

    let res = command.query_async::<()>(&mut cache_connection).await;
    match res {
        Ok(_) => {
            tracing::debug!("cached items");
            Ok(())
        }
        Err(err) => {
            tracing::error!("failed to cache items: {err:#?}");
            Err(anyhow::anyhow!(err))
        }
    }
}

pub async fn get_cached_repos(
    cache_connection: &MultiplexedConnection,
) -> Result<Vec<RepositoryMeta>> {
    let mut cache_connection = cache_connection.clone();

    let mut command = redis::cmd("MGET");
    for repo in CURATED_REPOS {
        command.arg(generate_repo_cache_key(repo));
    }

    Ok(command
        .query_async::<Vec<Option<RepositoryMeta>>>(&mut cache_connection)
        .await?
        .into_iter()
        .flatten()
        .collect::<Vec<RepositoryMeta>>())
}

/// Generates a cache key for a Repository
fn generate_repo_cache_key(repo: &str) -> String {
    format!("repo:{repo}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_cache_key_is_namespaced_by_repo_name() {
        assert_eq!(generate_repo_cache_key("amackerel"), "repo:amackerel");
    }

    #[test]
    fn default_redis_port_is_the_standard_port() {
        assert_eq!(default_redis_port(), 6379);
    }
}
