use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Fork entry representing a known template fork
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkEntry {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub domain: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maintainer: Option<ForkMaintainer>,
    pub kernel_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forked_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_synced: Option<String>,
    pub status: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pain_points: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_items: Option<ForkRelatedItems>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkMaintainer {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkRelatedItems {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub issues: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub adrs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub friction: Vec<String>,
}

/// Fork statistics
#[derive(Debug, Default)]
pub struct ForkStats {
    pub active_count: usize,
    pub archived_count: usize,
    pub experimental_count: usize,
    pub total_count: usize,
    pub by_kernel_version: std::collections::HashMap<String, usize>,
}

impl ForkEntry {
    /// Load fork entry from YAML file
    pub fn load(filepath: &Path) -> Result<Self> {
        let content = fs::read_to_string(filepath)
            .with_context(|| format!("Failed to read fork entry file: {}", filepath.display()))?;

        let entry: ForkEntry = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse fork entry YAML: {}", filepath.display()))?;

        Ok(entry)
    }

    /// Save fork entry to YAML file
    pub fn save(&self) -> Result<PathBuf> {
        let forks_dir = Path::new("forks");
        fs::create_dir_all(forks_dir)
            .with_context(|| format!("Failed to create directory: {}", forks_dir.display()))?;

        let filename = format!("{}.yaml", self.id);
        let filepath = forks_dir.join(&filename);

        let yaml_content = serde_yaml::to_string(&self)
            .with_context(|| format!("Failed to serialize fork entry: {}", self.id))?;

        // Add header comment
        let content = format!(
            "# Fork Entry: {}\n# Domain: {}\n# Status: {}\n# Kernel Version: {}\n\n{}",
            self.name, self.domain, self.status, self.kernel_version, yaml_content
        );

        fs::write(&filepath, content)
            .with_context(|| format!("Failed to write fork entry file: {}", filepath.display()))?;

        Ok(filepath)
    }
}

/// Load all fork entries from forks/ directory
pub fn load_all_fork_entries() -> Result<Vec<ForkEntry>> {
    let forks_dir = Path::new("forks");
    if !forks_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();

    for entry in fs::read_dir(forks_dir)
        .with_context(|| format!("Failed to read directory: {}", forks_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        // Skip non-YAML files, schema files, and registry index
        if !path.is_file()
            || path.extension().and_then(|s| s.to_str()) != Some("yaml")
            || path
                .file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|name| name.contains("schema") || name.contains("registry"))
        {
            continue;
        }

        match ForkEntry::load(&path) {
            Ok(fork) => entries.push(fork),
            Err(e) => {
                eprintln!("Warning: Failed to load fork entry from {}: {}", path.display(), e);
            }
        }
    }

    // Sort by domain, then by name
    entries.sort_by(|a, b| a.domain.cmp(&b.domain).then_with(|| a.name.cmp(&b.name)));

    Ok(entries)
}

/// Calculate fork statistics
pub fn calculate_stats(entries: &[ForkEntry]) -> ForkStats {
    let mut stats = ForkStats::default();

    for entry in entries {
        stats.total_count += 1;

        // Count by status
        match entry.status.as_str() {
            "active" => stats.active_count += 1,
            "archived" => stats.archived_count += 1,
            "experimental" => stats.experimental_count += 1,
            _ => {}
        }

        // Count by kernel version
        *stats.by_kernel_version.entry(entry.kernel_version.clone()).or_insert(0) += 1;
    }

    stats
}

/// JSON output structure for fork-list
#[derive(Debug, Serialize)]
struct ForkListJson {
    timestamp: String,
    total_count: usize,
    stats: ForkStatsJson,
    forks: Vec<ForkEntry>,
}

#[derive(Debug, Serialize)]
struct ForkStatsJson {
    active: usize,
    archived: usize,
    experimental: usize,
    by_kernel_version: std::collections::HashMap<String, usize>,
}

/// List fork entries filtered by status or domain
pub fn list_fork_entries(
    status_filter: Option<&str>,
    domain_filter: Option<&str>,
    json: bool,
) -> Result<()> {
    let entries = load_all_fork_entries()?;

    let filtered: Vec<&ForkEntry> = entries
        .iter()
        .filter(|e| {
            let status_match = status_filter.is_none() || Some(e.status.as_str()) == status_filter;
            let domain_match = domain_filter.is_none() || e.domain.contains(domain_filter.unwrap());
            status_match && domain_match
        })
        .collect();

    if filtered.is_empty() {
        if json {
            // Empty JSON output
            let output = ForkListJson {
                timestamp: chrono::Utc::now().to_rfc3339(),
                total_count: 0,
                stats: ForkStatsJson {
                    active: 0,
                    archived: 0,
                    experimental: 0,
                    by_kernel_version: std::collections::HashMap::new(),
                },
                forks: Vec::new(),
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            println!("No fork entries found.");
        }
        return Ok(());
    }

    let stats = calculate_stats(&entries);

    if json {
        // JSON output
        let output = ForkListJson {
            timestamp: chrono::Utc::now().to_rfc3339(),
            total_count: filtered.len(),
            stats: ForkStatsJson {
                active: stats.active_count,
                archived: stats.archived_count,
                experimental: stats.experimental_count,
                by_kernel_version: stats.by_kernel_version.clone(),
            },
            forks: filtered.into_iter().cloned().collect(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        // Human-readable output
        println!(
            "\n{} Fork Entries:\n",
            if status_filter.is_some() || domain_filter.is_some() { "Filtered" } else { "All" }
        );

        for entry in filtered {
            let status_badge = match entry.status.as_str() {
                "active" => "[ACTIVE]",
                "archived" => "[ARCHIVED]",
                "experimental" => "[EXPERIMENTAL]",
                _ => "[UNKNOWN]",
            };

            println!("  {} {} - {}", status_badge, entry.id, entry.name);
            println!("     Domain: {} | Kernel: {}", entry.domain, entry.kernel_version);

            if let Some(url) = &entry.url {
                println!("     URL: {}", url);
            }

            if let Some(maintainer) = &entry.maintainer {
                print!("     Maintainer: {}", maintainer.name);
                if let Some(contact) = &maintainer.contact {
                    print!(" ({})", contact);
                }
                println!();
            }

            if !entry.features.is_empty() {
                println!("     Features: {}", entry.features.join(", "));
            }

            if !entry.pain_points.is_empty() {
                println!("     Pain Points: {}", entry.pain_points.join(", "));
            }

            println!();
        }

        println!("Total: {} forks", stats.total_count);
        println!(
            "Status: active: {}, experimental: {}, archived: {}",
            stats.active_count, stats.experimental_count, stats.archived_count
        );

        if !stats.by_kernel_version.is_empty() {
            print!("Kernel Versions: ");
            let mut versions: Vec<_> = stats.by_kernel_version.iter().collect();
            versions.sort_by(|a, b| b.0.cmp(a.0)); // Sort by version descending
            let version_strs: Vec<String> =
                versions.iter().map(|(v, c)| format!("{} ({})", v, c)).collect();
            println!("{}", version_strs.join(", "));
        }

        println!();
    }

    Ok(())
}

/// Get the domain prefix for fork ID generation
fn get_domain_prefix(domain: &str) -> String {
    // Extract uppercase letters or use first 4 chars
    let prefix: String =
        domain.chars().filter(|c| c.is_uppercase() || c.is_numeric()).take(4).collect();

    if prefix.is_empty() {
        // Fallback: use first 4 chars, uppercase
        domain.chars().take(4).collect::<String>().to_uppercase()
    } else {
        prefix
    }
}

/// Find the next available fork ID for a given domain
fn find_next_fork_id(domain: &str) -> Result<String> {
    let entries = load_all_fork_entries()?;
    let prefix = get_domain_prefix(domain);
    let pattern = format!("FORK-{}-", prefix);

    // Find the highest number for this domain prefix
    let max_number = entries
        .iter()
        .filter_map(|e| {
            if e.id.starts_with(&pattern) {
                // Extract the number part
                e.id.strip_prefix(&pattern)?.parse::<u32>().ok()
            } else {
                None
            }
        })
        .max()
        .unwrap_or(0);

    // Next ID is max + 1
    let next_number = max_number + 1;
    Ok(format!("FORK-{}-{:03}", prefix, next_number))
}

/// Arguments for creating a new fork entry
pub struct CreateForkArgs<'a> {
    pub name: &'a str,
    pub domain: &'a str,
    pub kernel_version: &'a str,
    pub url: Option<&'a str>,
    pub maintainer_name: Option<&'a str>,
    pub maintainer_contact: Option<&'a str>,
    pub status: Option<&'a str>,
    pub notes: Option<&'a str>,
}

/// Create a new fork entry
pub fn create_fork_entry(args: CreateForkArgs) -> Result<()> {
    // Validate status
    let valid_statuses = ["active", "archived", "experimental"];
    let status = args.status.unwrap_or("experimental");
    if !valid_statuses.contains(&status) {
        anyhow::bail!("Invalid status '{}'. Must be one of: {}", status, valid_statuses.join(", "));
    }

    // Validate kernel version format
    if !args.kernel_version.starts_with('v') || args.kernel_version.matches('.').count() != 2 {
        anyhow::bail!(
            "Invalid kernel_version '{}'. Must be in format vX.Y.Z (e.g., v3.3.3)",
            args.kernel_version
        );
    }

    // Generate ID
    let id = find_next_fork_id(args.domain)?;

    // Get today's date
    let forked_at = chrono::Local::now().format("%Y-%m-%d").to_string();

    // Build maintainer if name provided
    let maintainer = args.maintainer_name.map(|name| ForkMaintainer {
        name: name.to_string(),
        contact: args.maintainer_contact.map(String::from),
    });

    // Create fork entry
    let entry = ForkEntry {
        id: id.clone(),
        name: args.name.to_string(),
        url: args.url.map(String::from),
        domain: args.domain.to_string(),
        maintainer,
        kernel_version: args.kernel_version.to_string(),
        forked_at: Some(forked_at.clone()),
        last_synced: None,
        status: status.to_string(),
        features: Vec::new(),
        pain_points: Vec::new(),
        notes: args.notes.map(String::from),
        related_items: None,
    };

    // Save to file
    let filepath = entry.save()?;

    println!("✅ Registered fork: {}", id);
    println!("   File: {}", filepath.display());
    println!("   Name: {}", args.name);
    println!("   Domain: {}", args.domain);
    println!("   Kernel Version: {}", args.kernel_version);
    println!("   Status: {}", status);
    println!("   Forked At: {}", forked_at);

    println!("\n💡 Next steps:");
    println!("   1. Edit {} to add features, pain_points, or notes", filepath.display());
    println!("   2. Use 'cargo xtask fork-list' to view all registered forks");
    println!("   3. Share kernel pain points via GitHub issues or friction entries");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fork_entry_serialization() {
        let entry = ForkEntry {
            id: "FORK-TEST-001".to_string(),
            name: "Test Fork".to_string(),
            url: Some("https://github.com/org/test-fork".to_string()),
            domain: "testing".to_string(),
            maintainer: Some(ForkMaintainer {
                name: "Test Maintainer".to_string(),
                contact: Some("test@example.com".to_string()),
            }),
            kernel_version: "v3.3.3".to_string(),
            forked_at: Some("2025-11-26".to_string()),
            last_synced: None,
            status: "experimental".to_string(),
            features: vec!["test-feature".to_string()],
            pain_points: Vec::new(),
            notes: Some("Test notes".to_string()),
            related_items: None,
        };

        let yaml = serde_yaml::to_string(&entry).unwrap();
        assert!(yaml.contains("FORK-TEST-001"));
        assert!(yaml.contains("testing"));
        assert!(yaml.contains("v3.3.3"));
    }

    #[test]
    fn test_fork_entry_deserialization() {
        let yaml = r#"
id: FORK-TEST-002
name: "Another Test Fork"
domain: ml-platform
kernel_version: v3.3.3
status: active
"#;

        let entry: ForkEntry = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(entry.id, "FORK-TEST-002");
        assert_eq!(entry.domain, "ml-platform");
        assert_eq!(entry.kernel_version, "v3.3.3");
        assert_eq!(entry.status, "active");
    }

    #[test]
    fn test_domain_prefix_generation() {
        assert_eq!(get_domain_prefix("rust-sdk"), "RUST");
        assert_eq!(get_domain_prefix("MLPlatform"), "MLP");
        assert_eq!(get_domain_prefix("web"), "WEB");
        assert_eq!(get_domain_prefix("KnowledgeHub"), "KH");
    }
}
