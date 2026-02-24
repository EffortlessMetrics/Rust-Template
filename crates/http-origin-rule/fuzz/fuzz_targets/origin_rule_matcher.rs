#![no_main]

use arbitrary::Arbitrary;
use http_origin_rule::is_origin_allowed_by_rule;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct Input {
    allowed: String,
    origin: String,
}

fuzz_target!(|input: Input| {
    let _ = is_origin_allowed_by_rule(input.allowed.as_str(), input.origin.as_str());
});
