use std::sync::LazyLock;

use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use secrecy::ExposeSecret;

use crate::config::AppConfig;

pub const GITHUB_USERNAME: &str = "alixmacdonald10";

/// Hand-picked repos to show off, as bare repo names owned by [GITHUB_USERNAME].
/// Edit this list (and rebuild) to change which projects appear.
pub const CURATED_REPOS: &[&str] = &["amackerel", "chronofile"];

pub const GITHUB_API_URL: &str = "https://api.github.com";

/// Thread safe, lazy initialisation of a minimum viable [HeaderMap] for Github requests
pub static GITHUB_MIN_HEADERS: LazyLock<HeaderMap<HeaderValue>> = LazyLock::new(|| {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/vnd.github+json".parse().unwrap());
    headers.insert("X-GitHub-Api-Version", "2026-03-10".parse().unwrap());
    headers.insert(USER_AGENT, env!("CARGO_PKG_NAME").parse().unwrap());
    headers
});

pub static GITHUB_URL: LazyLock<String> = LazyLock::new(|| format!("https://github.com/{GITHUB_USERNAME}"));

pub fn compile_github_headers(config: &AppConfig) -> anyhow::Result<HeaderMap<HeaderValue>> {
    let mut map = GITHUB_MIN_HEADERS.clone();

    if let Some(ref token) = config.github_token {
        map.insert(
            AUTHORIZATION,
            format!("Bearer {}", token.expose_secret()).parse()?,
        );
    }
    tracing::debug!("{map:#?}");
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashSet;

    use secrecy::SecretString;

    fn config_with_token(token: &str) -> AppConfig {
        AppConfig {
            github_token: Some(SecretString::from(token)),
        }
    }

    fn assert_carries_min_headers(headers: &HeaderMap<HeaderValue>) {
        assert_eq!(headers[ACCEPT], "application/vnd.github+json");
        assert_eq!(headers["X-GitHub-Api-Version"], "2026-03-10");
        assert_eq!(headers[USER_AGENT], env!("CARGO_PKG_NAME"));
    }

    #[test]
    fn min_headers_carry_accept_api_version_and_user_agent() {
        assert_carries_min_headers(&GITHUB_MIN_HEADERS);
        assert!(!GITHUB_MIN_HEADERS.contains_key(AUTHORIZATION));
    }

    #[test]
    fn compile_headers_adds_authorization_when_a_token_is_present() {
        let config = config_with_token("ghp_test");

        let headers = compile_github_headers(&config).expect("headers should compile");

        assert_eq!(headers[AUTHORIZATION], "Bearer ghp_test");
    }

    #[test]
    fn compile_headers_omits_authorization_when_no_token_is_present() {
        let config = AppConfig { github_token: None };

        let headers = compile_github_headers(&config).expect("headers should compile");

        assert!(!headers.contains_key(AUTHORIZATION));
    }

    #[test]
    fn compile_headers_preserves_the_minimum_headers() {
        let with_token =
            compile_github_headers(&config_with_token("ghp_test")).expect("headers should compile");
        let without_token = compile_github_headers(&AppConfig { github_token: None })
            .expect("headers should compile");

        assert_carries_min_headers(&with_token);
        assert_carries_min_headers(&without_token);
    }

    #[test]
    fn compile_headers_does_not_mutate_the_shared_static() {
        let _ =
            compile_github_headers(&config_with_token("ghp_test")).expect("headers should compile");

        // The static is process-wide; a missing clone would leak the token into it.
        assert!(!GITHUB_MIN_HEADERS.contains_key(AUTHORIZATION));
    }

    #[test]
    fn compile_headers_errors_on_a_token_with_invalid_header_bytes() {
        let config = config_with_token("bad\ntoken");

        assert!(compile_github_headers(&config).is_err());
    }

    #[test]
    fn curated_repos_are_non_empty_and_unique() {
        assert!(!CURATED_REPOS.is_empty());
        assert!(CURATED_REPOS.iter().all(|repo| !repo.is_empty()));

        let unique: HashSet<_> = CURATED_REPOS.iter().collect();
        assert_eq!(unique.len(), CURATED_REPOS.len());

        assert!(!GITHUB_USERNAME.is_empty());
        assert!(GITHUB_API_URL.starts_with("https://"));
    }
}
