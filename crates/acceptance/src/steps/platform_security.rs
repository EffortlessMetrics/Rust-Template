use crate::world::World;
use cucumber::{given, then};

#[given(regex = r#"^platform auth mode is "([^"]+)" with token "([^"]+)"$"#)]
async fn given_platform_auth_mode(world: &mut World, mode: String, token: String) {
    // SAFETY: Tests mutate process env in a single-threaded runner to configure the app instance.
    unsafe {
        std::env::set_var("PLATFORM_AUTH_MODE", &mode);
        std::env::set_var("PLATFORM_AUTH_TOKEN", &token);
    }
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
