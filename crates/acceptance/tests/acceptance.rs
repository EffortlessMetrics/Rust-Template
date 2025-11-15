use acceptance::World;
use cucumber::{World as _, WriterExt, writer};
use std::fs::File;

// Import steps module to ensure step definitions are registered
#[allow(unused_imports)]
use acceptance::steps;

#[tokio::main]
async fn main() {
    // Find the workspace root by going up from CARGO_MANIFEST_DIR
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = std::path::Path::new(manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("Failed to find workspace root");

    let features_path = workspace_root.join("specs/features");
    let junit_dir = workspace_root.join("target/junit");
    let junit_path = junit_dir.join("acceptance.xml");

    // Support environment variable for JSON output path (default: target/ac_report.json)
    let json_path = std::env::var("AC_REPORT_JSON")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| workspace_root.join("target/ac_report.json"));

    // Ensure target/junit directory exists
    std::fs::create_dir_all(&junit_dir).expect("Failed to create junit directory");

    // Ensure JSON parent directory exists
    if let Some(parent) = json_path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create JSON output directory");
    }

    // Create output files
    let junit_file = File::create(&junit_path).expect("Failed to create JUnit output file");
    let json_file = File::create(&json_path).expect("Failed to create JSON output file");

    // Initialize service availability at test run start
    // (guards against state bleeding from previous test runs)
    core::set_service_available(true);

    // Triple output: console + JUnit + JSON
    World::cucumber()
        .filter_run("not @skip", |_, _, sc| !sc.tags.iter().any(|t| t == "skip"))
        .with_writer(
            writer::Basic::stdout().summarized().tee::<World, _>(
                writer::JUnit::new(junit_file, 0)
                    .tee::<World, _>(writer::Json::for_tee(json_file).normalized()),
            ),
        )
        .before(|_feature, _rule, _scenario, world| {
            Box::pin(async move {
                *world = World::new();
                // Reset service availability before each scenario
                core::set_service_available(true);
            })
        })
        .after(|_feature, _rule, _scenario, _event, _world| {
            Box::pin(async move {
                // Ensure service is available after each scenario (cleanup)
                core::set_service_available(true);
            })
        })
        .run(features_path.to_str().unwrap())
        .await;
}
