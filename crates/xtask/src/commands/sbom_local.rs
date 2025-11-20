use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn run() -> Result<()> {
    println!("{}", "📋 Generating local SBOM...".blue().bold());
    println!();

    // Create target directory if it doesn't exist
    fs::create_dir_all("target")?;

    // Generate SBOM using cargo tree
    print!("Generating SBOM from dependency tree... ");

    let output = Command::new("cargo").args(["tree", "--format", "{p} {l}"]).output()?;

    if !output.status.success() {
        println!("{}", "✗ Failed".red());
        anyhow::bail!("cargo tree command failed");
    }

    // Create a simple SPDX-like JSON structure
    let tree_output = String::from_utf8_lossy(&output.stdout);

    let sbom = serde_json::json!({
        "spdxVersion": "SPDX-2.3",
        "dataLicense": "CC0-1.0",
        "SPDXID": "SPDXRef-DOCUMENT",
        "name": "Rust-Template-SBOM",
        "documentNamespace": format!("https://github.com/EffortlessMetrics/Rust-Template/sbom-{}",
            chrono::Local::now().format("%Y%m%d%H%M%S")),
        "creationInfo": {
            "created": chrono::Local::now().to_rfc3339(),
            "creators": ["Tool: cargo-tree via xtask"]
        },
        "packages": tree_output.lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    Some(serde_json::json!({
                        "name": parts[0],
                        "SPDXID": format!("SPDXRef-{}", parts[0].replace(['/', '-', '.'], "_")),
                        "versionInfo": parts.get(1).unwrap_or(&"unknown"),
                        "licenseConcluded": parts.get(2).unwrap_or(&"NOASSERTION")
                    }))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    });

    // Write SBOM
    let sbom_path = Path::new("target/sbom.spdx.json");
    fs::write(sbom_path, serde_json::to_string_pretty(&sbom)?)?;

    println!("{}", "✓ Done".green());
    println!();
    println!("{} SBOM written to: {}", "📄".cyan(), sbom_path.display());
    println!();
    println!("{}", "Note:".bold());
    println!("  This is a simplified local SBOM for development.");
    println!("  For CI-grade attestations & provenance, see:");
    println!("  {}", "docs/explanation/supply-chain-hardening.md".cyan());
    println!();
    println!("  CI generates official SBOMs on tagged releases via:");
    println!("  {}", ".github/workflows/ci-supply-chain.yml".dimmed());

    Ok(())
}
