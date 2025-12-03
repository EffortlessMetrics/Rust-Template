use crate::world::World;
use cucumber::{given, then};

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
