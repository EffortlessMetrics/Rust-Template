use crate::{PlatformAuthConfig, PlatformAuthMode};
use spec_runtime::ValidatedConfig;

const DEFAULT_AUTH_MODE: &str = "open";
const ENV_AUTH_MODE: &str = "PLATFORM_AUTH_MODE";
const ENV_AUTH_TOKEN: &str = "PLATFORM_AUTH_TOKEN";
const ENV_JWT_SECRET: &str = "PLATFORM_JWT_SECRET";
const CONFIG_AUTH_MODE: &str = "platform.auth_mode";
const CONFIG_AUTH_TOKEN: &str = "platform.auth_token";
const CONFIG_JWT_SECRET: &str = "platform.jwt_secret";

impl PlatformAuthConfig {
    /// Build auth config from env vars and optional validated config.
    ///
    /// Precedence is environment-first, then config.
    ///
    /// Fails closed when auth mode is invalid.
    pub fn try_from_sources(config: Option<&ValidatedConfig>) -> Result<Self, String> {
        let mode_raw = env_or_config_setting(ENV_AUTH_MODE, config, CONFIG_AUTH_MODE)
            .unwrap_or_else(|| DEFAULT_AUTH_MODE.to_string());
        let mode = PlatformAuthMode::parse_strict(&mode_raw)?;
        let token = env_or_config_secret(ENV_AUTH_TOKEN, config, CONFIG_AUTH_TOKEN);
        let jwt_secret = env_or_config_secret(ENV_JWT_SECRET, config, CONFIG_JWT_SECRET);

        Ok(Self { mode, token, jwt_secret })
    }
}

fn env_or_config_setting(
    env_key: &str,
    config: Option<&ValidatedConfig>,
    config_key: &str,
) -> Option<String> {
    std::env::var(env_key).ok().or_else(|| {
        config
            .and_then(|cfg| cfg.settings.get(config_key))
            .and_then(|value| value.as_str())
            .map(ToString::to_string)
    })
}

fn env_or_config_secret(
    env_key: &str,
    config: Option<&ValidatedConfig>,
    config_key: &str,
) -> Option<String> {
    std::env::var(env_key)
        .ok()
        .or_else(|| config.and_then(|cfg| cfg.secrets.get(config_key).cloned()))
}
