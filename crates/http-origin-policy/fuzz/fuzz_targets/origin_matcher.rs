#![no_main]

use arbitrary::Arbitrary;
use http_origin_policy::{is_origin_allowed, is_origin_allowed_by_any};
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct Input {
    allowed: String,
    origin: String,
    allow_a: String,
    allow_b: String,
}

fuzz_target!(|input: Input| {
    let _ = is_origin_allowed(input.allowed.as_str(), input.origin.as_str());

    let allowed = vec![input.allow_a, input.allow_b];
    let _ = is_origin_allowed_by_any(&allowed, input.origin.as_str());
});
