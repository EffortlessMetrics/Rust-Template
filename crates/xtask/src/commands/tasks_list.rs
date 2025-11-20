use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

pub fn run() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().unwrap().parent().unwrap();

    let tasks_spec = spec_runtime::load_tasks(&root.join("specs/tasks.yaml"))?;

    println!("{}\n", "📋 Tasks (from specs/tasks.yaml)".blue().bold());
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
