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
