//! Verify the governed Clippy policy ledger and workspace lint surface.

use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value;

const ROOT_CARGO: &str = "Cargo.toml";
const CLIPPY_POLICY: &str = "policy/clippy-lints.toml";
const CLIPPY_DEBT: &str = "policy/clippy-debt.toml";
const CLIPPY_CONFIG: &str = "clippy.toml";
const NO_PANIC_ALLOWLIST: &str = "policy/no-panic-allowlist.toml";
const NON_RUST_ALLOWLIST: &str = "policy/non-rust-allowlist.toml";

const TEST_CARVEOUTS: &[&str] = &[
    "allow-unwrap-in-tests",
    "allow-expect-in-tests",
    "allow-panic-in-tests",
    "allow-indexing-slicing-in-tests",
    "allow-dbg-in-tests",
];

/// Run the lint policy coherence gate.
pub fn run() -> Result<()> {
    let root = std::env::current_dir().context("failed to determine current directory")?;
    let cargo = read_toml(&root.join(ROOT_CARGO))?;
    let policy = read_toml(&root.join(CLIPPY_POLICY))?;

    check_msrv(&cargo, &policy)?;
    check_policy_flags(&policy)?;
    check_active_lints(&cargo, &policy)?;
    check_planned_lints_are_staged(&cargo, &policy)?;
    let lint_inheritance_debt = check_debt(&root.join(CLIPPY_DEBT))?;
    check_workspace_lint_inheritance(&root, &cargo, &lint_inheritance_debt)?;
    check_no_test_carveouts(&root.join(CLIPPY_CONFIG))?;
    check_no_panic_allowlist(&root.join(NO_PANIC_ALLOWLIST))?;
    check_non_rust_allowlist(&root.join(NON_RUST_ALLOWLIST))?;

    println!("lint policy is coherent");
    Ok(())
}

fn read_toml(path: &Path) -> Result<Value> {
    let text =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    toml::from_str::<Value>(&text)
        .with_context(|| format!("failed to parse TOML from {}", path.display()))
}

fn check_msrv(cargo: &Value, policy: &Value) -> Result<()> {
    let cargo_msrv = string_at(cargo, &["workspace", "package", "rust-version"])?;
    let policy_msrv = string_at(policy, &["msrv"])?;
    if cargo_msrv != policy_msrv {
        bail!(
            "workspace.package.rust-version ({cargo_msrv}) must match {CLIPPY_POLICY} msrv ({policy_msrv})"
        );
    }
    Ok(())
}

fn check_policy_flags(policy: &Value) -> Result<()> {
    let policy_table = table_at(policy, &["policy"])?;
    require_bool(policy_table, "panic_free_tests", true)?;
    require_bool(policy_table, "allow_test_carveouts", false)?;
    require_bool(policy_table, "blanket_categories", false)?;
    let suppression_style = string_in_table(policy_table, "suppression_style")?;
    if suppression_style != "expect-with-reason" {
        bail!("policy.suppression_style must be expect-with-reason");
    }
    Ok(())
}

fn check_active_lints(cargo: &Value, policy: &Value) -> Result<()> {
    let cargo_clippy = table_at(cargo, &["workspace", "lints", "clippy"])?;
    for lint in lint_entries(policy)? {
        let status = string_in_table(lint, "status")?;
        if status != "active" {
            continue;
        }
        require_lint_fields(lint, true)?;
        let name = string_in_table(lint, "name")?;
        let short_name = name
            .strip_prefix("clippy::")
            .with_context(|| format!("active lint {name} must use clippy:: prefix"))?;
        let expected_level = string_in_table(lint, "level")?;
        let Some(actual_value) = cargo_clippy.get(short_name) else {
            bail!("active lint {name} is missing from [workspace.lints.clippy]");
        };
        let actual_level = lint_level(actual_value)
            .with_context(|| format!("lint {name} must use a string level or {{ level = ... }}"))?;
        if actual_level != expected_level {
            bail!(
                "active lint {name} has level {actual_level} in Cargo.toml, expected {expected_level}"
            );
        }
    }
    Ok(())
}

fn check_planned_lints_are_staged(cargo: &Value, policy: &Value) -> Result<()> {
    let cargo_msrv = string_at(cargo, &["workspace", "package", "rust-version"])?;
    let cargo_clippy = table_at(cargo, &["workspace", "lints", "clippy"])?;
    for lint in lint_entries(policy)? {
        let status = string_in_table(lint, "status")?;
        if status != "planned" {
            continue;
        }
        require_lint_fields(lint, false)?;
        let activate_when = string_in_table(lint, "activate_when_msrv")?;
        let name = string_in_table(lint, "name")?;
        let short_name = name
            .strip_prefix("clippy::")
            .with_context(|| format!("planned lint {name} must use clippy:: prefix"))?;
        if version_less(cargo_msrv, activate_when) && cargo_clippy.contains_key(short_name) {
            bail!(
                "planned lint {name} is active before MSRV {activate_when}; current MSRV is {cargo_msrv}"
            );
        }
    }
    Ok(())
}

fn check_workspace_lint_inheritance(
    root: &Path,
    cargo: &Value,
    debt_paths: &[String],
) -> Result<()> {
    let members = array_at(cargo, &["workspace", "members"])?;
    for member in members {
        let pattern =
            member.as_str().with_context(|| "workspace.members must contain string paths")?;
        let manifests = member_manifests(root, pattern)?;
        for manifest in manifests {
            let relative_manifest = manifest
                .strip_prefix(root)
                .unwrap_or(manifest.as_path())
                .to_string_lossy()
                .replace('\\', "/");
            if debt_paths.iter().any(|path| path == &relative_manifest) {
                continue;
            }
            let member_cargo = read_toml(&manifest)?;
            let lints = table_at(&member_cargo, &["lints"]).with_context(|| {
                format!("{} must contain [lints] workspace = true", manifest.display())
            })?;
            let workspace = lints.get("workspace").and_then(Value::as_bool).with_context(|| {
                format!("{} [lints].workspace must be true", manifest.display())
            })?;
            if !workspace {
                bail!("{} [lints].workspace must be true", manifest.display());
            }
        }
    }
    Ok(())
}

fn check_no_test_carveouts(path: &Path) -> Result<()> {
    let text =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    for carveout in TEST_CARVEOUTS {
        if text.contains(carveout) {
            bail!("{} must not contain test carveout {carveout}", path.display());
        }
    }
    Ok(())
}

fn check_debt(path: &Path) -> Result<Vec<String>> {
    let debt = read_toml(path)?;
    let mut lint_inheritance_debt = Vec::new();
    let today = chrono::Utc::now().date_naive();
    for entry in optional_array(&debt, "debt")? {
        let lint = require_string(entry, "lint")?;
        let debt_path = require_string(entry, "path")?;
        require_string(entry, "owner")?;
        require_string(entry, "reason")?;
        let expires = require_string(entry, "expires")?;
        let expires = NaiveDate::parse_from_str(expires, "%Y-%m-%d")
            .with_context(|| format!("{} debt expires must be YYYY-MM-DD", path.display()))?;
        if expires < today {
            bail!("{} contains expired lint debt dated {expires}", path.display());
        }
        if lint == "workspace::lint_inheritance" {
            lint_inheritance_debt.push(debt_path.to_string());
        }
    }
    Ok(lint_inheritance_debt)
}

fn check_no_panic_allowlist(path: &Path) -> Result<()> {
    let allowlist = read_toml(path)?;
    for entry in optional_array(&allowlist, "allow")? {
        require_string(entry, "path")?;
        require_string(entry, "family")?;
        require_string(entry, "classification")?;
        require_string(entry, "owner")?;
        require_string(entry, "explanation")?;
        table_in_table(entry, "selector")?;
        if let Some(expires) = optional_string(entry, "expires")? {
            let expires = NaiveDate::parse_from_str(expires, "%Y-%m-%d").with_context(|| {
                format!("{} panic allowlist expires must be YYYY-MM-DD", path.display())
            })?;
            if expires < chrono::Utc::now().date_naive() {
                bail!("{} contains expired panic allowlist entry dated {expires}", path.display());
            }
        }
    }
    Ok(())
}

fn check_non_rust_allowlist(path: &Path) -> Result<()> {
    let allowlist = read_toml(path)?;
    for entry in optional_array(&allowlist, "allow")? {
        let has_path = optional_string(entry, "path")?.is_some();
        let has_glob = optional_string(entry, "glob")?.is_some();
        if has_path == has_glob {
            bail!("{} non-rust allow entries must set exactly one of path or glob", path.display());
        }
        require_string(entry, "kind")?;
        require_string(entry, "owner")?;
        require_string(entry, "reason")?;
        require_string(entry, "surface")?;
        require_string(entry, "classification")?;
        let covered_by = entry.get("covered_by").and_then(Value::as_array).with_context(|| {
            format!("{} non-rust allow entries must include covered_by", path.display())
        })?;
        if covered_by.is_empty() {
            bail!(
                "{} non-rust allow entries must include at least one covered_by command",
                path.display()
            );
        }
        if let Some(expires) = optional_string(entry, "expires")? {
            let expires = NaiveDate::parse_from_str(expires, "%Y-%m-%d").with_context(|| {
                format!("{} non-rust allowlist expires must be YYYY-MM-DD", path.display())
            })?;
            if expires < chrono::Utc::now().date_naive() {
                bail!(
                    "{} contains expired non-rust allowlist entry dated {expires}",
                    path.display()
                );
            }
        }
    }
    Ok(())
}

fn require_lint_fields(lint: &toml::map::Map<String, Value>, active: bool) -> Result<()> {
    require_string_in_table(lint, "name")?;
    require_string_in_table(lint, "level")?;
    require_string_in_table(lint, "status")?;
    require_string_in_table(lint, "class")?;
    require_string_in_table(lint, "reason")?;
    if !active {
        require_string_in_table(lint, "activate_when_msrv")?;
    }
    Ok(())
}

fn lint_entries(policy: &Value) -> Result<Vec<&toml::map::Map<String, Value>>> {
    let entries = array_at(policy, &["lint"])?;
    entries
        .iter()
        .map(|entry| entry.as_table().with_context(|| "each [[lint]] entry must be a table"))
        .collect()
}

fn member_manifests(root: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    if let Some(prefix) = pattern.strip_suffix("/*") {
        let dir = root.join(prefix);
        let mut manifests = Vec::new();
        for entry in
            fs::read_dir(&dir).with_context(|| format!("failed to read {}", dir.display()))?
        {
            let entry =
                entry.with_context(|| format!("failed to read entry in {}", dir.display()))?;
            let manifest = entry.path().join("Cargo.toml");
            if manifest.is_file() {
                manifests.push(manifest);
            }
        }
        manifests.sort();
        return Ok(manifests);
    }
    Ok(vec![root.join(pattern).join("Cargo.toml")])
}

fn version_less(left: &str, right: &str) -> bool {
    version_parts(left) < version_parts(right)
}

fn version_parts(version: &str) -> (u64, u64, u64) {
    let mut parts = version.split('.').map(|part| part.parse::<u64>().unwrap_or(0));
    (parts.next().unwrap_or(0), parts.next().unwrap_or(0), parts.next().unwrap_or(0))
}

fn lint_level(value: &Value) -> Option<&str> {
    value.as_str().or_else(|| value.as_table()?.get("level")?.as_str())
}

fn string_at<'a>(value: &'a Value, path: &[&str]) -> Result<&'a str> {
    let mut current = value;
    for key in path {
        current =
            current.get(*key).with_context(|| format!("missing TOML key {}", path.join(".")))?;
    }
    current.as_str().with_context(|| format!("TOML key {} must be a string", path.join(".")))
}

fn table_at<'a>(value: &'a Value, path: &[&str]) -> Result<&'a toml::map::Map<String, Value>> {
    let mut current = value;
    for key in path {
        current =
            current.get(*key).with_context(|| format!("missing TOML table {}", path.join(".")))?;
    }
    current.as_table().with_context(|| format!("TOML key {} must be a table", path.join(".")))
}

fn array_at<'a>(value: &'a Value, path: &[&str]) -> Result<&'a Vec<Value>> {
    let mut current = value;
    for key in path {
        current =
            current.get(*key).with_context(|| format!("missing TOML array {}", path.join(".")))?;
    }
    current.as_array().with_context(|| format!("TOML key {} must be an array", path.join(".")))
}

fn optional_array<'a>(
    value: &'a Value,
    key: &str,
) -> Result<Vec<&'a toml::map::Map<String, Value>>> {
    let Some(entries) = value.get(key) else {
        return Ok(Vec::new());
    };
    let entries = entries.as_array().with_context(|| format!("{key} must be an array"))?;
    entries
        .iter()
        .map(|entry| {
            entry.as_table().with_context(|| format!("each [[{key}]] entry must be a table"))
        })
        .collect()
}

fn require_bool(table: &toml::map::Map<String, Value>, key: &str, expected: bool) -> Result<()> {
    let actual = table
        .get(key)
        .and_then(Value::as_bool)
        .with_context(|| format!("policy.{key} must be a boolean"))?;
    if actual != expected {
        bail!("policy.{key} must be {expected}");
    }
    Ok(())
}

fn string_in_table<'a>(table: &'a toml::map::Map<String, Value>, key: &str) -> Result<&'a str> {
    table.get(key).and_then(Value::as_str).with_context(|| format!("{key} must be a string"))
}

fn table_in_table<'a>(
    table: &'a toml::map::Map<String, Value>,
    key: &str,
) -> Result<&'a toml::map::Map<String, Value>> {
    table.get(key).and_then(Value::as_table).with_context(|| format!("{key} must be a table"))
}

fn require_string<'a>(table: &'a toml::map::Map<String, Value>, key: &str) -> Result<&'a str> {
    let value = string_in_table(table, key)?;
    if value.trim().is_empty() {
        bail!("{key} must not be empty");
    }
    Ok(value)
}

fn require_string_in_table<'a>(
    table: &'a toml::map::Map<String, Value>,
    key: &str,
) -> Result<&'a str> {
    require_string(table, key)
}

fn optional_string<'a>(
    table: &'a toml::map::Map<String, Value>,
    key: &str,
) -> Result<Option<&'a str>> {
    match table.get(key) {
        Some(value) => value.as_str().map(Some).with_context(|| format!("{key} must be a string")),
        None => Ok(None),
    }
}
