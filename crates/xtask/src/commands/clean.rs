use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::Path;

pub fn run() -> Result<()> {
    println!("{}", "🧹 Cleaning workspace...".blue().bold());
    println!();

    let mut cleaned = 0;
    let mut skipped = 0;

    // Clean target directory
    if Path::new("target").exists() {
        print!("Removing target/... ");
        match fs::remove_dir_all("target") {
            Ok(_) => {
                println!("{}", "✓ Removed".green());
                cleaned += 1;
            }
            Err(e) => {
                println!("{} {}", "✗ Failed:".red(), e);
                skipped += 1;
            }
        }
    } else {
        println!("target/... {} Already clean", "✓".dimmed());
    }

    // Clean LLM bundles
    if Path::new(".llm/bundle").exists() {
        print!("Removing .llm/bundle/... ");
        match fs::remove_dir_all(".llm/bundle") {
            Ok(_) => {
                println!("{}", "✓ Removed".green());
                cleaned += 1;
            }
            Err(e) => {
                println!("{} {}", "✗ Failed:".red(), e);
                skipped += 1;
            }
        }
    } else {
        println!(".llm/bundle/... {} Already clean", "✓".dimmed());
    }

    // Clean generated docs
    if Path::new("docs/feature_status.md").exists() {
        print!("Removing docs/feature_status.md... ");
        match fs::remove_file("docs/feature_status.md") {
            Ok(_) => {
                println!("{}", "✓ Removed".green());
                cleaned += 1;
            }
            Err(e) => {
                println!("{} {}", "✗ Failed:".red(), e);
                skipped += 1;
            }
        }
    } else {
        println!("docs/feature_status.md... {} Already clean", "✓".dimmed());
    }

    // Clean node_modules if present
    if Path::new("node_modules").exists() {
        print!("Removing node_modules/... ");
        match fs::remove_dir_all("node_modules") {
            Ok(_) => {
                println!("{}", "✓ Removed".green());
                cleaned += 1;
            }
            Err(e) => {
                println!("{} {}", "✗ Failed:".red(), e);
                skipped += 1;
            }
        }
    }

    println!();
    if cleaned > 0 {
        println!("{} Cleaned {} item(s)", "✓".green().bold(), cleaned);
    } else {
        println!("{} Workspace already clean", "✓".green().bold());
    }

    if skipped > 0 {
        println!("{} {} item(s) could not be cleaned", "⚠".yellow(), skipped);
    }

    Ok(())
}
