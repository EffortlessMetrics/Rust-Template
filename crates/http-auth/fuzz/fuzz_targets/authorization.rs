#![no_main]

use arbitrary::Arbitrary;
use http_auth::{PlatformAuthConfig, PlatformAuthMode};
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct Input {
    mode: u8,
    token: Option<String>,
    jwt_secret: Option<String>,
    provided: Option<String>,
}

fuzz_target!(|input: Input| {
    let mode = match input.mode % 3 {
        0 => PlatformAuthMode::Open,
        1 => PlatformAuthMode::Basic,
        _ => PlatformAuthMode::Jwt,
    };

    let cfg = PlatformAuthConfig {
        mode,
        token: input.token,
        jwt_secret: input.jwt_secret,
    };

    let _ = cfg.requires_auth();
    let _ = cfg.can_enforce_auth();
    let _ = cfg.mode_label();
    let _ = cfg.token_present();
    let _ = cfg.warn_if_misconfigured();
    let _ = cfg.is_authorized(input.provided.as_deref());
});
