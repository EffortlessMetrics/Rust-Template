#![no_main]

use arbitrary::Arbitrary;
use http_body_format::classify_body_format;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct Input {
    content_type: Option<String>,
}

fuzz_target!(|input: Input| {
    let _ = classify_body_format(input.content_type.as_deref());
});
