#![no_main]

use arbitrary::Arbitrary;
use http_auth_config_inputs::PlatformAuthConfigInputs;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct Input {
    mode_env: Option<String>,
    token_env: Option<String>,
    jwt_secret_env: Option<String>,
    mode_config: Option<String>,
    token_config: Option<String>,
    jwt_secret_config: Option<String>,
}

fuzz_target!(|input: Input| {
    let inputs = PlatformAuthConfigInputs {
        mode_env: input.mode_env,
        token_env: input.token_env,
        jwt_secret_env: input.jwt_secret_env,
        mode_config: input.mode_config,
        token_config: input.token_config,
        jwt_secret_config: input.jwt_secret_config,
    };

    let _ = inputs.mode();
    let _ = inputs.token();
    let _ = inputs.jwt_secret();
});
