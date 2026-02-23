use crate::world::World;
use cucumber::when;
use cucumber::{given, then};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[given(regex = r#"^platform auth mode is "([^"]+)" with token "([^"]+)"$"#)]
async fn given_platform_auth_mode(world: &mut World, mode: String, token: String) {
    // Store auth config in World for isolation between parallel scenarios.
    // reload_app() will apply these settings via env vars just before creating the app.
    world.set_platform_auth(Some(mode), Some(token));
    world.reload_app();
}

#[given(regex = r#"^platform auth mode is "basic" without a token$"#)]
async fn given_basic_auth_without_token(world: &mut World) {
    // Store auth config in World for isolation between parallel scenarios.
    // reload_app() will apply these settings via env vars just before creating the app.
    // Use empty string for token to explicitly indicate "no token" (prevents fallback to config).
    world.set_platform_auth(Some("basic".to_string()), Some(String::new()));
    world.reload_app();
}

#[given(regex = r#"^platform auth mode is "jwt" with secret "([^"]+)"$"#)]
async fn given_jwt_auth_mode(world: &mut World, secret: String) {
    world.set_platform_jwt_auth(Some("jwt".to_string()), Some(secret));
    world.reload_app();
}

#[derive(Serialize)]
struct JwtClaims {
    sub: String,
    exp: u64,
    iat: u64,
    iss: String,
}

#[when(regex = r#"^I set Authorization bearer token signed with secret "([^"]+)"$"#)]
async fn when_set_bearer_token(world: &mut World, secret: String) {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let claims = JwtClaims {
        sub: "bdd-user".to_string(),
        exp: now + 3600,
        iat: now,
        iss: "acceptance-tests".to_string(),
    };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
        .expect("JWT token should encode");

    let value = http::HeaderValue::from_str(&format!("Bearer {}", token))
        .expect("authorization header should be valid");
    world.request_headers.insert(http::header::AUTHORIZATION, value);
}

#[then(regex = r#"^the response body should not contain "([^"]+)"$"#)]
async fn response_body_not_contains(world: &mut World, needle: String) {
    let response = world.last_response.as_ref().expect("response should exist");
    assert!(
        !response.raw_body.contains(&needle),
        "Response body should not contain '{}', but was: {}",
        needle,
        response.raw_body
    );
}
