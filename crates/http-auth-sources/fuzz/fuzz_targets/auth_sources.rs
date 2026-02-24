#![no_main]

use arbitrary::Arbitrary;
use http_auth_sources::resolve_auth_sources;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct Input {
    mode_env: Option<String>,
    token_env: Option<String>,
    jwt_env: Option<String>,
    mode_cfg: Option<String>,
    token_cfg: Option<String>,
    jwt_cfg: Option<String>,
}

fuzz_target!(|input: Input| {
    let _ = resolve_auth_sources(
        input.mode_env.as_deref(),
        input.token_env.as_deref(),
        input.jwt_env.as_deref(),
        input.mode_cfg.as_deref(),
        input.token_cfg.as_deref(),
        input.jwt_cfg.as_deref(),
    );
});
