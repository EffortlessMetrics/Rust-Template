//! Rust implementations for legacy shell maintenance scripts.
//!
//! The repository keeps tiny shell wrappers for backwards-compatible entrypoints,
//! but command logic lives here so validation and maintenance flows are typed,
//! testable, and cross-platform where external CLIs allow it.

use anyhow::{Context, Result, anyhow, bail};
use colored::Colorize;
use serde_json::json;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn validate_ts_config() -> Result<()> {
    println!("Validating TypeScript configuration standards...\n");
    let mut violations = 0usize;

    for path in find_files(Path::new("."), "tsconfig.json")? {
        let rel = display_path(&path);
        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;

        if regex::Regex::new(r#""moduleResolution"\s*:\s*"(node10|node)""#)?.is_match(&content) {
            println!("{} {}", "✗".red(), rel);
            println!("  - Uses deprecated moduleResolution (node10 or node)");
            println!("  - Fix: Use \"moduleResolution\": \"NodeNext\"");
            violations += 1;
        }

        if regex::Regex::new(r#""ignoreDeprecations""#)?.is_match(&content) {
            println!("{} {}", "✗".red(), rel);
            println!("  - Contains ignoreDeprecations flag");
            println!("  - Fix: Remove ignoreDeprecations and address warnings");
            violations += 1;
        }

        if content.contains("\"moduleResolution\"")
            && !regex::Regex::new(r#""moduleResolution"\s*:\s*"NodeNext""#)?.is_match(&content)
        {
            println!("{} {}", "⚠".yellow(), rel);
            println!("  - moduleResolution is not NodeNext (advisory)");
        }
    }

    if violations > 0 {
        println!("\n{}", format!("Found {violations} TypeScript config violation(s)").red());
        println!("\nTypeScript configuration standards for this repo:");
        println!("  - module: \"NodeNext\"");
        println!("  - moduleResolution: \"NodeNext\"");
        println!("  - No ignoreDeprecations flags");
        println!("\nSee docs/how-to/implement-backstage-plugin.md for details.");
        bail!("TypeScript configuration validation failed");
    }

    println!("{} All TypeScript configurations pass validation", "✓".green());
    Ok(())
}

pub fn validate_build_infrastructure() -> Result<()> {
    println!("=== Build Infrastructure Validation ===\n");
    let mut failed = 0usize;

    check(
        "scripts/tools.sha256 exists and is readable",
        Path::new("scripts/tools.sha256").is_file(),
        &mut failed,
    );

    let checksums = fs::read_to_string("scripts/tools.sha256").unwrap_or_default();
    let checksum_re = regex::Regex::new(r"^[A-Za-z0-9-]*-[0-9A-Za-z.-]* [a-f0-9]{64}$")?;
    let entries = checksums.lines().filter(|line| checksum_re.is_match(line)).count();
    check("scripts/tools.sha256 contains checksum entries", entries > 0, &mut failed);
    check("all required tools have checksum entries", entries >= 12, &mut failed);
    check(
        "checksum format covers oasdiff, buf, and atlas",
        checksums.lines().any(|line| line.starts_with("oasdiff-1.11.7-"))
            && checksums.lines().any(|line| line.starts_with("buf-1.45.0-"))
            && checksums.lines().any(|line| line.starts_with("atlas-v0.31.0-")),
        &mut failed,
    );

    let toolchain_version = extract_quoted_value("rust-toolchain.toml", "channel")?;
    let cargo_version = extract_quoted_value("Cargo.toml", "rust-version")?;
    check(
        &format!(
            "Rust version consistency (toolchain={}, Cargo={})",
            toolchain_version.as_deref().unwrap_or("NOT_FOUND"),
            cargo_version.as_deref().unwrap_or("NOT_FOUND")
        ),
        toolchain_version.is_some() && toolchain_version == cargo_version,
        &mut failed,
    );

    let crate_manifests = find_files(Path::new("crates"), "Cargo.toml")?;
    let crates_with_rust_version = crate_manifests
        .iter()
        .filter(|path| fs::read_to_string(path).unwrap_or_default().contains("rust-version"))
        .count();
    if crates_with_rust_version == crate_manifests.len() {
        println!(
            "{} MSRV declared in all crates ({}/{})",
            "✓".green(),
            crates_with_rust_version,
            crate_manifests.len()
        );
    } else {
        println!(
            "{} MSRV missing in some crates ({}/{})",
            "⚠".yellow(),
            crates_with_rust_version,
            crate_manifests.len()
        );
    }

    let ignored_advisories = fs::read_to_string("deny.toml")
        .unwrap_or_default()
        .lines()
        .filter(|line| line.contains("\"RUSTSEC-"))
        .count();
    if ignored_advisories > 0 {
        println!(
            "{} {ignored_advisories} ignored security advisories documented in deny.toml",
            "⚠".yellow()
        );
    }

    check(
        "bootstrap-tools.sh remains as compatibility bootstrap",
        Path::new("bootstrap-tools.sh").is_file(),
        &mut failed,
    );
    check(
        "Rust checksum verifier is available",
        Path::new("crates/xtask/src/commands/tools_checksum_verify.rs").is_file(),
        &mut failed,
    );
    check(
        "Rust checksum updater is available",
        Path::new("crates/xtask/src/commands/tools_checksum_update.rs").is_file(),
        &mut failed,
    );
    check(
        "validate-build-infrastructure wrapper is executable",
        is_executable("scripts/validate-build-infrastructure.sh"),
        &mut failed,
    );
    check(
        "validate-ts-config wrapper is executable",
        is_executable("scripts/validate-ts-config.sh"),
        &mut failed,
    );

    if failed > 0 {
        bail!("build infrastructure validation failed with {failed} error(s)");
    }
    println!("\n{} Build infrastructure validation passed", "✓".green());
    Ok(())
}

pub fn validate_ci_optimizations() -> Result<()> {
    println!("=== CI Optimization Validation ===\n");
    let mut failed = 0usize;

    check(
        "setup-rust-nix composite action exists",
        Path::new(".github/actions/setup-rust-nix/action.yml").is_file(),
        &mut failed,
    );
    check(
        "sccache-stats composite action exists",
        Path::new(".github/actions/sccache-stats/action.yml").is_file(),
        &mut failed,
    );

    for action in find_files(Path::new(".github/actions"), "action.yml")? {
        let content = fs::read_to_string(&action)?;
        check(
            &format!(
                "{} has valid structure",
                action
                    .parent()
                    .and_then(Path::file_name)
                    .and_then(OsStr::to_str)
                    .unwrap_or("action")
            ),
            content.contains("name:")
                && content.contains("description:")
                && content.contains("runs:"),
            &mut failed,
        );
    }

    let workflow_files = find_files(Path::new(".github/workflows"), "yml")?;
    let composite_users = workflow_files
        .iter()
        .filter(|path| {
            fs::read_to_string(path).unwrap_or_default().contains("uses: ./.github/actions")
        })
        .count();
    if composite_users >= 5 {
        println!("{} {composite_users} workflows using composite actions", "✓".green());
    } else {
        println!(
            "{} Only {composite_users} workflows using composite actions (expected 5+)",
            "⚠".yellow()
        );
    }

    for workflow in ["ci-agents.yml", "tier1-selftest.yml", "policy-test.yml"] {
        let content =
            fs::read_to_string(Path::new(".github/workflows").join(workflow)).unwrap_or_default();
        check(
            &format!("{workflow} has concurrency control"),
            content.contains("concurrency:"),
            &mut failed,
        );
    }

    check_contains(
        ".github/workflows/ci-msrv.yml",
        "Test with MSRV",
        "ci-msrv.yml runs tests",
        &mut failed,
    );
    check_contains(
        ".github/workflows/tier1-selftest.yml",
        "cargo xtask selftest",
        "tier1-selftest.yml runs selftest",
        &mut failed,
    );

    let missing_timeout = workflow_files
        .iter()
        .filter(|path| !fs::read_to_string(path).unwrap_or_default().contains("timeout-minutes:"))
        .count();
    if missing_timeout == 0 {
        println!("{} All workflows have timeouts", "✓".green());
    } else {
        println!("{} {missing_timeout} workflows missing timeouts", "⚠".yellow());
    }

    if failed > 0 {
        bail!("CI optimization validation failed with {failed} error(s)");
    }
    println!("\n=== Validation Summary ===\nℹ All critical checks passed ✓");
    Ok(())
}

pub fn fix_msrv_declarations() -> Result<()> {
    println!("=== Fixing MSRV Declarations Across Workspace ===");
    println!("Target: Add rust-version.workspace = true to all crates\n");
    let crates = [
        "acceptance",
        "ac-kernel",
        "adapters-db-sqlx",
        "adapters-grpc",
        "app-http",
        "business-core",
        "model",
        "rust_iac_xtask_core",
        "spec-runtime",
        "telemetry",
        "xtask",
    ];
    let mut fixed = 0usize;

    for krate in crates {
        let path = Path::new("crates").join(krate).join("Cargo.toml");
        if !path.exists() {
            println!("  {} Cargo.toml not found for {krate}", "✗".red());
            continue;
        }
        let content = fs::read_to_string(&path)?;
        if content.contains("rust-version") {
            println!("Processing {krate}: {} already has rust-version declaration", "✓".green());
            continue;
        }
        if !content.contains("edition.workspace = true") {
            println!("Processing {krate}: {} no edition.workspace found", "✗".red());
            continue;
        }
        let updated = content.replace(
            "edition.workspace = true",
            "edition.workspace = true\nrust-version.workspace = true",
        );
        fs::write(&path, updated)?;
        fixed += 1;
        println!("Processing {krate}: {} added rust-version.workspace = true", "✓".green());
    }

    println!("\n=== MSRV Fix Summary ===");
    println!("Crates processed: {}", crates.len());
    println!("Crates fixed: {fixed}");
    println!("Next steps:");
    println!("1. Run 'cargo check --workspace' to verify fixes");
    println!("2. Run 'cargo xtask validate-build-infrastructure' to confirm");
    println!("3. Test MSRV validation with: cargo +1.92.0 check --workspace");
    Ok(())
}

pub fn security_advisories_plan() -> Result<()> {
    println!(
        "=== Security Advisory Resolution Plan ===\nAnalyzing and providing fixes for ignored advisories\n"
    );
    advisory(
        "RUSTSEC-2025-0057",
        "fxhash unmaintained",
        "selectors → scraper → app-http (dev-only)",
        "scraper",
    )?;
    advisory(
        "RUSTSEC-2025-0134",
        "rustls-pemfile unmaintained",
        "bollard → testcontainers → adapters-db-sqlx (dev-only)",
        "testcontainers",
    )?;
    println!("3. Recommended Immediate Actions:");
    println!("   ✓ Keep advisories ignored but add monitoring and timeline");
    println!("   ✓ Add quarterly security review for these dependencies");
    println!("   ✓ Document acceptance criteria for dev-only dependencies");
    println!("   ✓ Create GitHub issue trackers for upstream fixes\n");
    println!("4. Long-term Security Strategy:");
    println!("   - Implement automated dependency update workflow");
    println!("   - Add security scanning to CI pipeline");
    println!("   - Establish policy for dev-only dependency risks");
    println!("   - Create security advisory response process");
    Ok(())
}

pub fn k8s_secrets_policy_test() -> Result<()> {
    println!("K8s Secrets Policy Test (Manual Verification)");
    println!("==============================================\n");
    let tests = [
        (
            "Valid deployment with envFrom (should PASS)",
            "policy/testdata/k8s_secrets_valid.json",
            "[]",
        ),
        (
            "Deployment with literal DATABASE_URL (should FAIL)",
            "policy/testdata/k8s_secrets_literal_database_url.json",
            "literal value for sensitive env var 'DATABASE_URL'",
        ),
        (
            "Deployment with literal API keys (should FAIL)",
            "policy/testdata/k8s_secrets_literal_api_key.json",
            "API_KEY",
        ),
        (
            "Deployment using ConfigMap for passwords (should FAIL)",
            "policy/testdata/k8s_secrets_configmap_for_secret.json",
            "configMapKeyRef",
        ),
    ];
    let mut failed = 0usize;
    for (idx, (name, input, expected)) in tests.iter().enumerate() {
        println!("Test {}: {name}", idx + 1);
        let output = opa_eval(input)?;
        if output.contains(expected) {
            println!("  {} PASS", "✓".green());
        } else {
            println!("  {} FAIL: expected output containing {expected:?}", "✗".red());
            println!("  Output: {output}");
            failed += 1;
        }
        println!();
    }
    if failed > 0 {
        bail!("{failed} K8s secret policy test(s) failed");
    }
    println!("==============================================\nManual policy verification complete");
    Ok(())
}

pub fn check_schema_compat() -> Result<()> {
    let registry = std::env::var("SCHEMA_REGISTRY_URL")
        .context("Environment variable SCHEMA_REGISTRY_URL must be set")?;
    let auth = std::env::var("SCHEMA_REGISTRY_AUTH").ok();
    println!("Checking schema compatibility against {registry}");

    for schema_file in find_files(Path::new("specs/events/json-schema"), "json")? {
        let filename = schema_file
            .file_stem()
            .and_then(OsStr::to_str)
            .ok_or_else(|| anyhow!("invalid schema filename"))?;
        let subject = format!("{filename}-value");
        println!("---------------------------------------------------");
        println!("Subject: {subject}");
        println!("File:    {}", schema_file.display());
        let schema = fs::read_to_string(&schema_file)?;
        let payload = json!({"schemaType": "JSON", "schema": schema}).to_string();
        let (code, body) = curl_post_schema(&registry, auth.as_deref(), &subject, &payload)?;
        println!("HTTP Code: {code}");
        println!("{body}");

        match code {
            200 => {
                let value: serde_json::Value =
                    serde_json::from_str(&body).context("schema registry returned invalid JSON")?;
                if value.get("is_compatible").and_then(serde_json::Value::as_bool) == Some(true) {
                    println!("✅ Compatible");
                } else {
                    println!("❌ Incompatible");
                    if let Some(messages) =
                        value.get("messages").and_then(serde_json::Value::as_array)
                    {
                        for message in messages {
                            println!("{}", message.as_str().unwrap_or("<non-string message>"));
                        }
                    }
                    bail!("schema {subject} is incompatible");
                }
            }
            404 => {
                let value: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
                let error_code = value.get("error_code").and_then(serde_json::Value::as_i64);
                if matches!(error_code, Some(40401 | 40402)) {
                    println!(
                        "ℹ️  Subject or version not found (likely first version). Treated as compatible."
                    );
                } else {
                    bail!("Error 404: {body}");
                }
            }
            _ => bail!("Error checking compatibility: HTTP {code}\n{body}"),
        }
    }
    println!("---------------------------------------------------\nAll schemas checked.");
    Ok(())
}

fn find_files(root: &Path, name_or_ext: &str) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(root).into_iter().filter_entry(|entry| {
        let path = entry.path();
        !path.components().any(|component| {
            matches!(component.as_os_str().to_str(), Some(".git" | "node_modules" | "target"))
        })
    }) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let path = entry.path();
            let file_name = path.file_name().and_then(OsStr::to_str).unwrap_or_default();
            let ext = path.extension().and_then(OsStr::to_str).unwrap_or_default();
            if file_name == name_or_ext || ext == name_or_ext {
                files.push(path.to_path_buf());
            }
        }
    }
    files.sort();
    Ok(files)
}

fn check(label: &str, ok: bool, failed: &mut usize) {
    if ok {
        println!("{} {label}", "✓".green());
    } else {
        println!("{} {label}", "✗".red());
        *failed += 1;
    }
}

fn check_contains(path: &str, needle: &str, label: &str, failed: &mut usize) {
    check(label, fs::read_to_string(path).unwrap_or_default().contains(needle), failed);
}

fn display_path(path: &Path) -> String {
    path.strip_prefix(".").unwrap_or(path).display().to_string()
}

fn extract_quoted_value(path: &str, key: &str) -> Result<Option<String>> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).with_context(|| format!("failed to read {path}")),
    };
    let pattern = format!(r#"(?m)^\s*{}\s*=\s*"([^"]+)""#, regex::escape(key));
    Ok(regex::Regex::new(&pattern)?
        .captures(&content)
        .and_then(|captures| captures.get(1).map(|matched| matched.as_str().to_string())))
}

#[cfg(unix)]
fn is_executable(path: &str) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path).map(|meta| meta.permissions().mode() & 0o111 != 0).unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &str) -> bool {
    Path::new(path).is_file()
}

fn advisory(id: &str, title: &str, path: &str, crate_name: &str) -> Result<()> {
    println!("Analyzing {id} ({title})");
    println!("   Path: {path}");
    println!("   Risk: Unmaintained dependency (not directly exploitable)");
    println!("   Dev-only status: {}", is_dev_only(crate_name)?);
    println!("   Resolution Options:");
    println!("   a) Wait for upstream fix");
    println!("   b) Replace the dependency in affected tests/tools");
    println!("   c) Accept risk for dev-only dependency with monitoring\n");
    Ok(())
}

fn is_dev_only(crate_name: &str) -> Result<bool> {
    let needle = format!("{crate_name} =");
    for path in find_files(Path::new("crates"), "Cargo.toml")? {
        let content = fs::read_to_string(&path)?;
        let mut in_dev = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') {
                in_dev = trimmed == "[dev-dependencies]";
            }
            if trimmed.starts_with(&needle) && !in_dev {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

fn opa_eval(input_path: &str) -> Result<String> {
    let input =
        fs::File::open(input_path).with_context(|| format!("failed to open {input_path}"))?;
    let output = Command::new("opa")
        .current_dir("policy")
        .args(["eval", "-d", "k8s.rego", "-I", "data.main.deny"])
        .stdin(Stdio::from(input))
        .output()
        .context("failed to run opa; install OPA to execute policy tests")?;
    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    Ok(text)
}

fn curl_post_schema(
    registry: &str,
    auth: Option<&str>,
    subject: &str,
    payload: &str,
) -> Result<(u16, String)> {
    let body = tempfile::NamedTempFile::new()?;
    let mut payload_file = tempfile::NamedTempFile::new()?;
    payload_file.write_all(payload.as_bytes())?;
    let body_path = body.path().to_path_buf();
    let url = format!(
        "{}/compatibility/subjects/{}/versions/latest",
        registry.trim_end_matches('/'),
        subject
    );

    let mut command = Command::new("curl");
    command.args(["-s", "-w", "%{http_code}", "-o"]);
    command.arg(&body_path);
    if let Some(auth) = auth {
        command.args(["-u", auth]);
    }
    command.args([
        "-H",
        "Content-Type: application/vnd.schemaregistry.v1+json",
        "-X",
        "POST",
        "-d",
    ]);
    command.arg(format!("@{}", payload_file.path().display()));
    command.arg(url);

    let output = command.output().context("failed to run curl")?;
    if !output.status.success() {
        bail!("curl failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    let code_text = String::from_utf8(output.stdout)?.trim().to_string();
    let code = code_text
        .parse::<u16>()
        .with_context(|| format!("invalid curl HTTP code {code_text:?}"))?;
    let response_body = fs::read_to_string(body.path())?;
    Ok((code, response_body))
}
