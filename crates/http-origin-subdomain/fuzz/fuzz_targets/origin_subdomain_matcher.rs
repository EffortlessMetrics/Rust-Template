#![no_main]

use arbitrary::Arbitrary;
use http_origin_subdomain::matches_subdomain_wildcard_rule;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct Input {
    allowed: String,
    origin: String,
}

fuzz_target!(|input: Input| {
    let _ = matches_subdomain_wildcard_rule(input.allowed.as_str(), input.origin.as_str());
});
