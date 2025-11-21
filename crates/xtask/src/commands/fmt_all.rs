use crate::run_cmd;
use anyhow::Result;
use colored::Colorize;
use std::process::Command;

pub fn run() -> Result<()> {
    println!("{}", "🎨 Formatting all code...".blue().bold());
    println!();

    let mut formatted = 0;

    // Format Rust code
    print!("Formatting Rust (cargo fmt --all)... ");
    print!("Formatting Rust (cargo fmt --all)... ");
    let mut cmd = crate::cargo_cmd("fmt", &["--all"]);
    match run_cmd(&mut cmd) {
        Ok(_) => {
            println!("{}", "✓ Done".green());
            formatted += 1;
        }
        Err(e) => {
            println!("{} {}", "✗ Failed:".red(), e);
        }
    }

    // Format YAML files (if yq is available)
    if which::which("yq").is_ok() {
        print!("Checking YAML formatting... ");
        // Note: yq can validate but auto-formatting YAML is risky (loses comments)
        // So we just validate here
        let yaml_files = ["specs/spec_ledger.yaml", ".llm/contextpack.yaml"];
        let mut valid = true;
        for file in &yaml_files {
            if std::path::Path::new(file).exists() {
                let mut cmd = Command::new("yq");
                cmd.args(["eval", ".", file]);
                cmd.stdout(std::process::Stdio::null());
                if cmd.status().is_err() {
                    valid = false;
                    break;
                }
            }
        }
        if valid {
            println!("{} Valid", "✓".green());
        } else {
            println!("{} Issues found", "⚠".yellow());
        }
    }

    // Format JSON files (if jq is available)
    if which::which("jq").is_ok() {
        print!("Formatting JSON (if any)... ");
        // This would format any .json files in the repo
        // For now, just validate
        println!("{} Skipped (manual)", "✓".dimmed());
    }

    println!();
    if formatted > 0 {
        println!("{} Formatting complete", "✓".green().bold());
    } else {
        println!("{} No formatting applied", "⚠".yellow());
    }

    Ok(())
}
