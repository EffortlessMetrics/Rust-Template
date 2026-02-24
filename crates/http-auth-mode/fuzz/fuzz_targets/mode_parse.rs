#![no_main]

use http_auth_mode::PlatformAuthMode;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let candidate = std::str::from_utf8(data).unwrap_or_default();
    let mode = PlatformAuthMode::from(candidate);
    let _ = mode.label();
    let _ = PlatformAuthMode::parse_strict(candidate);
});
