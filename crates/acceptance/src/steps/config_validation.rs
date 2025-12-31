use crate::World;
use cucumber::{gherkin::Step, given, then, when};

#[given(regex = r#"^the config file "([^"]+)" contains:$"#)]
async fn given_config_file(world: &mut World, rel_path: String, step: &Step) {
    let content = step.docstring.as_ref().cloned();
    assert!(content.is_some(), "Config content must be provided for {}", rel_path);
    let content = content.unwrap();

    let root = world.spec_root();
    let target = root.join(rel_path);

    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create parent directory for config");
    }

    std::fs::write(&target, content).expect("Failed to write config file");
}

#[when("I validate the configuration against the schema")]
async fn when_validate_config(world: &mut World) {
    let root = world.spec_root();
    let schema_path = root.join("specs/config_schema.yaml");
    let config_path = root.join("config/local.yaml");

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
    assert!(world.config_validation_ok.is_some(), "Configuration validation was not executed");
    let ok = world.config_validation_ok.unwrap();

    assert!(
        !ok,
        "Expected configuration validation to fail, but it succeeded. Error: {:?}",
        world.config_validation_error
    );
}

#[then(regex = r#"^the validation error should contain "([^"]+)"$"#)]
async fn then_validation_error_contains(world: &mut World, needle: String) {
    let error = world.config_validation_error.as_ref();
    assert!(error.is_some(), "Expected validation error message to be captured");
    let error = error.unwrap();

    assert!(
        error.contains(&needle),
        "Expected validation error to contain '{}', but got: {}",
        needle,
        error
    );
}
