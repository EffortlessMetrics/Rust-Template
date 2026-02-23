#![no_main]

use http_auth::PlatformAuthMode;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let candidate = std::str::from_utf8(data).unwrap_or_default();
    let _ = PlatformAuthMode::parse_strict(candidate);
    let _ = PlatformAuthMode::from(candidate);
});
