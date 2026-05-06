//! Validate the workspace strict Clippy policy ledger and inheritance shape.

use anyhow::{Context, Result, bail};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const REQUIRED_POLICY_FILES: &[&str] = &[
    "policy/clippy-lints.toml",
    "policy/clippy-debt.toml",
    "policy/no-panic-allowlist.toml",
    "policy/non-rust-allowlist.toml",
    "docs/CLIPPY_POLICY.md",
    "clippy.toml",
];

const TEST_CARVEOUTS: &[&str] = &[
    "allow-unwrap-in-tests",
    "allow-expect-in-tests",
    "allow-panic-in-tests",
    "allow-indexing-slicing-in-tests",
    "allow-dbg-in-tests",
];

const REQUIRED_PLANNED: &[(&str, &str)] = &[
    ("clippy::same_length_and_capacity", "1.94"),
    ("clippy::manual_ilog2", "1.94"),
    ("clippy::decimal_bitwise_operands", "1.94"),
    ("clippy::needless_type_cast", "1.94"),
    ("clippy::disallowed_fields", "1.95"),
    ("clippy::manual_checked_ops", "1.95"),
    ("clippy::manual_take", "1.95"),
    ("clippy::manual_pop_if", "1.95"),
    ("clippy::duration_suboptimal_units", "1.95"),
    ("clippy::unnecessary_trailing_comma", "1.95"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckLintPolicyArgs {
    pub report_only: bool,
}

pub fn run(args: CheckLintPolicyArgs) -> Result<()> {
    let root = workspace_root()?;
    let mut findings = Vec::new();

    check_required_files(&root, &mut findings);
    let cargo = read(root.join("Cargo.toml"), &mut findings);
    let ledger = read(root.join("policy/clippy-lints.toml"), &mut findings);
    let debt = read(root.join("policy/clippy-debt.toml"), &mut findings);
    let clippy = read(root.join("clippy.toml"), &mut findings);

    if let (Some(cargo), Some(ledger)) = (cargo.as_deref(), ledger.as_deref()) {
        check_msrv(cargo, ledger, &mut findings);
        check_active_lints(cargo, ledger, &mut findings);
        check_planned_lints(cargo, ledger, &mut findings);
    }

    if let Some(cargo) = cargo.as_deref() {
        check_workspace_members(&root, cargo, &mut findings)?;
    }

    if let Some(clippy) = clippy.as_deref() {
        check_no_test_carveouts(clippy, &mut findings);
    }

    if let Some(debt) = debt.as_deref() {
        check_debt(debt, &mut findings);
    }

    if findings.is_empty() {
        println!("lint policy check passed");
        return Ok(());
    }

    if args.report_only {
        for finding in &findings {
            println!("lint policy: {finding}");
        }
        println!("lint policy report completed with {} finding(s)", findings.len());
        Ok(())
    } else {
        for finding in &findings {
            eprintln!("lint policy: {finding}");
        }
        bail!("lint policy check failed with {} finding(s)", findings.len())
    }
}

fn workspace_root() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("failed to resolve workspace root from CARGO_MANIFEST_DIR")
}

fn check_required_files(root: &Path, findings: &mut Vec<String>) {
    for rel in REQUIRED_POLICY_FILES {
        if !root.join(rel).exists() {
            findings.push(format!("missing required policy file `{rel}`"));
        }
    }
}

fn read(path: PathBuf, findings: &mut Vec<String>) -> Option<String> {
    match fs::read_to_string(&path) {
        Ok(contents) => Some(contents),
        Err(err) => {
            findings.push(format!("failed to read `{}`: {err}", path.display()));
            None
        }
    }
}

fn check_msrv(cargo: &str, ledger: &str, findings: &mut Vec<String>) {
    let cargo_msrv = scalar_value(cargo, "rust-version");
    let ledger_msrv = scalar_value(ledger, "msrv");

    if cargo_msrv.as_deref() != Some("1.93") {
        findings.push("workspace.package.rust-version must be `1.93`".to_string());
    }

    if ledger_msrv.as_deref() != Some("1.93") {
        findings.push("policy/clippy-lints.toml msrv must be `1.93`".to_string());
    }

    if cargo_msrv != ledger_msrv {
        findings.push(
            "workspace.package.rust-version must match policy/clippy-lints.toml msrv".to_string(),
        );
    }
}

fn scalar_value(contents: &str, key: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let line = line.trim();
        if line.starts_with('#') || !line.starts_with(key) {
            return None;
        }
        let (_, value) = line.split_once('=')?;
        Some(value.trim().trim_matches('"').to_string())
    })
}

fn check_active_lints(cargo: &str, ledger: &str, findings: &mut Vec<String>) {
    if !cargo.contains("[workspace.lints.rust]") {
        findings.push("root Cargo.toml missing [workspace.lints.rust]".to_string());
    }
    if !cargo.contains("[workspace.lints.clippy]") {
        findings.push("root Cargo.toml missing [workspace.lints.clippy]".to_string());
    }

    let cargo_lints = cargo_workspace_lints(cargo);
    for entry in ledger_entries(ledger, "lint") {
        if entry.get("status").map(String::as_str) != Some("active") {
            continue;
        }
        let Some(name) = entry.get("name") else {
            findings.push("active lint entry missing name".to_string());
            continue;
        };
        let Some(level) = entry.get("level") else {
            findings.push(format!("active lint `{name}` missing level"));
            continue;
        };
        if entry.get("reason").is_none_or(String::is_empty) {
            findings.push(format!("active lint `{name}` missing reason"));
        }
        match cargo_lints.get(name) {
            Some(actual) if actual == level => {}
            Some(actual) => findings.push(format!(
                "active lint `{name}` ledger level `{level}` does not match Cargo.toml `{actual}`"
            )),
            None => findings.push(format!("active lint `{name}` missing from root Cargo.toml")),
        }
    }
}

fn cargo_workspace_lints(cargo: &str) -> BTreeMap<String, String> {
    let mut lints = BTreeMap::new();
    let mut prefix = None;
    for line in cargo.lines() {
        let trimmed = line.trim();
        match trimmed {
            "[workspace.lints.rust]" => {
                prefix = Some("rust");
                continue;
            }
            "[workspace.lints.clippy]" => {
                prefix = Some("clippy");
                continue;
            }
            _ if trimmed.starts_with('[') => {
                prefix = None;
                continue;
            }
            _ => {}
        }

        let Some(prefix) = prefix else { continue };
        if trimmed.starts_with('#') || !trimmed.contains('=') {
            continue;
        }
        if let Some((name, value)) = trimmed.split_once('=') {
            let level = value.trim().trim_matches('"').to_string();
            lints.insert(format!("{prefix}::{}", name.trim()), level);
        }
    }
    lints
}

fn check_planned_lints(cargo: &str, ledger: &str, findings: &mut Vec<String>) {
    let planned_entries = ledger_entries(ledger, "planned");
    let planned_names: BTreeSet<String> =
        planned_entries.iter().filter_map(|entry| entry.get("name").cloned()).collect();

    for (name, msrv) in REQUIRED_PLANNED {
        if !planned_names.contains(*name) {
            findings.push(format!("planned lint `{name}` for Rust {msrv} missing from ledger"));
        }
    }

    let cargo_lints = cargo_workspace_lints(cargo);
    for entry in planned_entries {
        let Some(name) = entry.get("name") else {
            findings.push("planned lint entry missing name".to_string());
            continue;
        };
        if entry.get("level").is_none_or(String::is_empty) {
            findings.push(format!("planned lint `{name}` missing level"));
        }
        if entry.get("activate_when_msrv").is_none_or(String::is_empty) {
            findings.push(format!("planned lint `{name}` missing activate_when_msrv"));
        }
        if entry.get("reason").is_none_or(String::is_empty) {
            findings.push(format!("planned lint `{name}` missing reason"));
        }
        if cargo_lints.contains_key(name) {
            findings.push(format!("planned lint `{name}` is active before the MSRV bump"));
        }
    }
}

fn ledger_entries(contents: &str, section: &str) -> Vec<BTreeMap<String, String>> {
    let header = format!("[[{section}]]");
    let mut entries = Vec::new();
    let mut current: Option<BTreeMap<String, String>> = None;

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[[") {
            if let Some(entry) = current.take() {
                entries.push(entry);
            }
            current = (trimmed == header).then(BTreeMap::new);
            continue;
        }
        let Some(entry) = current.as_mut() else { continue };
        if trimmed.starts_with('#') || !trimmed.contains('=') {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            entry.insert(key.trim().to_string(), value.trim().trim_matches('"').to_string());
        }
    }

    if let Some(entry) = current {
        entries.push(entry);
    }
    entries
}

fn check_workspace_members(root: &Path, cargo: &str, findings: &mut Vec<String>) -> Result<()> {
    let members = workspace_members(root, cargo)?;
    for member in members {
        let manifest_path = root.join(&member).join("Cargo.toml");
        let manifest = fs::read_to_string(&manifest_path)
            .with_context(|| format!("failed to read `{}`", manifest_path.display()))?;
        if !manifest.contains("[lints]") || !manifest.contains("workspace = true") {
            findings
                .push(format!("workspace member `{member}` must inherit [lints] workspace = true"));
        }
    }
    Ok(())
}

fn workspace_members(root: &Path, cargo: &str) -> Result<Vec<String>> {
    let mut members = Vec::new();
    if cargo.contains("members = [\"crates/*\"]") {
        for entry in fs::read_dir(root.join("crates")).context("failed to read crates directory")? {
            let entry = entry.context("failed to read crates directory entry")?;
            let path = entry.path();
            if path.join("Cargo.toml").exists() {
                members.push(format!("crates/{}", entry.file_name().to_string_lossy()));
            }
        }
    }
    members.sort();
    Ok(members)
}

fn check_no_test_carveouts(clippy: &str, findings: &mut Vec<String>) {
    for carveout in TEST_CARVEOUTS {
        if clippy.contains(carveout) {
            findings.push(format!("clippy.toml must not contain test carveout `{carveout}`"));
        }
    }
}

fn check_debt(debt: &str, findings: &mut Vec<String>) {
    for entry in ledger_entries(debt, "debt") {
        for required in ["lint", "path", "owner", "reason", "expires"] {
            if entry.get(required).is_none_or(String::is_empty) {
                findings.push(format!("clippy debt entry missing `{required}`"));
            }
        }
        if let Some(expires) = entry.get("expires") {
            if expires < &"2026-05-06".to_string() {
                findings.push(format!("clippy debt entry expired on {expires}"));
            }
        }
    }
}
