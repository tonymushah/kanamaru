use bsky_sdk::agent::config::Config as BskyAgentConfig;
use duration_string::DurationString;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ReqwestClientConfig {
    pub rate_limit_number: Option<u64>,
    pub rate_limit_duration: Option<DurationString>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub bsk_config: BskyAgentConfig,
    #[serde(default)]
    pub reqwest: ReqwestClientConfig,
}
