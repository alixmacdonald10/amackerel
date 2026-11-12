use anyhow::Result;
use redis::aio::MultiplexedConnection;
use redis_macros::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

use crate::{
    cache::{cache_repos, get_cached_repos},
    config::AppConfig,
    utils::io::{
        github::{CURATED_REPOS, GITHUB_USERNAME},
        request::{
            perform_repositories_query,
            repositories_query::{self, RepositoriesQuerySearchNodes, ResponseData},
        },
    },
};

/// Metadata for a single GitHub project, shown as a card on the homepage.
#[derive(Clone, Debug, Deserialize, Serialize, FromRedisValue, ToRedisArgs)]
pub struct RepositoryMeta {
    pub name: String,
    pub description: String,
    pub primary_language: String,
    pub languages: Vec<String>,
    pub stars: u32,
    pub url: String,
}

/// Convert a GraphQL Response data into a useable Vec of repo metadata
impl From<ResponseData> for Vec<RepositoryMeta> {
    fn from(value: ResponseData) -> Self {
        value
            .search
            .nodes
            .unwrap_or_default()
            .into_iter()
            .flatten()
            .filter_map(|node| match node {
                RepositoriesQuerySearchNodes::Repository(repo) => Some(repo),
                _ => None,
            })
            .map(|repo| {
                let primary_language = if let Some(lang) = repo.primary_language {
                    lang.name
                } else {
                    "".to_string()
                };

                let languages = if let Some(languages) = repo.languages {
                    languages
                        .nodes
                        .unwrap_or_default()
                        .into_iter()
                        .flatten()
                        .map(|lang| lang.name)
                        .collect::<Vec<String>>()
                } else {
                    Vec::default()
                };
                RepositoryMeta {
                    name: repo.name,
                    description: repo.description.unwrap_or_default(),
                    primary_language,
                    languages,
                    stars: repo.stargazer_count as u32,
                    url: repo.url,
                }
            })
            .collect()
    }
}

pub async fn fetch_repositories(
    app_config: &AppConfig,
    cache_connection: &MultiplexedConnection,
    reqwest_client: &reqwest::Client,
) -> Result<Vec<RepositoryMeta>> {
    let mut repositories = get_cached_repos(cache_connection).await?;
    let missing = if repositories.is_empty() {
        // All repos are missing from cache
        CURATED_REPOS.to_vec()
    } else {
        // some or none are missing
        let cached = repositories
            .iter()
            .map(|repo| repo.name.as_str())
            .collect::<Vec<&str>>();

        CURATED_REPOS
            .iter()
            .cloned()
            .filter(|repo| !cached.contains(repo))
            .collect::<Vec<&str>>()
    };

    // fetch the missing repos from the API
    if !missing.is_empty() {
        let mut fetched = load_projects(app_config, reqwest_client, &missing).await?;
        // cache in separate thread so as not to block
        tokio::task::spawn(cache_repos(cache_connection.clone(), fetched.clone()));
        repositories.append(&mut fetched);
    }
    repositories.sort_by_key(|repo| repo.name.clone());

    Ok(repositories)
}

/// Returns the curated project list, stale-while-revalidate
///
/// Any cached list is served immediately and an expired one is refreshed in the background, so
/// only a cold start ever waits on GitHub. `Unavailable` means there is nothing at all to show.
pub async fn load_projects(
    app_config: &AppConfig,
    client: &reqwest::Client,
    curated_repos: &[&str],
) -> anyhow::Result<Vec<RepositoryMeta>> {
    let mut search_query = String::new();
    for repo in curated_repos {
        search_query += &format!("repo:{GITHUB_USERNAME}/{repo} ");
    }

    let variables = repositories_query::Variables { search_query };

    let response_data =
        perform_repositories_query(variables, client, &app_config.github_token).await?;
    Ok(response_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    use graphql_client::Response;

    /// Deserializes a raw GitHub GraphQL body the same way [perform_repositories_query] does.
    fn parse(json: serde_json::Value) -> Response<ResponseData> {
        serde_json::from_value(json).expect("should deserialize")
    }

    fn repos_from(json: serde_json::Value) -> Vec<RepositoryMeta> {
        parse(json).data.expect("response should carry data").into()
    }

    #[test]
    fn response_data_maps_a_multi_repo_success_response() {
        let repos = repos_from(serde_json::json!({
            "data": {
                "search": {
                    "nodes": [
                        {
                            "__typename": "Repository",
                            "name": "amackerel",
                            "description": "Developer website",
                            "url": "https://github.com/alixmacdonald10/amackerel",
                            "stargazerCount": 1,
                            "primaryLanguage": { "name": "Rust" },
                            "languages": { "nodes": [{ "name": "Rust" }, { "name": "CSS" }] }
                        },
                        {
                            "__typename": "Repository",
                            "name": "chronofile",
                            "description": "A time-travelable file for Rust.",
                            "url": "https://github.com/alixmacdonald10/chronofile",
                            "stargazerCount": 3,
                            "primaryLanguage": { "name": "Rust" },
                            "languages": { "nodes": [{ "name": "Rust" }] }
                        }
                    ]
                }
            }
        }));

        assert_eq!(repos.len(), 2);

        let amackerel = &repos[0];
        assert_eq!(amackerel.name, "amackerel");
        assert_eq!(amackerel.description, "Developer website");
        assert_eq!(amackerel.url, "https://github.com/alixmacdonald10/amackerel");
        assert_eq!(amackerel.stars, 1);
        assert_eq!(amackerel.primary_language, "Rust");
        assert_eq!(
            amackerel.languages,
            vec!["Rust".to_string(), "CSS".to_string()]
        );

        assert_eq!(repos[1].name, "chronofile");
        assert_eq!(repos[1].stars, 3);
    }

    #[test]
    fn response_data_defaults_missing_optional_fields() {
        let repos = repos_from(serde_json::json!({
            "data": {
                "search": {
                    "nodes": [{
                        "__typename": "Repository",
                        "name": "bare",
                        "description": null,
                        "url": "https://github.com/alixmacdonald10/bare",
                        "stargazerCount": 0,
                        "primaryLanguage": null,
                        "languages": null
                    }]
                }
            }
        }));

        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].description, "");
        assert_eq!(repos[0].primary_language, "");
        assert!(repos[0].languages.is_empty());
    }

    #[test]
    fn response_data_skips_null_and_non_repository_nodes() {
        let repos = repos_from(serde_json::json!({
            "data": {
                "search": {
                    "nodes": [
                        null,
                        { "__typename": "Issue" },
                        {
                            "__typename": "Repository",
                            "name": "amackerel",
                            "description": "Developer website",
                            "url": "https://github.com/alixmacdonald10/amackerel",
                            "stargazerCount": 0,
                            "primaryLanguage": { "name": "Rust" },
                            "languages": { "nodes": [null, { "name": "Rust" }] }
                        }
                    ]
                }
            }
        }));

        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].name, "amackerel");
        assert_eq!(repos[0].languages, vec!["Rust".to_string()]);
    }

    #[test]
    fn response_data_maps_an_empty_search_to_no_repos() {
        let repos = repos_from(serde_json::json!({
            "data": { "search": { "nodes": null } }
        }));

        assert!(repos.is_empty());
    }

    #[test]
    fn response_deserializes_a_graphql_error_body() {
        let response = parse(serde_json::json!({
            "errors": [
                { "message": "Could not resolve to a Repository." }
            ]
        }));

        assert!(response.data.is_none());
        let errors = response.errors.expect("response should carry errors");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Could not resolve to a Repository.");
    }

    #[test]
    fn repository_meta_round_trips_through_json() {
        // redis-macros stores `RepositoryMeta` in the cache as JSON.
        let repo = RepositoryMeta {
            name: "chronofile".to_string(),
            description: "A time-travelable file for Rust.".to_string(),
            primary_language: "Rust".to_string(),
            languages: vec!["Rust".to_string(), "Shell".to_string()],
            stars: 3,
            url: "https://github.com/alixmacdonald10/chronofile".to_string(),
        };

        let json = serde_json::to_string(&repo).expect("should serialize");
        let back: RepositoryMeta = serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(back.name, repo.name);
        assert_eq!(back.description, repo.description);
        assert_eq!(back.primary_language, repo.primary_language);
        assert_eq!(back.languages, repo.languages);
        assert_eq!(back.stars, repo.stars);
        assert_eq!(back.url, repo.url);
    }
}

