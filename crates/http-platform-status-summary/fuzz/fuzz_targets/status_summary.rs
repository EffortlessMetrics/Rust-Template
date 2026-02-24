#![no_main]

use arbitrary::Arbitrary;
use http_platform_status_summary::ConfigSummary;
use libfuzzer_sys::fuzz_target;
use serde_yaml::Value as YamlValue;
use spec_runtime::ValidatedConfig;
use std::collections::HashMap;

#[derive(Arbitrary, Debug)]
struct Input {
    http_port: u16,
    env: Option<String>,
    settings: Vec<(String, String)>,
    secrets: Vec<(String, String)>,
    mode: String,
    token_present: bool,
}

fuzz_target!(|input: Input| {
    let settings = input
        .settings
        .into_iter()
        .filter_map(|(k, v)| {
            if k.is_empty() {
                None
            } else {
                Some((k, YamlValue::String(v)))
            }
        })
        .collect::<HashMap<String, YamlValue>>();

    let secrets = input
        .secrets
        .into_iter()
        .filter_map(|(k, v)| if k.is_empty() { None } else { Some((k, v)) })
        .collect::<HashMap<String, String>>();

    let config = ValidatedConfig {
        env: input.env.filter(|env| !env.is_empty()),
        http_port: input.http_port,
        settings,
        secrets,
    };

    let _ = ConfigSummary::from_parts(&config, input.mode.as_str(), input.token_present);
});
