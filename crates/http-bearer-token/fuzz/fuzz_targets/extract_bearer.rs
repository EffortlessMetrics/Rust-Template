#![no_main]

use arbitrary::Arbitrary;
use http_bearer_token::extract_bearer_token;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct Input {
    bytes: Vec<u8>,
}

fuzz_target!(|input: Input| {
    if let Ok(value) = std::str::from_utf8(&input.bytes) {
        let _ = extract_bearer_token(value);
    }
});
