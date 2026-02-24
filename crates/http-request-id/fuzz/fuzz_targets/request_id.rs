#![no_main]

use arbitrary::Arbitrary;
use axum::http::{HeaderMap, HeaderValue};
use http_request_id::{extract_request_id, REQUEST_ID_HEADER};
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct Input {
    request_id: Option<String>,
}

fuzz_target!(|input: Input| {
    let mut headers = HeaderMap::new();

    if let Some(value) = input.request_id {
        if let Ok(header) = HeaderValue::from_str(&value) {
            headers.insert(REQUEST_ID_HEADER, header);
        }
    }

    let request_id = extract_request_id(&headers);
    let _ = request_id.as_str();
    let _ = request_id.to_string();
});
