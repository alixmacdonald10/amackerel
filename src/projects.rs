use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Metadata for a single GitHub project, shown as a card on the homepage.
/// Crosses the wire, so it lives outside the ssr-only module.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub name: String,
    pub description: String,
    /// All languages GitHub detects, ordered most-used first.
    pub languages: Vec<String>,
    pub stars: u32,
    pub url: String,
    pub homepage: Option<String>,
}

/// Server-only machinery: reqwest, the cache, GitHub types. None of this can
/// compile to wasm, so the whole module is gated behind the `ssr` feature.
#[cfg(feature = "ssr")]
mod ssr {
    use super::ProjectMeta;
    use serde::Deserialize;
    use std::sync::{Mutex, OnceLock};
    use std::time::{Duration, Instant};

    /// Hand-picked repos to show off, as `owner/repo` slugs.
    /// Edit this list (and rebuild) to change which projects appear.
    const CURATED: &[&str] = &[
        "alixmacdonald10/amackerel",
        "alixmacdonald10/chronofile",
        "alixmacdonald10/checkerel",
    ];

    /// How long a fetched list stays fresh before we hit GitHub again.
    /// Keeps us well under the 60 req/hr unauthenticated rate limit.
    const CACHE_TTL: Duration = Duration::from_secs(15 * 60);

    static CACHE: OnceLock<Mutex<Option<(Instant, Vec<ProjectMeta>)>>> = OnceLock::new();

    /// Shared client, built once. Reused connection pool, and a request timeout
    /// so a hung GitHub socket can't wedge the server fn indefinitely.
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

    fn client() -> &'static reqwest::Client {
        CLIENT.get_or_init(|| {
            reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default()
        })
    }

    /// The subset of the GitHub repo API response we care about.
    #[derive(Deserialize)]
    struct GhRepo {
        html_url: String,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        language: Option<String>,
        #[serde(default)]
        stargazers_count: u32,
        #[serde(default)]
        homepage: Option<String>,
    }

    /// Builds a GitHub API GET with the required headers, plus an optional
    /// `GITHUB_TOKEN` (lifts the rate limit 60 -> 5000/hr; read at runtime only).
    fn gh_get(client: &reqwest::Client, url: String) -> reqwest::RequestBuilder {
        let mut req = client
            .get(url)
            .header("User-Agent", "amackerel")
            .header("Accept", "application/vnd.github+json");
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            if !token.is_empty() {
                req = req.header("Authorization", format!("Bearer {token}"));
            }
        }
        req
    }

    /// Fetches one repo's metadata plus its full language breakdown. Returns
    /// `None` on any error so a single bad slug doesn't sink the whole list.
    async fn fetch_repo(client: &reqwest::Client, slug: &str) -> Option<ProjectMeta> {
        let resp = gh_get(client, format!("https://api.github.com/repos/{slug}"))
            .send()
            .await
            .ok()?;
        if !resp.status().is_success() {
            return None;
        }
        let repo: GhRepo = resp.json().await.ok()?;

        // The repo endpoint gives only the primary language; a second call
        // returns every language by byte count. Sort most-used first.
        let mut languages = Vec::new();
        if let Ok(resp) = gh_get(
            client,
            format!("https://api.github.com/repos/{slug}/languages"),
        )
        .send()
        .await
        {
            if let Ok(map) = resp.json::<std::collections::HashMap<String, u64>>().await {
                let mut pairs: Vec<(String, u64)> = map.into_iter().collect();
                pairs.sort_by(|a, b| b.1.cmp(&a.1));
                languages = pairs.into_iter().map(|(name, _)| name).collect();
            }
        }
        // Fall back to the primary language if the breakdown call failed.
        if languages.is_empty() {
            if let Some(lang) = repo.language.filter(|l| !l.is_empty()) {
                languages.push(lang);
            }
        }

        Some(ProjectMeta {
            name: slug.rsplit('/').next().unwrap_or(slug).to_string(),
            description: repo.description.unwrap_or_default(),
            languages,
            stars: repo.stargazers_count,
            url: repo.html_url,
            homepage: repo.homepage.filter(|h| !h.trim().is_empty()),
        })
    }

    /// Returns the curated project list, served from cache when fresh. On a
    /// total fetch failure, falls back to a stale cache if present.
    pub async fn load_projects() -> Vec<ProjectMeta> {
        let cache = CACHE.get_or_init(|| Mutex::new(None));

        // Serve fresh cache.
        if let Ok(guard) = cache.lock() {
            if let Some((fetched_at, list)) = guard.as_ref() {
                if fetched_at.elapsed() < CACHE_TTL {
                    return list.clone();
                }
            }
        }

        // Fetch every repo concurrently; join_all preserves CURATED order,
        // which is the display order on the homepage.
        let client = client();
        let results =
            futures::future::join_all(CURATED.iter().map(|slug| fetch_repo(client, slug))).await;
        let projects: Vec<ProjectMeta> = results.into_iter().flatten().collect();

        // Only trust — and cache — a complete fetch. A single failed repo (rate
        // limit, transient 404, network) must not poison the fresh cache with a
        // short list for the next TTL. Serve stale cache if we have it, else the
        // partial list (better than nothing on a cold start).
        if projects.len() < CURATED.len() {
            if let Ok(guard) = cache.lock() {
                if let Some((_, list)) = guard.as_ref() {
                    return list.clone();
                }
            }
            return projects;
        }

        if let Ok(mut guard) = cache.lock() {
            *guard = Some((Instant::now(), projects.clone()));
        }
        projects
    }
}

/// Server function exposed at `/api`; the wasm client calls it over HTTP.
#[server(ListProjects, "/api")]
pub async fn list_projects() -> Result<Vec<ProjectMeta>, ServerFnError> {
    Ok(ssr::load_projects().await)
}
