use anyhow::Result;
use colored::Colorize;

use crate::commands::tasks;

pub fn run() -> Result<()> {
    let root = tasks::spec_root();
    let tasks_spec = spec_runtime::load_tasks(&root.join("specs/tasks.yaml"))?;

    println!("{}", "Tasks (from specs/tasks.yaml)".blue().bold());
    println!(
        "{:<30} {:<12} {:<20} {:<30} {:<12} Title",
        "ID", "Status", "Requirement", "ACs", "Owner"
    );
    println!("{}", "-".repeat(140));

    for task in tasks_spec.tasks {
        let acs_display = if task.acs.is_empty() {
            "-".to_string()
        } else if task.acs.len() <= 2 {
            task.acs.join(", ")
        } else {
            format!("{}, +{} more", task.acs[0], task.acs.len() - 1)
        };

        println!(
            "{:<30} {:<12} {:<20} {:<30} {:<12} {}",
            task.id,
            task.status,
            task.requirement,
            acs_display,
            task.owner.unwrap_or_else(|| "-".to_string()),
            task.title
        );
    }

    println!("\nLegend:");
    println!("  Status: Todo | InProgress | Review | Done");
    println!("\nFor details: cargo xtask tasks-list --help");

    Ok(())
}
