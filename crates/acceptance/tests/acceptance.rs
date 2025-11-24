use acceptance::World;
use cucumber::{World as _, WriterExt, tag::Ext as _, writer};
use gherkin::tagexpr::TagOperation;
use std::fs::File;

// Platform-specific null device
#[cfg(unix)]
const NULL_DEVICE: &str = "/dev/null";
#[cfg(windows)]
const NULL_DEVICE: &str = "nul";

// Import steps module to ensure step definitions are registered
#[allow(unused_imports)]
use acceptance::steps;

#[tokio::main]
async fn main() {
    // Print a backtrace for any panic so failures in steps are easier to debug.
    std::panic::set_hook(Box::new(|info| {
        eprintln!("panic: {info}");
        eprintln!("{}", std::backtrace::Backtrace::force_capture());
    }));

    // Find the workspace root by going up from CARGO_MANIFEST_DIR
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root =
        std::path::Path::new(manifest_dir).parent().and_then(|p| p.parent()).unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap()
        });

    let features_path = workspace_root.join("specs/features");
    let junit_dir = workspace_root.join("target/junit");
    let junit_path = junit_dir.join("acceptance.xml");

    // Support environment variable for JSON output path (default: target/ac_report.json)
    let json_path = std::env::var("AC_REPORT_JSON")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| workspace_root.join("target/ac_report.json"));

    // Ensure target/junit directory exists
    std::fs::create_dir_all(&junit_dir).unwrap_or(());

    // Ensure JSON parent directory exists
    if let Some(parent) = json_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // Ensure xtask binary is built before running tests (to avoid Windows file locking)
    let xtask_binary = if cfg!(windows) {
        workspace_root.join("target/debug/xtask.exe")
    } else {
        workspace_root.join("target/debug/xtask")
    };

    if !xtask_binary.exists() {
        eprintln!("ERROR: xtask binary not found at {}", xtask_binary.display());
        eprintln!("Please build it first with: cargo build -p xtask");
        eprintln!("\nThis is required to avoid Windows file locking issues during tests.");
        std::process::exit(1);
    }

    // Create output files (fall back to null device if creation fails)
    let junit_file =
        File::create(&junit_path).unwrap_or_else(|_| std::fs::File::create(NULL_DEVICE).unwrap());
    let json_file =
        File::create(&json_path).unwrap_or_else(|_| std::fs::File::create(NULL_DEVICE).unwrap());

    let tag_expression: Option<TagOperation> = std::env::var("CUCUMBER_TAG_EXPRESSION")
        .ok()
        .and_then(|expr| expr.parse::<TagOperation>().ok());

    // Triple output: console + JUnit + JSON
    World::cucumber()
        // Run scenarios sequentially to avoid cross-test interference.
        // The test app uses the global SPEC_ROOT env var, so concurrent runs would race.
        .max_concurrent_scenarios(1)
        .before(|_feature, _rule, _scenario, world| {
            Box::pin(async move {
                *world = World::new();
            })
        })
        .with_writer(
            writer::Basic::stdout().summarized().tee::<World, _>(
                writer::JUnit::new(junit_file, 0)
                    .tee::<World, _>(writer::Json::for_tee(json_file).normalized()),
            ),
        )
        .filter_run(
            features_path.to_str().unwrap_or("specs/features"),
            move |feature, rule, scenario| {
                let tags: Vec<String> = feature
                    .tags
                    .iter()
                    .chain(rule.iter().flat_map(|r| r.tags.iter()))
                    .chain(scenario.tags.iter())
                    .map(|t| t.trim_start_matches('@').to_string())
                    .collect();

                let is_windows_only = tags.iter().any(|t| t == "windows_only");
                let is_unix_only = tags.iter().any(|t| t == "unix_only");

                if cfg!(windows) && is_unix_only {
                    return false;
                }

                if cfg!(unix) && is_windows_only {
                    return false;
                }

                if let Some(expr) = &tag_expression {
                    return expr.eval(tags.iter());
                }

                true
            },
        )
        .await;
}
