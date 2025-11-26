use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const SPEC_LEDGER_PATH: &str = "specs/spec_ledger.yaml";
const TEMPLATE_DESCRIPTION: &str = "Rust-as-Spec Platform Cell";

#[derive(Debug, Deserialize)]
struct SpecLedger {
    metadata: LedgerMetadata,
}

#[derive(Debug, Deserialize)]
struct LedgerMetadata {
    template_version: String,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Serialize)]
struct VersionOutput {
    kernel_version: String,
    spec_ledger_path: String,
    description: String,
}

#[derive(Default)]
pub struct VersionArgs {
    pub json: bool,
}

pub fn run(args: VersionArgs) -> Result<()> {
    let ledger_path = Path::new(SPEC_LEDGER_PATH);

    // Read and parse spec_ledger.yaml
    let content = fs::read_to_string(ledger_path)
        .with_context(|| format!("Failed to read spec ledger: {}", ledger_path.display()))?;

    let ledger: SpecLedger = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse spec ledger YAML: {}", ledger_path.display()))?;

    let version = ledger.metadata.template_version;
    let description =
        ledger.metadata.description.unwrap_or_else(|| TEMPLATE_DESCRIPTION.to_string());

    if args.json {
        // Machine-readable JSON output
        let output = VersionOutput {
            kernel_version: version,
            spec_ledger_path: SPEC_LEDGER_PATH.to_string(),
            description,
        };
        let json = serde_json::to_string_pretty(&output)
            .context("Failed to serialize version output to JSON")?;
        println!("{}", json);
    } else {
        // Human-readable output
        println!();
        println!("{}", "Rust-as-Spec Platform Cell".bold());
        println!("{}", "=========================".blue());
        println!();
        println!("  {}: {}", "Kernel version".bold(), version.green());
        println!("  {}: {}", "Spec ledger".bold(), SPEC_LEDGER_PATH.dimmed());
        println!("  {}: {}", "Description".bold(), description.dimmed());
        println!();
    }

    Ok(())
}
