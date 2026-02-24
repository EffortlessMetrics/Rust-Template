use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Verify all tools in bootstrap-tools.sh have checksums
pub fn run() -> Result<()> {
    println!("{} Verifying tool checksum coverage...", "INFO".blue());
    println!();

    // Read bootstrap-tools.sh to find all tools being downloaded
    let bootstrap_script =
        fs::read_to_string("bootstrap-tools.sh").context("Failed to read bootstrap-tools.sh")?;

    // Extract tool names from the script
    // Handle both function definitions (install_foo() {) and calls (install_foo; install_bar)
    let mut installed_tools = HashSet::new();
    for line in bootstrap_script.lines() {
        let trimmed = line.trim();
        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Handle function calls like: install_foo; install_bar; install_baz
        // Also handle function definitions like: install_foo() {
        for segment in trimmed.split(';') {
            let segment = segment.trim();
            if let Some(after_install) = segment.strip_prefix("install_") {
                // Extract just the function name (before any parentheses or whitespace)
                let tool_name = after_install
                    .split(|c: char| c == '(' || c.is_whitespace())
                    .next()
                    .unwrap_or("");
                if !tool_name.is_empty() {
                    installed_tools.insert(tool_name.to_string());
                }
            }
        }
    }

    // Read tools.sha256
    let checksums_path = Path::new("scripts/tools.sha256");
    let checksums_exist = checksums_path.exists();

    if !checksums_exist {
        println!("{} {}", "✗".red(), "scripts/tools.sha256 does not exist".bright_red());
        println!(
            "{} Run {} to create it",
            "  ACTION:".yellow(),
            "cargo xtask tools-checksum-update".bright_yellow()
        );
        return Err(anyhow::anyhow!("Checksum file missing"));
    }

    let checksums_content =
        fs::read_to_string(checksums_path).context("Failed to read scripts/tools.sha256")?;

    // Parse checksums to extract tool names
    let mut checksummed_tools = HashSet::new();
    for line in checksums_content.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let key = parts[0];
            if let Some(tool_name) = key.split('-').next() {
                checksummed_tools.insert(tool_name.to_string());
            }
        }
    }

    // Check for missing checksums
    let mut missing = Vec::new();
    for tool in &installed_tools {
        if !checksummed_tools.contains(tool) {
            missing.push(tool.clone());
        }
    }

    // Check for extra checksums (tools that have checksums but aren't installed)
    let mut extra = Vec::new();
    for tool in &checksummed_tools {
        if !installed_tools.contains(tool) {
            extra.push(tool.clone());
        }
    }

    // Report results
    if missing.is_empty() && extra.is_empty() {
        println!("{} All installed tools have checksums", "✓".green());
    } else {
        if !missing.is_empty() {
            println!("{} Missing checksums for: {}", "✗".red(), missing.join(", ").bright_red());
            println!(
                "{} Run {} to add them",
                "  ACTION:".yellow(),
                "cargo xtask tools-checksum-update".bright_yellow()
            );
        }
        if !extra.is_empty() {
            println!("{} Extra checksums for: {}", "⚠".yellow(), extra.join(", ").yellow());
            println!("  These tools have checksums but aren't installed by bootstrap-tools.sh");
        }

        if !missing.is_empty() {
            return Err(anyhow::anyhow!("Missing checksums for tools"));
        }
    }

    // Verify checksums are up-to-date by checking if they match current downloads
    println!();
    println!("{} Verifying checksums match current releases...", "INFO".blue());

    // For each checksum entry, try to download and verify
    let mut verification_errors = Vec::new();
    for line in checksums_content.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let key = parts[0];
        let expected_checksum = parts[1];

        // Extract tool info from key
        let key_parts: Vec<&str> = key.split('-').collect();
        if key_parts.len() < 4 {
            continue;
        }

        let tool_name = key_parts[0];
        let version = key_parts[1];
        let os = key_parts[2];
        let arch = key_parts[3];

        // Skip verification for "latest" version as it's dynamic
        if version == "latest" {
            println!("  {} Skipping {} (dynamic latest version)", "→".dimmed(), key.cyan());
            continue;
        }

        print!("  {} {}: ", "→".dimmed(), key.cyan());

        // Get the download URL based on tool and platform
        let url = match get_download_url(tool_name, version, os, arch) {
            Ok(u) => u,
            Err(e) => {
                println!("{} {}", "FAIL".red(), e);
                verification_errors.push(format!("{}: {}", key, e));
                continue;
            }
        };

        // Download and verify
        match verify_checksum(&url, expected_checksum) {
            Ok(_) => println!("{}", "OK".green()),
            Err(e) => {
                println!("{} {}", "FAIL".red(), e);
                verification_errors.push(format!("{}: {}", key, e));
            }
        }
    }

    if !verification_errors.is_empty() {
        println!();
        println!("{} Some checksums are out of date", "✗".red());
        println!(
            "{} Run {} to update them",
            "  ACTION:".yellow(),
            "cargo xtask tools-checksum-update".bright_yellow()
        );
        return Err(anyhow::anyhow!("Checksum verification failed"));
    }

    println!();
    println!("{} All checksums are valid and up-to-date!", "✓".green());
    Ok(())
}

/// Get download URL for a tool based on its info
fn get_download_url(tool_name: &str, version: &str, os: &str, arch: &str) -> Result<String> {
    match tool_name {
        "oasdiff" => {
            let oas_arch = if os == "darwin" { "all" } else { arch };
            Ok(format!(
                "https://github.com/oasdiff/oasdiff/releases/download/v{}/oasdiff_{}_{}_{}.tar.gz",
                version, version, os, oas_arch
            ))
        }
        "buf" => {
            let os_cap = format!("{}{}", &os[..1].to_uppercase(), &os[1..]);
            let buf_arch = match (os, arch) {
                (_, "amd64") => "x86_64",
                ("linux", "arm64") => "aarch64",
                ("darwin", "arm64") => "arm64",
                _ => arch,
            };
            let ext = if os == "windows" { ".exe" } else { "" };
            Ok(format!(
                "https://github.com/bufbuild/buf/releases/download/v{}/buf-{}-{}{}",
                version, os_cap, buf_arch, ext
            ))
        }
        "atlas" => {
            let ext = if os == "windows" { ".exe" } else { "" };
            Ok(format!("https://release.ariga.io/atlas/atlas-{}-{}-{}{}", os, arch, version, ext))
        }
        "gitleaks" => {
            let ext = if os == "windows" { "zip" } else { "tar.gz" };
            let platform_arch = match (os, arch) {
                ("linux", "amd64") => "linux_x64",
                ("darwin", "amd64") => "darwin_x64",
                ("darwin", "arm64") => "darwin_arm64",
                ("windows", "amd64") => "windows_x64",
                _ => return Err(anyhow::anyhow!("Unsupported platform: {}-{}", os, arch)),
            };
            Ok(format!(
                "https://github.com/gitleaks/gitleaks/releases/download/v{}/gitleaks_{}_{}.{}",
                version, version, platform_arch, ext
            ))
        }
        _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
    }
}

/// Verify that a URL has the expected checksum
fn verify_checksum(url: &str, expected_checksum: &str) -> Result<()> {
    use std::process::Command;

    // Download to temporary file
    let temp_file = format!("/tmp/verify_checksum_{}", std::process::id());

    let output = Command::new("curl")
        .args(["-sSfL", url, "-o", &temp_file])
        .output()
        .context("Failed to download tool")?;

    if !output.status.success() {
        let _ = std::fs::remove_file(&temp_file);
        return Err(anyhow::anyhow!(
            "Download failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Calculate checksum
    let sha_output =
        Command::new("sha256sum").arg(&temp_file).output().context("Failed to compute SHA256")?;

    let _ = std::fs::remove_file(&temp_file);

    if !sha_output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to compute SHA256: {}",
            String::from_utf8_lossy(&sha_output.stderr)
        ));
    }

    let sha_str = String::from_utf8_lossy(&sha_output.stdout);
    let actual_checksum = sha_str.split_whitespace().next().context("Invalid sha256sum output")?;

    if actual_checksum != expected_checksum {
        return Err(anyhow::anyhow!(
            "Checksum mismatch: expected {}, got {}",
            expected_checksum,
            actual_checksum
        ));
    }

    Ok(())
}
