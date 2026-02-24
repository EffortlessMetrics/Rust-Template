use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

/// Tool definition for checksum generation
#[derive(Debug, Clone)]
struct Tool {
    name: String,
    version: String,
    platforms: Vec<Platform>,
}

#[derive(Debug, Clone)]
struct Platform {
    os: String,
    arch: String,
    url: String,
}

/// Generate SHA256 checksum for a URL
fn get_sha256_for_url(url: &str) -> Result<String> {
    // Download to temporary file first
    let temp_file = format!("/tmp/tool_checksum_{}", std::process::id());

    let output = Command::new("curl")
        .args(["-sSfL", url, "-o", &temp_file])
        .output()
        .context("Failed to download tool")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to download from {}: {}",
            url,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let sha_output =
        Command::new("sha256sum").arg(&temp_file).output().context("Failed to compute SHA256")?;

    if !sha_output.status.success() {
        anyhow::bail!("Failed to compute SHA256: {}", String::from_utf8_lossy(&sha_output.stderr));
    }

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_file);

    let sha_str = String::from_utf8_lossy(&sha_output.stdout);
    let checksum = sha_str.split_whitespace().next().context("Invalid sha256sum output")?;
    Ok(checksum.to_string())
}

/// Generate checksums for all tools
fn generate_checksums() -> Result<Vec<(String, String)>> {
    let tools = vec![
        Tool {
            name: "oasdiff".to_string(),
            version: "1.11.7".to_string(),
            platforms: vec![
                Platform {
                    os: "linux".to_string(),
                    arch: "amd64".to_string(),
                    url: "https://github.com/oasdiff/oasdiff/releases/download/v1.11.7/oasdiff_1.11.7_linux_amd64.tar.gz".to_string(),
                },
                Platform {
                    os: "linux".to_string(),
                    arch: "arm64".to_string(),
                    url: "https://github.com/oasdiff/oasdiff/releases/download/v1.11.7/oasdiff_1.11.7_linux_arm64.tar.gz".to_string(),
                },
                Platform {
                    os: "darwin".to_string(),
                    arch: "all".to_string(),
                    url: "https://github.com/oasdiff/oasdiff/releases/download/v1.11.7/oasdiff_1.11.7_darwin_all.tar.gz".to_string(),
                },
                Platform {
                    os: "windows".to_string(),
                    arch: "amd64".to_string(),
                    url: "https://github.com/oasdiff/oasdiff/releases/download/v1.11.7/oasdiff_1.11.7_windows_amd64.tar.gz".to_string(),
                },
            ],
        },
        Tool {
            name: "buf".to_string(),
            version: "1.45.0".to_string(),
            platforms: vec![
                Platform {
                    os: "linux".to_string(),
                    arch: "amd64".to_string(),
                    url: "https://github.com/bufbuild/buf/releases/download/v1.45.0/buf-Linux-x86_64".to_string(),
                },
                Platform {
                    os: "linux".to_string(),
                    arch: "arm64".to_string(),
                    url: "https://github.com/bufbuild/buf/releases/download/v1.45.0/buf-Linux-aarch64".to_string(),
                },
                Platform {
                    os: "darwin".to_string(),
                    arch: "amd64".to_string(),
                    url: "https://github.com/bufbuild/buf/releases/download/v1.45.0/buf-Darwin-x86_64".to_string(),
                },
                Platform {
                    os: "darwin".to_string(),
                    arch: "arm64".to_string(),
                    url: "https://github.com/bufbuild/buf/releases/download/v1.45.0/buf-Darwin-arm64".to_string(),
                },
                Platform {
                    os: "windows".to_string(),
                    arch: "amd64".to_string(),
                    url: "https://github.com/bufbuild/buf/releases/download/v1.45.0/buf-Windows-x86_64.exe".to_string(),
                },
            ],
        },
        Tool {
            name: "atlas".to_string(),
            version: "v0.31.0".to_string(),
            platforms: vec![
                Platform {
                    os: "linux".to_string(),
                    arch: "amd64".to_string(),
                    url: "https://release.ariga.io/atlas/atlas-linux-amd64-v0.31.0".to_string(),
                },
                Platform {
                    os: "linux".to_string(),
                    arch: "arm64".to_string(),
                    url: "https://release.ariga.io/atlas/atlas-linux-arm64-v0.31.0".to_string(),
                },
                Platform {
                    os: "darwin".to_string(),
                    arch: "amd64".to_string(),
                    url: "https://release.ariga.io/atlas/atlas-darwin-amd64-v0.31.0".to_string(),
                },
                Platform {
                    os: "darwin".to_string(),
                    arch: "arm64".to_string(),
                    url: "https://release.ariga.io/atlas/atlas-darwin-arm64-v0.31.0".to_string(),
                },
                Platform {
                    os: "windows".to_string(),
                    arch: "amd64".to_string(),
                    url: "https://release.ariga.io/atlas/atlas-windows-amd64-v0.31.0.exe".to_string(),
                },
            ],
        },
        Tool {
            name: "gitleaks".to_string(),
            version: "8.21.2".to_string(),
            platforms: vec![
                Platform {
                    os: "linux".to_string(),
                    arch: "amd64".to_string(),
                    url: "https://github.com/gitleaks/gitleaks/releases/download/v8.21.2/gitleaks_8.21.2_linux_x64.tar.gz".to_string(),
                },
                Platform {
                    os: "darwin".to_string(),
                    arch: "amd64".to_string(),
                    url: "https://github.com/gitleaks/gitleaks/releases/download/v8.21.2/gitleaks_8.21.2_darwin_x64.tar.gz".to_string(),
                },
                Platform {
                    os: "darwin".to_string(),
                    arch: "arm64".to_string(),
                    url: "https://github.com/gitleaks/gitleaks/releases/download/v8.21.2/gitleaks_8.21.2_darwin_arm64.tar.gz".to_string(),
                },
                Platform {
                    os: "windows".to_string(),
                    arch: "amd64".to_string(),
                    url: "https://github.com/gitleaks/gitleaks/releases/download/v8.21.2/gitleaks_8.21.2_windows_x64.zip".to_string(),
                },
            ],
        },
    ];

    let mut checksums = Vec::new();

    for tool in tools {
        println!("{} Generating checksums for {} {}", "[INFO]".blue(), tool.name, tool.version);

        for platform in tool.platforms {
            let key = format!("{}-{}-{}-{}", tool.name, tool.version, platform.os, platform.arch);
            print!("  {} {}: ", "→".dimmed(), key);

            match get_sha256_for_url(&platform.url) {
                Ok(checksum) => {
                    println!("{}", "OK".green());
                    checksums.push((key, checksum));
                }
                Err(e) => {
                    println!("{} {}", "FAIL".red(), e);
                    return Err(e);
                }
            }
        }
    }

    Ok(checksums)
}

/// Write checksums to file
fn write_checksums(checksums: &[(String, String)]) -> Result<()> {
    let path = Path::new("scripts/tools.sha256");
    let mut file = fs::File::create(path).context("Failed to create tools.sha256")?;

    writeln!(file, "# Tool checksums for bootstrap-tools.sh integrity verification")?;
    writeln!(file, "# Generated: {}", chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ"))?;
    writeln!(file, "# Format: <tool_name>-<version>-<platform> <sha256_checksum>")?;
    writeln!(file, "# To update: cargo xtask tools-checksum-update")?;
    writeln!(file)?;

    // Group by tool for better organization
    let mut grouped: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for (key, checksum) in checksums {
        let parts: Vec<&str> = key.split('-').collect();
        if parts.len() >= 2 {
            let tool_name = parts[0].to_string();
            grouped.entry(tool_name).or_default().push((key.clone(), checksum.clone()));
        }
    }

    // Write in organized sections
    for tool_name in ["oasdiff", "buf", "atlas", "gitleaks"] {
        if let Some(entries) = grouped.get(tool_name) {
            writeln!(file)?;

            match tool_name {
                "oasdiff" => {
                    writeln!(file, "# oasdiff v1.11.7 - OpenAPI Specification diff tool")?;
                    writeln!(
                        file,
                        "# GitHub releases: https://github.com/oasdiff/oasdiff/releases"
                    )?;
                }
                "buf" => {
                    writeln!(file, "# buf v1.45.0 - Protocol Buffers toolchain")?;
                    writeln!(file, "# GitHub releases: https://github.com/bufbuild/buf/releases")?;
                }
                "atlas" => {
                    writeln!(file, "# atlas v0.31.0 - Database schema management tool")?;
                    writeln!(file, "# Release server: https://release.ariga.io/atlas/")?;
                }
                "gitleaks" => {
                    writeln!(file, "# gitleaks v8.21.2 - Secret scanning tool (CI-only)")?;
                    writeln!(
                        file,
                        "# GitHub releases: https://github.com/gitleaks/gitleaks/releases"
                    )?;
                }
                _ => {}
            }

            for (key, checksum) in entries {
                writeln!(file, "{} {}", key, checksum)?;
            }
        }
    }

    println!("{} Updated {}", "✓".green(), path.display());
    Ok(())
}

pub fn run() -> Result<()> {
    println!("{} Updating tool checksums...", "INFO".blue());
    println!();

    let checksums = generate_checksums()?;
    write_checksums(&checksums)?;

    println!();
    println!("{} All checksums updated successfully!", "✓".green());
    println!(
        "{} Run {} to verify the new checksums.",
        "HINT".cyan(),
        "ENFORCE_CHECKSUMS=1 ./bootstrap-tools.sh".bright_yellow()
    );

    Ok(())
}
