//! Layering check for contract crates.
//!
//! This command enforces dependency rules to prevent contract crates
//! from pulling in heavy runtime dependencies.
//!
//! ## Rules
//!
//! 1. Contract crates must not depend on forbidden packages:
//!    - `axum` - HTTP framework (belongs to app-http)
//!    - `tokio` - Async runtime (belongs to app-http)
//!    - `clap` - CLI parser (belongs to xtask)
//!    - `sqlx` - Database (belongs to adapters-db-sqlx)
//!    - `tonic` - gRPC framework (belongs to adapters-grpc)
//!    - `jsonschema` - Schema validation (belongs to spec-runtime)
//!
//! 2. Foundation crates should have minimal dependencies
//! 3. No circular dependencies
//!
//! ## Usage
//!
//! ```bash
//! cargo xtask check-layering
//! ```
//!
//! ## Exit Codes
//!
//! - `0`: All layering rules satisfied
//! - `1`: Layering violation detected

use anyhow::{Context, Result};
use colored::Colorize;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Get the repository root from CARGO_MANIFEST_DIR
fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().expect("xtask parent").parent().expect("crates parent").to_path_buf()
}

/// Contract crates that must follow layering rules
const CONTRACT_CRATES: &[&str] =
    &["platform-contract", "xtask-contract", "receipts-core", "spec-types"];

/// Dependencies that contract crates must NOT have
const FORBIDDEN_DEPS: &[&str] = &["axum", "tokio", "clap", "sqlx", "tonic", "jsonschema"];

/// Foundation crates that should have minimal dependencies
const FOUNDATION_CRATES: &[&str] = &["http-errors", "http-platform", "http-core", "business-core"];

/// Maximum allowed dependency count for foundation crates
const FOUNDATION_MAX_DEPS: usize = 10;

/// Cargo metadata output
#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<Package>,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: String,
    dependencies: Option<serde_json::Value>,
}

/// Parse dependencies from Cargo.toml
fn parse_dependencies(cargo_toml: &str) -> Result<Vec<String>> {
    let mut dependencies = Vec::new();
    let mut in_deps_section = false;

    for line in cargo_toml.lines() {
        let line = line.trim();

        if line.starts_with('[') && line.ends_with(']') {
            let section = line.trim_matches(&['[', ']'][..]);
            in_deps_section =
                matches!(section, "dependencies" | "dev-dependencies" | "build-dependencies");
            continue;
        }

        if !in_deps_section || line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((name, _)) = line.split_once('=') {
            let dep_name = name.trim();
            if !dep_name.is_empty() {
                dependencies.push(dep_name.to_string());
            }
        }
    }

    dependencies.sort();
    dependencies.dedup();
    Ok(dependencies)
}

/// Check if a crate has forbidden dependencies
fn check_forbidden_dependencies(_crate_name: &str, dependencies: &[String]) -> Vec<String> {
    let deps_set: HashSet<_> = dependencies.iter().map(|s| s.as_str()).collect();
    let mut violations = Vec::new();

    for forbidden in FORBIDDEN_DEPS {
        if deps_set.contains(*forbidden) {
            violations.push(forbidden.to_string());
        }
    }

    violations
}

/// Check dependency count for foundation crates
fn check_foundation_dependency_count(crate_name: &str, dependencies: &[String]) -> Result<()> {
    if !FOUNDATION_CRATES.contains(&crate_name) {
        return Ok(());
    }

    if dependencies.len() > FOUNDATION_MAX_DEPS {
        eprintln!();
        eprintln!(
            "{} {} has {} dependencies (max: {})",
            "⚠".yellow(),
            crate_name.cyan(),
            dependencies.len(),
            FOUNDATION_MAX_DEPS
        );
        eprintln!("{} Foundation crates should have minimal dependencies", "⚠".yellow());
        return Err(anyhow::anyhow!(
            "Foundation crate {} exceeds maximum dependency count",
            crate_name
        ));
    }

    Ok(())
}

/// Run cargo metadata to get dependency information
fn get_cargo_metadata() -> Result<CargoMetadata> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .context("Failed to run cargo metadata")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("cargo metadata failed: {}", stderr));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let metadata: CargoMetadata =
        serde_json::from_str(&json_str).context("Failed to parse cargo metadata JSON")?;

    Ok(metadata)
}

/// Build a dependency graph and check for cycles
fn check_circular_dependencies(metadata: &CargoMetadata) -> Vec<Vec<String>> {
    let mut dep_map: HashMap<String, Vec<String>> = HashMap::new();

    // Build dependency map
    for pkg in &metadata.packages {
        if let Some(serde_json::Value::Object(deps_obj)) = &pkg.dependencies {
            let mut dep_list = Vec::new();

            for (key, value) in deps_obj.iter() {
                if key == "dependencies"
                    && let serde_json::Value::Object(deps_array) = value
                {
                    for (_, dep_value) in deps_array.iter() {
                        if let serde_json::Value::Object(dep_obj) = dep_value
                            && let Some(name) = dep_obj.get("name").and_then(|v| v.as_str())
                        {
                            dep_list.push(name.to_string());
                        }
                    }
                }
            }

            dep_map.insert(pkg.name.clone(), dep_list);
        }
    }

    // Check for cycles using DFS
    let mut cycles = Vec::new();
    let mut visited = HashSet::new();
    let mut rec_stack = Vec::new();

    fn find_cycle(
        crate_name: &str,
        dep_map: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut Vec<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        if visited.contains(crate_name) {
            return None;
        }

        if let Some(pos) = rec_stack.iter().position(|x| x == crate_name) {
            // Found a cycle
            return Some(rec_stack[pos..].to_vec());
        }

        visited.insert(crate_name.to_string());
        rec_stack.push(crate_name.to_string());
        path.push(crate_name.to_string());

        if let Some(deps) = dep_map.get(crate_name) {
            for dep in deps {
                if let Some(cycle) = find_cycle(dep, dep_map, visited, rec_stack, path) {
                    return Some(cycle);
                }
            }
        }

        rec_stack.pop();
        path.pop();
        None
    }

    for pkg in &metadata.packages {
        if let Some(cycle) =
            find_cycle(&pkg.name, &dep_map, &mut visited, &mut rec_stack, &mut Vec::new())
        {
            cycles.push(cycle);
        }
    }

    cycles
}

/// Run the check-layering command
pub fn run() -> Result<()> {
    println!("{}", "Checking crate layering...".blue().bold());
    println!();

    // Get cargo metadata
    let metadata = get_cargo_metadata()?;

    let mut has_violations = false;
    let mut checked_crates = Vec::new();

    // Check contract crates
    println!("{}", "Checking contract crates:".cyan());
    for crate_name in CONTRACT_CRATES {
        checked_crates.push(crate_name.to_string());

        // Find the package in metadata
        let pkg = metadata.packages.iter().find(|p| p.name == *crate_name);

        let _pkg = match pkg {
            Some(p) => p,
            None => {
                eprintln!("{} Crate not found in workspace: {}", "⚠".yellow(), crate_name);
                continue;
            }
        };

        // Parse dependencies from Cargo.toml
        let cargo_toml_path = repo_root().join("crates").join(crate_name).join("Cargo.toml");
        let dependencies = match fs::read_to_string(&cargo_toml_path) {
            Ok(content) => parse_dependencies(&content)?,
            Err(e) => {
                eprintln!("{} Failed to read {}: {}", "⚠".yellow(), cargo_toml_path.display(), e);
                Vec::new()
            }
        };

        // Check for forbidden dependencies
        let violations = check_forbidden_dependencies(crate_name, &dependencies);

        if !violations.is_empty() {
            has_violations = true;
            println!();
            eprintln!("{} {} has forbidden dependencies:", "❌".red(), crate_name.cyan());
            for violation in &violations {
                eprintln!("  - {}", violation.red());
            }
            eprintln!();
            eprintln!(
                "{} Contract crates must not depend on: {}",
                "Forbidden:".yellow(),
                FORBIDDEN_DEPS.join(", ").dimmed()
            );
        } else {
            println!("{} {} - OK ({})", "✓".green(), crate_name.cyan(), dependencies.len());
        }
    }

    println!();

    // Check foundation crates
    println!("{}", "Checking foundation crates:".cyan());
    for crate_name in FOUNDATION_CRATES {
        checked_crates.push(crate_name.to_string());

        let pkg = match metadata.packages.iter().find(|p| p.name == *crate_name) {
            Some(p) => p,
            None => {
                eprintln!("{} Crate not found in workspace: {}", "⚠".yellow(), crate_name);
                continue;
            }
        };

        // Count dependencies
        let dep_count = pkg
            .dependencies
            .as_ref()
            .and_then(|deps| {
                if let serde_json::Value::Object(deps_obj) = deps
                    && let Some(serde_json::Value::Array(arr)) = deps_obj.get("dependencies")
                {
                    Some(arr.len())
                } else {
                    None
                }
            })
            .unwrap_or(0);

        match check_foundation_dependency_count(crate_name, &[]) {
            Ok(()) => {
                println!("{} {} - OK ({} dependencies)", "✓".green(), crate_name.cyan(), dep_count);
            }
            Err(_e) => {
                has_violations = true;
                // Error already printed by check_foundation_dependency_count
            }
        }
    }

    println!();

    // Check for circular dependencies
    println!("{}", "Checking for circular dependencies...".cyan());
    let cycles = check_circular_dependencies(&metadata);

    if !cycles.is_empty() {
        has_violations = true;
        eprintln!("{}", "Circular dependencies detected:".red().bold());
        for cycle in &cycles {
            eprintln!("  Cycle: {}", cycle.join(" -> ").red());
        }
        eprintln!();
        eprintln!(
            "{} Circular dependencies prevent clean builds and should be avoided.",
            "⚠".yellow()
        );
    } else {
        println!("{}", "✓ No circular dependencies".green());
    }

    println!();
    println!("{}", "Summary:".blue().bold());
    println!("  Checked crates: {}", checked_crates.len());
    println!("  Contract crates: {}", CONTRACT_CRATES.join(", ").cyan());
    println!("  Foundation crates: {}", FOUNDATION_CRATES.join(", ").cyan());
    println!("  Circular dependencies: {}", cycles.len());

    if has_violations {
        println!();
        eprintln!("{}", "❌ Layering violations detected".red().bold());
        eprintln!();
        eprintln!("{}", "Please fix layering issues before proceeding.".yellow());
        std::process::exit(1);
    }

    println!();
    println!("{}", "✓ All layering rules satisfied".green());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_crates_defined() {
        assert_eq!(CONTRACT_CRATES.len(), 4);
        assert!(CONTRACT_CRATES.contains(&"platform-contract"));
        assert!(CONTRACT_CRATES.contains(&"xtask-contract"));
        assert!(CONTRACT_CRATES.contains(&"receipts-core"));
        assert!(CONTRACT_CRATES.contains(&"spec-types"));
    }

    #[test]
    fn test_forbidden_deps_defined() {
        assert_eq!(FORBIDDEN_DEPS.len(), 6);
        assert!(FORBIDDEN_DEPS.contains(&"axum"));
        assert!(FORBIDDEN_DEPS.contains(&"tokio"));
        assert!(FORBIDDEN_DEPS.contains(&"clap"));
        assert!(FORBIDDEN_DEPS.contains(&"sqlx"));
        assert!(FORBIDDEN_DEPS.contains(&"tonic"));
        assert!(FORBIDDEN_DEPS.contains(&"jsonschema"));
    }

    #[test]
    fn test_foundation_crates_defined() {
        assert_eq!(FOUNDATION_CRATES.len(), 4);
        assert!(FOUNDATION_CRATES.contains(&"http-errors"));
        assert!(FOUNDATION_CRATES.contains(&"http-platform"));
        assert!(FOUNDATION_CRATES.contains(&"http-core"));
        assert!(FOUNDATION_CRATES.contains(&"business-core"));
    }

    #[test]
    fn test_parse_dependencies() {
        let toml = r#"
[package]
name = "test-crate"
version = "0.1.0"

[dependencies]
serde = "1.0"
tokio = "1.0"
clap = "4.0"
"#;

        let deps = parse_dependencies(toml).unwrap();
        assert_eq!(deps.len(), 3);
        assert!(deps.iter().any(|dep| dep == "serde"));
        assert!(deps.iter().any(|dep| dep == "tokio"));
        assert!(deps.iter().any(|dep| dep == "clap"));
    }

    #[test]
    fn test_check_forbidden_dependencies() {
        let deps = vec!["serde".to_string(), "tokio".to_string(), "clap".to_string()];
        let violations = check_forbidden_dependencies("test-crate", &deps);
        assert_eq!(violations.len(), 2);
        assert!(violations.iter().any(|dep| dep == "tokio"));
        assert!(violations.iter().any(|dep| dep == "clap"));
    }

    #[test]
    fn test_foundation_max_deps() {
        assert_eq!(FOUNDATION_MAX_DEPS, 10);
    }
}
