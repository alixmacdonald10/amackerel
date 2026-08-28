use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde::Deserialize;

/// Hand-picked repos to show off, as `owner/repo` slugs.
/// Edit this list (and rebuild) to change which projects appear.
const CURATED: &[&str] = &[
    "alixmacdonald10/amackerel",
    "alixmacdonald10/chronofile",
    "alixmacdonald10/tacklebox",
];

/// How long a fetched list stays fresh before we refresh it in the background.
/// Keeps us well under the 60 req/hr unauthenticated rate limit.
const CACHE_TTL: Duration = Duration::from_secs(15 * 60);

static CACHE: OnceLock<CacheSlot> = OnceLock::new();

/// True while a background refresh is in flight, so a burst of requests past
/// the TTL fires one refresh rather than one per request.
static REFRESHING: AtomicBool = AtomicBool::new(false);

/// Shared client, built once. Reused connection pool, and a request timeout
/// so a hung GitHub socket can't wedge the request indefinitely.
static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

/// Metadata for a single GitHub project, shown as a card on the homepage.
#[derive(Clone, Debug)]
pub struct ProjectMeta {
    pub name: String,
    pub description: String,
    /// All languages GitHub detects, ordered most-used first.
    pub languages: Vec<String>,
    pub stars: u32,
    pub url: String,
}

/// GitHub could not be reached and there is no cache to fall back on.
#[derive(Debug)]
pub struct Unavailable;

/// The cached list with the instant it was fetched; `None` until the first
/// complete fetch lands.
type CacheSlot = Mutex<Option<(Instant, Vec<ProjectMeta>)>>;

fn cache() -> &'static CacheSlot {
    CACHE.get_or_init(|| Mutex::new(None))
}

/// Clears `REFRESHING` on drop, so a panicking refresh task can't wedge
/// refreshes off for the rest of the process's life.
struct RefreshGuard;

impl Drop for RefreshGuard {
    fn drop(&mut self) {
        REFRESHING.store(false, Ordering::SeqCst);
    }
}

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
            pairs.sort_by_key(|b| std::cmp::Reverse(b.1));
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
    })
}

/// Fetches every curated repo concurrently;
///
/// `join_all` preserves `CURATED` order, which is the display order on the homepage.
async fn fetch_all() -> Vec<ProjectMeta> {
    let client = client();
    let results =
        futures::future::join_all(CURATED.iter().map(|slug| fetch_repo(client, slug))).await;
    results.into_iter().flatten().collect()
}

/// A clone of whatever is cached, fresh or stale, with the time it was fetched.
fn cached() -> Option<(Instant, Vec<ProjectMeta>)> {
    let guard = cache().lock().ok()?;
    (*guard).clone()
}

/// Caches `projects` only if the fetch was complete, and reports whether it
/// was.
///
/// A single failed repo (rate limit, transient 404, network) must not
/// poison the cache with a short list for the next TTL.
fn store_if_complete(projects: &[ProjectMeta]) -> bool {
    if projects.len() < CURATED.len() {
        return false;
    }
    if let Ok(mut guard) = cache().lock() {
        *guard = Some((Instant::now(), projects.to_vec()));
    }
    true
}

/// Refreshes the cache off the request path. Serving stale data while this runs
/// is what keeps a page render from ever waiting on GitHub.
pub fn spawn_refresh() {
    if REFRESHING.swap(true, Ordering::SeqCst) {
        // this was already refreshing
        return;
    }
    tokio::spawn(async {
        let _guard = RefreshGuard;
        store_if_complete(&fetch_all().await);
    });
}

/// Returns the curated project list, stale-while-revalidate
///
/// Any cached list is served immediately and an expired one is refreshed in the background, so
/// only a cold start ever waits on GitHub. `Unavailable` means there is nothing at all to show.
pub async fn load_projects() -> Result<Vec<ProjectMeta>, Unavailable> {
    if let Some((fetched_at, list)) = cached() {
        if fetched_at.elapsed() >= CACHE_TTL {
            spawn_refresh();
        }
        return Ok(list);
    }

    // Cold start: nothing to serve, so this request has to wait for the fetch.
    let projects = fetch_all().await;
    if store_if_complete(&projects) {
        return Ok(projects);
    }

    // Nothing fetched and no cache: GitHub is unreachable. Distinct from
    // "the curated list is empty", which is a legitimate empty state.
    if projects.is_empty() && !CURATED.is_empty() {
        return Err(Unavailable);
    }

    // Partial list — better than nothing on a cold start, and left uncached so
    // the next request retries.
    Ok(projects)
}
