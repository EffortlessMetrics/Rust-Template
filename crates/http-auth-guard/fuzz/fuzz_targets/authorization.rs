#![no_main]

use arbitrary::Arbitrary;
use http::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use http::Method;
use http_auth_guard::{is_request_authorized_for_platform, build_auth_config};
use http_auth_policy::PlatformAuthMode;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct Input {
    method: u8,
    mode: u8,
    token: Option<String>,
    jwt_secret: Option<String>,
    bearer_token: Option<String>,
}

fuzz_target!(|input: Input| {
    let method = match input.method % 4 {
        0 => Method::OPTIONS,
        1 => Method::GET,
        2 => Method::POST,
        _ => Method::DELETE,
    };

    let mode = match input.mode % 3 {
        0 => PlatformAuthMode::Open,
        1 => PlatformAuthMode::Basic,
        _ => PlatformAuthMode::Jwt,
    };

    let config = build_auth_config(mode, input.token.as_deref(), input.jwt_secret.as_deref());
    let mut headers = HeaderMap::new();

    if let Some(token) = input.bearer_token {
        if let Ok(value) = HeaderValue::from_str(&format!("Bearer {token}")) {
            headers.insert(AUTHORIZATION, value);
        }
    }

    let _ = is_request_authorized_for_platform(&method, &headers, &config);
});
