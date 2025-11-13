use anyhow::{Context, Result};
use std::process::Command;

/// Generate LLM context bundle for a task
pub fn run(task: &str) -> Result<()> {
    println!("Generating LLM context bundle for task: {}", task);

    let output = Command::new("bash")
        .args(["scripts/make-context.sh", task])
        .output()
        .context("Failed to execute make-context.sh")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("make-context.sh failed:\n{}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    print!("{}", stdout);

    println!("✓ Bundle generated: .llm/bundle/{}.md", task);
    Ok(())
}
