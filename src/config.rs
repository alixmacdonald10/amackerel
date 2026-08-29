use config::{Config, Environment};
use secrecy::SecretString;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub github_token: Option<SecretString>,
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

    /// Loads an [AppConfig] with only the given values in the environment.
    ///
    /// Both vars are always named so a token in the developer's real environment
    /// cannot change the outcome. `temp_env` serialises the closures against each
    /// other and restores the environment afterwards.
    fn load_with(app_github_token: Option<&str>, github_token: Option<&str>) -> AppConfig {
        temp_env::with_vars(
            [
                ("APP_GITHUB_TOKEN", app_github_token),
                ("GITHUB_TOKEN", github_token),
            ],
            || AppConfig::load().expect("config should load"),
        )
    }

    #[test]
    fn load_reads_the_token_from_app_github_token() {
        let config = load_with(Some("ghp_test"), None);

        let token = config.github_token.expect("token should be set");
        assert_eq!(token.expose_secret(), "ghp_test");
    }

    #[test]
    fn load_yields_no_token_when_the_environment_is_empty() {
        let config = load_with(None, None);

        assert!(config.github_token.is_none());
    }

    #[test]
    fn load_ignores_github_token_without_the_app_prefix() {
        let config = load_with(None, Some("ghp_unprefixed"));

        assert!(config.github_token.is_none());
    }

    #[test]
    fn load_prefers_the_prefixed_token_over_the_unprefixed_one() {
        let config = load_with(Some("ghp_prefixed"), Some("ghp_unprefixed"));

        let token = config.github_token.expect("token should be set");
        assert_eq!(token.expose_secret(), "ghp_prefixed");
    }
}
