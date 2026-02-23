#![no_main]

use arbitrary::Arbitrary;
use http::{HeaderMap, HeaderValue};
use http_auth_token::{
    AUTHORIZATION_HEADER, PLATFORM_AUTH_HEADER, extract_auth_token, extract_auth_token_from_headers,
};
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct Input {
    include_authorization: bool,
    include_platform: bool,
    authorization_bytes: Vec<u8>,
    platform_bytes: Vec<u8>,
    authorization_text: Option<String>,
    platform_text: Option<String>,
}

fuzz_target!(|input: Input| {
    let _ = extract_auth_token(input.authorization_text.as_deref(), input.platform_text.as_deref());

    let mut headers = HeaderMap::new();

    if input.include_authorization {
        if let Ok(value) = HeaderValue::from_bytes(&input.authorization_bytes) {
            headers.insert(AUTHORIZATION_HEADER, value);
        }
    }

    if input.include_platform {
        if let Ok(value) = HeaderValue::from_bytes(&input.platform_bytes) {
            headers.insert(PLATFORM_AUTH_HEADER, value);
        }
    }

    let _ = extract_auth_token_from_headers(&headers);
});
