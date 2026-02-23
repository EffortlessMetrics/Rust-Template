#![no_main]

use arbitrary::Arbitrary;
use http_origin_boundary::{has_label_boundary_before_suffix, has_path_boundary_or_empty};
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct Input {
    value: String,
    suffix: String,
    remainder: String,
}

fuzz_target!(|input: Input| {
    let _ = has_label_boundary_before_suffix(input.value.as_str(), input.suffix.as_str());
    let _ = has_path_boundary_or_empty(input.remainder.as_str());
});
