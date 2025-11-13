use acceptance::World;
use cucumber::{writer, World as _, WriterExt};
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

    // Ensure target/junit directory exists
    std::fs::create_dir_all(&junit_dir).expect("Failed to create junit directory");

    // Create the JUnit output file
    let junit_file = File::create(&junit_path).expect("Failed to create JUnit output file");

    // Combine basic console output with JUnit XML output
    World::cucumber()
        .with_writer(
            writer::Basic::stdout().summarized().tee::<World, _>(writer::JUnit::new(junit_file, 0)),
        )
        .before(|_feature, _rule, _scenario, world| {
            Box::pin(async move {
                *world = World::new();
            })
        })
        .run(features_path.to_str().unwrap())
        .await;
}
