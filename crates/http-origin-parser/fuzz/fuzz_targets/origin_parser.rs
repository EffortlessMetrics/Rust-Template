#![no_main]

use arbitrary::Arbitrary;
use http_origin_parser::parse_http_origin;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct Input {
    value: String,
}

fuzz_target!(|input: Input| {
    let _ = parse_http_origin(input.value.as_str());
});
