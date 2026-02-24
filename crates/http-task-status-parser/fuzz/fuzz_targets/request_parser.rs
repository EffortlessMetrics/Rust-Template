#![no_main]

use arbitrary::Arbitrary;
use http_task_status_parser::parse_update_task_status;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct Input {
    content_type: Option<String>,
    body: Vec<u8>,
}

fuzz_target!(|input: Input| {
    let _ = parse_update_task_status(input.content_type.as_deref(), &input.body);
});
