#![no_main]

use arbitrary::Arbitrary;
use http_auth_verifier::{authorize_token, constant_time_eq, validate_jwt_token};
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct Input {
    provided: Option<String>,
    basic_token: Option<String>,
    jwt_secret: Option<String>,
    left: String,
    right: String,
}

fuzz_target!(|input: Input| {
    let _ = authorize_token(
        input.provided.as_deref(),
        input.basic_token.as_deref(),
        input.jwt_secret.as_deref(),
    );

    if let (Some(token), Some(secret)) = (input.provided.as_deref(), input.jwt_secret.as_deref()) {
        let _ = validate_jwt_token(token, secret);
    }

    let _ = constant_time_eq(input.left.as_str(), input.right.as_str());
});
