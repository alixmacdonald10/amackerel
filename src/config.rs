use config::{Config, Environment};
use secrecy::SecretString;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub github_token: SecretString,
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
        try_load_with(app_github_token, github_token).expect("config should load")
    }

    fn try_load_with(
        app_github_token: Option<&str>,
        github_token: Option<&str>,
    ) -> anyhow::Result<AppConfig> {
        temp_env::with_vars(
            [
                ("APP_GITHUB_TOKEN", app_github_token),
                ("GITHUB_TOKEN", github_token),
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
}
