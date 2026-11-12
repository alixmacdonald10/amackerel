use config::{Config, Environment};
use secrecy::SecretString;
use serde::Deserialize;

use crate::cache::default_redis_port;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub github_token: SecretString,
    pub redis_host: String,
    #[serde(default = "default_redis_port")]
    pub redis_port: u16,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        Ok(Config::builder()
            .add_source(Environment::with_prefix("APP"))
            .build()?
            .try_deserialize::<AppConfig>()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use secrecy::ExposeSecret;

    const REDIS_HOST: &str = "redis";

    /// Loads an [AppConfig] with only the given token values in the environment, plus a
    /// redis host so token behaviour is tested in isolation.
    fn load_with(app_github_token: Option<&str>, github_token: Option<&str>) -> AppConfig {
        try_load_with(app_github_token, github_token).expect("config should load")
    }

    fn try_load_with(
        app_github_token: Option<&str>,
        github_token: Option<&str>,
    ) -> anyhow::Result<AppConfig> {
        try_load_with_env(app_github_token, github_token, Some(REDIS_HOST), None)
    }

    /// Every var is always named so values in the developer's real environment
    /// cannot change the outcome. `temp_env` serialises the closures against each
    /// other and restores the environment afterwards.
    fn try_load_with_env(
        app_github_token: Option<&str>,
        github_token: Option<&str>,
        redis_host: Option<&str>,
        redis_port: Option<&str>,
    ) -> anyhow::Result<AppConfig> {
        temp_env::with_vars(
            [
                ("APP_GITHUB_TOKEN", app_github_token),
                ("GITHUB_TOKEN", github_token),
                ("APP_REDIS_HOST", redis_host),
                ("APP_REDIS_PORT", redis_port),
            ],
            AppConfig::load,
        )
    }

    #[test]
    fn load_reads_the_token_from_app_github_token() {
        let config = load_with(Some("ghp_test"), None);

        assert_eq!(config.github_token.expose_secret(), "ghp_test");
    }

    #[test]
    fn load_errors_when_the_environment_is_empty() {
        let result = try_load_with(None, None);

        assert!(result.is_err());
    }

    #[test]
    fn load_errors_when_only_the_unprefixed_token_is_set() {
        let result = try_load_with(None, Some("ghp_unprefixed"));

        assert!(result.is_err());
    }

    #[test]
    fn load_prefers_the_prefixed_token_over_the_unprefixed_one() {
        let config = load_with(Some("ghp_prefixed"), Some("ghp_unprefixed"));

        assert_eq!(config.github_token.expose_secret(), "ghp_prefixed");
    }

    #[test]
    fn load_reads_the_redis_host_and_defaults_the_port() {
        let config = load_with(Some("ghp_test"), None);

        assert_eq!(config.redis_host, REDIS_HOST);
        assert_eq!(config.redis_port, 6379);
    }

    #[test]
    fn load_reads_an_explicit_redis_port() {
        let config = try_load_with_env(Some("ghp_test"), None, Some(REDIS_HOST), Some("6380"))
            .expect("config should load");

        assert_eq!(config.redis_port, 6380);
    }

    #[test]
    fn load_errors_when_the_redis_host_is_missing() {
        let result = try_load_with_env(Some("ghp_test"), None, None, None);

        assert!(result.is_err());
    }
}
