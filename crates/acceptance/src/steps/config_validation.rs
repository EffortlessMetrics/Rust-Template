use crate::World;
use cucumber::{gherkin::Step, given, then, when};
use std::path::Path;

#[given(regex = r#"^the config file "([^"]+)" contains:$"#)]
async fn given_config_file(_world: &mut World, rel_path: String, step: &Step) {
    let content = step
        .docstring
        .as_ref()
        .cloned()
        .unwrap_or_else(|| panic!("Config content must be provided for {}", rel_path));

    let root = std::env::var("SPEC_ROOT").unwrap_or_else(|_| ".".to_string());
    let target = Path::new(&root).join(rel_path);

    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create parent directory for config");
    }

    std::fs::write(&target, content).expect("Failed to write config file");
}

#[when("I validate the configuration against the schema")]
async fn when_validate_config(world: &mut World) {
    let root = std::env::var("SPEC_ROOT").unwrap_or_else(|_| ".".to_string());
    let schema_path = Path::new(&root).join("specs/config_schema.yaml");
    let config_path = Path::new(&root).join("config/local.yaml");

    match spec_runtime::validate_config(&schema_path, &config_path) {
        Ok(_) => {
            world.config_validation_ok = Some(true);
            world.config_validation_error = None;
        }
        Err(err) => {
            world.config_validation_ok = Some(false);
            world.config_validation_error = Some(err.to_string());
        }
    }
}

#[then("the configuration validation should fail")]
async fn then_validation_should_fail(world: &mut World) {
    let Some(ok) = world.config_validation_ok else {
        panic!("Configuration validation was not executed");
    };

    assert!(
        !ok,
        "Expected configuration validation to fail, but it succeeded. Error: {:?}",
        world.config_validation_error
    );
}

#[then(regex = r#"^the validation error should contain "([^"]+)"$"#)]
async fn then_validation_error_contains(world: &mut World, needle: String) {
    let error = world
        .config_validation_error
        .as_ref()
        .unwrap_or_else(|| panic!("Expected validation error message to be captured"));

    assert!(
        error.contains(&needle),
        "Expected validation error to contain '{}', but got: {}",
        needle,
        error
    );
}
