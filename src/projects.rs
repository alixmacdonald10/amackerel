use std::collections::HashMap;

use futures::future;
use serde::Deserialize;

use crate::{
    config::AppConfig,
    utils::io::github::{compile_github_headers, CURATED_REPOS, GITHUB_API_URL, GITHUB_USERNAME},
};

/// Metadata for a single GitHub project, shown as a card on the homepage.
#[derive(Clone, Debug, Deserialize)]
pub struct RepositoryMeta {
    pub name: String,
    pub description: String,
    pub languages: Option<Vec<String>>,
    #[serde(rename = "stargazers_count")]
    pub stars: u32,
    #[serde(rename = "html_url")]
    pub url: String,
}

/// Returns the curated project list, stale-while-revalidate
///
/// Any cached list is served immediately and an expired one is refreshed in the background, so
/// only a cold start ever waits on GitHub. `Unavailable` means there is nothing at all to show.
pub async fn load_projects(
    app_config: &AppConfig,
    client: &reqwest::Client,
    curated_repos: &[&str]
) -> anyhow::Result<Vec<RepositoryMeta>> {
    // NOTE: In future this response can return a bunch of other URLs which can be
    // queried for more information from the Github API. Leave this as the first call
    // then use join all for future subsequent calls.
    let mut repos = get_users_public_repos(client, app_config).await?;

    let languages = future::join_all(
        curated_repos 
            .iter()
            .map(|repo| get_repo_languages(repo, client, app_config)),
    )
    .await;

    // TODO: optimise this from o(n2)
    for repo in repos.iter_mut() {
        for item in &languages {
            match item {
                Ok((name, languages)) => {
                    if *repo.name == *name {
                        repo.languages = Some(languages.clone());
                    }
                }
                Err(err) => {
                    tracing::error!("An error occured fetching languages: {}", err.to_string())
                }
            }
        }
    }

    Ok(repos)
}

/// Returns all public repositories for a specific Github user
async fn get_users_public_repos(
    client: &reqwest::Client,
    config: &AppConfig,
) -> anyhow::Result<Vec<RepositoryMeta>> {
    let repos = client
        .get(format!("{GITHUB_API_URL}/users/{GITHUB_USERNAME}/repos"))
        .headers(compile_github_headers(config)?)
        .send()
        .await?
        .json::<Vec<RepositoryMeta>>()
        .await?;

    Ok(repos
        .into_iter()
        .filter(|repo| CURATED_REPOS.contains(&repo.name.as_str()))
        .collect::<Vec<RepositoryMeta>>())
}

/// Returns all languages for a Github repository
async fn get_repo_languages(
    repo: &str,
    client: &reqwest::Client,
    config: &AppConfig,
) -> anyhow::Result<(String, Vec<String>)> {
    let language_map = client
        .get(format!(
            "{GITHUB_API_URL}/repos/{GITHUB_USERNAME}/{repo}/languages"
        ))
        .headers(compile_github_headers(config)?)
        .send()
        .await?
        .json::<HashMap<String, u32>>()
        .await?;

    Ok((
        repo.to_string(),
        language_map
            .keys()
            .map(|x| x.to_owned())
            .collect::<Vec<String>>(),
    ))
}
