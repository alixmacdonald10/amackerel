use std::collections::BTreeMap;

use reqwest::header::HeaderMap;
use serde::{Deserialize, Deserializer};

use crate::{
    config::AppConfig,
    utils::io::{
        github::{compile_github_headers, GITHUB_API_URL, GITHUB_USERNAME},
        request::GraphQLResult,
    },
};

/// Metadata for a single GitHub project, shown as a card on the homepage.
#[derive(Clone, Debug, Deserialize)]
pub struct RepositoryMeta {
    pub name: String,
    pub description: String,
    #[serde(
        rename = "primaryLanguage",
        deserialize_with = "deserialize_primary_language"
    )]
    pub primary_language: String,
    #[serde(rename = "languages", deserialize_with = "deserialize_languages")]
    pub languages: Vec<String>,
    #[serde(rename = "stargazerCount")]
    pub stars: u32,
    pub url: String,
}

fn deserialize_languages<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Languages {
        nodes: Vec<Language>,
    }

    #[derive(Deserialize)]
    struct Language {
        name: String,
    }

    let langs = Languages::deserialize(deserializer)?;
    Ok(langs.nodes.into_iter().map(|l| l.name).collect())
}

fn deserialize_primary_language<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Language {
        name: String,
    }

    let lang = Language::deserialize(deserializer)?;
    Ok(lang.name)
}

/// Returns the curated project list, stale-while-revalidate
///
/// Any cached list is served immediately and an expired one is refreshed in the background, so
/// only a cold start ever waits on GitHub. `Unavailable` means there is nothing at all to show.
pub async fn load_projects(
    app_config: &AppConfig,
    client: &reqwest::Client,
    curated_repos: &[&str],
) -> anyhow::Result<BTreeMap<String, RepositoryMeta>> {
    // TODO: have a compile query helper func
    let mut repos_query = vec![];
    for repo in curated_repos {
        repos_query.push(format!(r#"
        {repo}: repository(owner: "{GITHUB_USERNAME}", name: "{repo}") {{ 
            name description url stargazerCount primaryLanguage {{ name }} languages(first:10) {{ nodes  {{ name }} }} 
        }}"#));
    }
    let query = format!(r#"query {{ {} }}"#, repos_query.join(" "));

    let headers = compile_github_headers(app_config)?;

    let res = get_users_repo_data(client, &headers, &query).await?;
    match res {
        GraphQLResult::Ok(repos) => Ok(repos),
        GraphQLResult::Err(error) => {
            Err(anyhow::anyhow!("Error running GraphQL Query: {error:#?}"))
        }
    }
}

/// Returns all public repositories for a specific Github user
async fn get_users_repo_data(
    client: &reqwest::Client,
    headers: &HeaderMap,
    query: &str,
) -> anyhow::Result<GraphQLResult<BTreeMap<String, RepositoryMeta>>> {
    let body = serde_json::json!({ "query": query, "variables": {} });

    let resp = client
        .post(format!("{GITHUB_API_URL}/graphql"))
        .headers(headers.to_owned())
        .json(&body)
        .send()
        .await?;

    Ok(resp
        .json::<GraphQLResult<BTreeMap<String, RepositoryMeta>>>()
        .await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils::io::request::GraphQLError;

    #[test]
    fn repository_meta_deserializes_primary_language_and_languages_from_graphql_shape() {
        let json = serde_json::json!({
            "name": "chronofile",
            "description": "A time-travelable file for Rust.",
            "url": "https://github.com/alixmacdonald10/chronofile",
            "stargazerCount": 3,
            "primaryLanguage": { "name": "Rust" },
            "languages": { "nodes": [{ "name": "Rust" }, { "name": "Shell" }] }
        });

        let repo: RepositoryMeta = serde_json::from_value(json).expect("should deserialize");

        assert_eq!(repo.name, "chronofile");
        assert_eq!(repo.primary_language, "Rust");
        assert_eq!(repo.languages, vec!["Rust".to_string(), "Shell".to_string()]);
        assert_eq!(repo.stars, 3);
    }

    #[test]
    fn repository_meta_languages_can_include_the_primary_language() {
        let json = serde_json::json!({
            "name": "amackerel",
            "description": "Developer website",
            "url": "https://github.com/alixmacdonald10/amackerel",
            "stargazerCount": 0,
            "primaryLanguage": { "name": "Rust" },
            "languages": { "nodes": [{ "name": "Rust" }] }
        });

        let repo: RepositoryMeta = serde_json::from_value(json).expect("should deserialize");

        assert_eq!(repo.languages, vec!["Rust".to_string()]);
    }

    #[test]
    fn load_projects_query_deserializes_a_multi_repo_success_response() {
        let json = serde_json::json!({
            "data": {
                "amackerel": {
                    "name": "amackerel",
                    "description": "Developer website",
                    "url": "https://github.com/alixmacdonald10/amackerel",
                    "stargazerCount": 1,
                    "primaryLanguage": { "name": "Rust" },
                    "languages": { "nodes": [{ "name": "Rust" }] }
                },
                "chronofile": {
                    "name": "chronofile",
                    "description": "A time-travelable file for Rust.",
                    "url": "https://github.com/alixmacdonald10/chronofile",
                    "stargazerCount": 0,
                    "primaryLanguage": { "name": "Rust" },
                    "languages": { "nodes": [{ "name": "Rust" }] }
                }
            }
        });

        let result: GraphQLResult<BTreeMap<String, RepositoryMeta>> =
            serde_json::from_value(json).expect("should deserialize");

        match result {
            GraphQLResult::Ok(repos) => {
                assert_eq!(repos.len(), 2);
                assert!(repos.contains_key("amackerel"));
                assert!(repos.contains_key("chronofile"));
            }
            other => panic!("expected a successful result, got {other:?}"),
        }
    }

    #[test]
    fn load_projects_query_deserializes_a_graphql_error_response() {
        let json = serde_json::json!({
            "errors": [
                { "message": "Could not resolve to a Repository." }
            ]
        });

        let result: GraphQLResult<BTreeMap<String, RepositoryMeta>> =
            serde_json::from_value(json).expect("should deserialize");

        match result {
            GraphQLResult::Err(GraphQLError::Query(messages)) => {
                assert_eq!(messages, vec!["Could not resolve to a Repository."]);
            }
            other => panic!("expected a query error, got {other:?}"),
        }
    }
}
