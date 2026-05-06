use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const POLICY: &str = "policy/clippy-lints.toml";
const DEBT: &str = "policy/clippy-debt.toml";
const CLIPPY: &str = "clippy.toml";

const TEST_CARVEOUTS: &[&str] = &[
    "allow-unwrap-in-tests",
    "allow-expect-in-tests",
    "allow-panic-in-tests",
    "allow-indexing-slicing-in-tests",
    "allow-dbg-in-tests",
];

const PLANNED_FLIPS: &[(&str, &str)] = &[
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

#[derive(Debug, Clone, Default)]
struct LintEntry {
    name: String,
    level: String,
    status: String,
    activate_when_msrv: Option<String>,
}

pub fn run() -> Result<()> {
    let root = Path::new(".");
    let cargo = read(root.join("Cargo.toml"))?;
    let policy = read(root.join(POLICY))?;
    let debt = read(root.join(DEBT))?;
    let clippy = read(root.join(CLIPPY))?;

    let mut failures = Vec::new();

    let workspace_msrv = field_value(&cargo, "rust-version").unwrap_or_default();
    let policy_msrv = field_value(&policy, "msrv").unwrap_or_default();
    if normalize_version(&workspace_msrv) != normalize_version(&policy_msrv) {
        failures.push(format!(
            "workspace.package.rust-version ({workspace_msrv}) must match {POLICY} msrv ({policy_msrv})"
        ));
    }

    if policy_value(&policy, "panic_free_tests") != Some("true".to_string()) {
        failures.push("policy.panic_free_tests must be true".to_string());
    }
    if policy_value(&policy, "allow_test_carveouts") != Some("false".to_string()) {
        failures.push("policy.allow_test_carveouts must be false".to_string());
    }
    if policy_value(&policy, "suppression_style") != Some("expect-with-reason".to_string()) {
        failures.push("policy.suppression_style must be expect-with-reason".to_string());
    }
    if policy_value(&policy, "blanket_categories") != Some("false".to_string()) {
        failures.push("policy.blanket_categories must be false".to_string());
    }

    check_workspace_lints(&cargo, &policy, &workspace_msrv, &mut failures);
    check_member_inheritance(root, &mut failures)?;
    check_clippy_toml(&clippy, &mut failures);
    check_debt(&debt, &mut failures);
    check_suppressions(root, &mut failures)?;

    if failures.is_empty() {
        println!(
            "lint policy checked: MSRV, inheritance, ledger, carveouts, debt, and suppressions are coherent"
        );
        Ok(())
    } else {
        for failure in &failures {
            eprintln!("lint policy violation: {failure}");
        }
        bail!("lint policy check failed with {} violation(s)", failures.len())
    }
}

fn check_workspace_lints(
    cargo: &str,
    policy: &str,
    workspace_msrv: &str,
    failures: &mut Vec<String>,
) {
    let cargo_lints = workspace_lints(cargo);
    let entries = lint_entries(policy);

    for entry in entries.values().filter(|entry| entry.status == "active") {
        match cargo_lints.get(&entry.name) {
            Some(level) if level == &entry.level => {}
            Some(level) => failures.push(format!(
                "active lint {} has Cargo level {level}, expected {}",
                entry.name, entry.level
            )),
            None => {
                failures.push(format!("active lint {} is missing from root Cargo.toml", entry.name))
            }
        }
    }

    for (name, msrv) in PLANNED_FLIPS {
        match entries.get(*name) {
            Some(entry) if entry.status == "planned" => {
                if entry.activate_when_msrv.as_deref() != Some(*msrv) {
                    failures.push(format!("planned lint {name} must activate when MSRV is {msrv}"));
                }
            }
            Some(entry) => failures
                .push(format!("planned lint {name} has status {}, expected planned", entry.status)),
            None => failures.push(format!("planned lint {name} is missing from {POLICY}")),
        }
    }

    for (name, _) in PLANNED_FLIPS {
        if cargo_lints.contains_key(*name) && version_lt(workspace_msrv, "1.94") {
            failures.push(format!(
                "planned lint {name} is active before the Rust 1.94/1.95 MSRV ratchet"
            ));
        }
    }
}

fn check_member_inheritance(root: &Path, failures: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(root.join("crates")).context("read crates directory")? {
        let path = entry?.path().join("Cargo.toml");
        if !path.exists() {
            continue;
        }
        let manifest = read(&path)?;
        if !manifest.contains("[lints]") || !manifest.contains("workspace = true") {
            failures.push(format!(
                "{} must inherit workspace lints with [lints] workspace = true",
                path.display()
            ));
        }
    }
    Ok(())
}

fn check_clippy_toml(clippy: &str, failures: &mut Vec<String>) {
    for carveout in TEST_CARVEOUTS {
        if clippy.contains(carveout) {
            failures.push(format!("{CLIPPY} must not set test carveout {carveout}"));
        }
    }
}

fn check_debt(debt: &str, failures: &mut Vec<String>) {
    for block in table_blocks(debt, "[[debt]]") {
        for key in ["lint", "path", "owner", "reason", "expires"] {
            if field_value(block, key).is_none() {
                failures.push(format!("{DEBT} debt entry missing {key}"));
            }
        }
        if let Some(expires) = field_value(block, "expires") {
            match NaiveDate::parse_from_str(&expires, "%Y-%m-%d") {
                Ok(date) if date < chrono::Utc::now().date_naive() => {
                    failures.push(format!("{DEBT} debt entry expired on {expires}"));
                }
                Ok(_) => {}
                Err(_) => {
                    failures.push(format!("{DEBT} debt entry has invalid expires date {expires}"))
                }
            }
        }
    }
}

fn check_suppressions(root: &Path, failures: &mut Vec<String>) -> Result<()> {
    for entry in WalkDir::new(root).into_iter().filter_entry(|entry| !is_ignored(entry.path())) {
        let entry = entry?;
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|ext| ext.to_str()) != Some("rs")
            || entry.path().ends_with("crates/xtask/src/commands/check_lint_policy.rs")
        {
            continue;
        }
        let source = read(entry.path())?;
        for (idx, line) in source.lines().enumerate() {
            if line.contains("#[allow") {
                failures.push(format!(
                    "{}:{} uses #[allow]; use #[expect(..., reason = \"...\")] or policy debt",
                    entry.path().display(),
                    idx + 1
                ));
            }
            if line.contains("#[expect") {
                let attribute = source.lines().skip(idx).take(8).collect::<Vec<_>>().join(" ");
                if !attribute.contains("reason =") {
                    failures.push(format!(
                        "{}:{} uses #[expect] without reason",
                        entry.path().display(),
                        idx + 1
                    ));
                }
            }
        }
    }
    Ok(())
}

fn read(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    fs::read_to_string(path).with_context(|| format!("read {}", path.display()))
}

fn is_ignored(path: &Path) -> bool {
    path.components().any(|component| {
        let text = component.as_os_str().to_string_lossy();
        matches!(text.as_ref(), ".git" | "target")
    })
}

fn normalize_version(version: &str) -> String {
    version.trim_end_matches(".0").to_string()
}

fn version_lt(left: &str, right: &str) -> bool {
    let parse = |value: &str| -> Vec<u32> {
        value.split('.').map(|part| part.parse::<u32>().unwrap_or(0)).collect()
    };
    parse(left) < parse(right)
}

fn workspace_lints(cargo: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let mut section = None;
    for line in cargo.lines() {
        match line.trim() {
            "[workspace.lints.rust]" => section = Some("rust"),
            "[workspace.lints.clippy]" => section = Some("clippy"),
            text if text.starts_with('[') => section = None,
            _ => {}
        }
        let Some(section) = section else { continue };
        let Some((key, value)) = line.split_once('=') else { continue };
        let key = key.trim();
        if key.is_empty() || key.starts_with('#') {
            continue;
        }
        let Some(level) = quoted_value(value).or_else(|| inline_level(value)) else { continue };
        let name = if section == "clippy" { format!("clippy::{key}") } else { key.to_string() };
        map.insert(name, level);
    }
    map
}

fn lint_entries(policy: &str) -> BTreeMap<String, LintEntry> {
    let mut entries = BTreeMap::new();
    for block in table_blocks(policy, "[[lint]]") {
        let Some(name) = field_value(block, "name") else { continue };
        entries.insert(
            name.clone(),
            LintEntry {
                name,
                level: field_value(block, "level").unwrap_or_default(),
                status: field_value(block, "status").unwrap_or_default(),
                activate_when_msrv: field_value(block, "activate_when_msrv"),
            },
        );
    }
    entries
}

fn table_blocks<'a>(text: &'a str, marker: &str) -> Vec<&'a str> {
    text.split(marker).skip(1).collect()
}

fn policy_value(text: &str, key: &str) -> Option<String> {
    let mut in_policy = false;
    for line in text.lines() {
        match line.trim() {
            "[policy]" => in_policy = true,
            value if value.starts_with('[') => in_policy = false,
            _ => {}
        }
        if in_policy {
            if let Some(value) = raw_field_value(line, key) {
                return Some(value);
            }
        }
    }
    None
}

fn field_value(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|line| {
        raw_field_value(line, key).and_then(
            |value| {
                if value.starts_with('[') { None } else { Some(value) }
            },
        )
    })
}

fn raw_field_value(line: &str, key: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.starts_with('#') {
        return None;
    }
    let (left, right) = trimmed.split_once('=')?;
    if left.trim() != key {
        return None;
    }
    Some(quoted_value(right).unwrap_or_else(|| right.trim().trim_end_matches(',').to_string()))
}

fn quoted_value(value: &str) -> Option<String> {
    let start = value.find('"')? + 1;
    let end = value[start..].find('"')? + start;
    Some(value[start..end].to_string())
}

fn inline_level(value: &str) -> Option<String> {
    value.split(',').find_map(|part| {
        let (key, value) = part.split_once('=')?;
        if key.trim().trim_start_matches('{').trim() == "level" {
            quoted_value(value)
        } else {
            None
        }
    })
}
