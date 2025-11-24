use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

pub fn run() -> Result<()> {
    let root = spec_root();
    let tasks_spec = spec_runtime::load_tasks(&root.join("specs/tasks.yaml"))?;

    println!("{}", "Tasks (from specs/tasks.yaml)".blue().bold());
    println!("{:<30} {:<15} {:<15} Title", "ID", "Status", "Owner");
    println!("{}", "-".repeat(100));

    for task in tasks_spec.tasks {
        println!(
            "{:<30} {:<15} {:<15} {}",
            task.id,
            task.status,
            task.owner.unwrap_or_default(),
            task.title
        );
    }

    println!("\nLegend:");
    println!("  status: open | in_progress | done | blocked");
    println!("\nFor details: cargo xtask tasks-list --help");

    Ok(())
}

fn spec_root() -> PathBuf {
    if let Ok(root) = std::env::var("SPEC_ROOT") {
        return PathBuf::from(root);
    }

    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf()
}
