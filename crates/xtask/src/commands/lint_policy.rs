use anyhow::{Context, Result, bail};
use chrono::{NaiveDate, Utc};
use colored::Colorize;
use glob::Pattern;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value;
use walkdir::WalkDir;

const ROOT_CARGO: &str = "Cargo.toml";
const CLIPPY_LEDGER: &str = "policy/clippy-lints.toml";
const CLIPPY_DEBT: &str = "policy/clippy-debt.toml";
const NO_PANIC_ALLOWLIST: &str = "policy/no-panic-allowlist.toml";
const NON_RUST_ALLOWLIST: &str = "policy/non-rust-allowlist.toml";
const TEST_CARVEOUTS: &[&str] = &[
    "allow-unwrap-in-tests",
    "allow-expect-in-tests",
    "allow-panic-in-tests",
    "allow-indexing-slicing-in-tests",
    "allow-dbg-in-tests",
];
const PANIC_FAMILIES: &[&str] =
    &["unwrap", "expect", "panic", "todo", "unimplemented", "unreachable"];

#[derive(Debug, Deserialize)]
struct ClippyLedger {
    msrv: String,
    policy: LedgerPolicy,
    #[serde(default)]
    lint: Vec<LedgerLint>,
}

#[derive(Debug, Deserialize)]
struct LedgerPolicy {
    panic_free_tests: bool,
    allow_test_carveouts: bool,
    suppression_style: String,
    blanket_categories: bool,
}

#[derive(Debug, Deserialize)]
struct LedgerLint {
    name: String,
    level: String,
    status: String,
    #[serde(default)]
    activate_when_msrv: Option<String>,
    #[serde(default)]
    class: Option<String>,
    reason: String,
}

#[derive(Debug, Deserialize)]
struct ClippyDebtLedger {
    schema: u64,
    #[serde(default)]
    debt: Vec<DebtEntry>,
}

#[derive(Debug, Deserialize)]
struct DebtEntry {
    lint: String,
    path: String,
    owner: String,
    reason: String,
    expires: String,
}

#[derive(Debug, Deserialize)]
struct NoPanicAllowlist {
    schema_version: String,
    #[serde(default)]
    allow: Vec<PanicAllow>,
}

#[derive(Debug, Deserialize)]
struct PanicAllow {
    path: String,
    family: String,
    classification: String,
    owner: String,
    explanation: String,
    #[serde(default)]
    expires: Option<String>,
    selector: PanicSelector,
    #[serde(default)]
    last_seen: Option<LastSeen>,
}

#[derive(Debug, Deserialize)]
struct PanicSelector {
    kind: String,
    #[serde(default)]
    container: Option<String>,
    #[serde(default)]
    callee: Option<String>,
    #[serde(default)]
    receiver_fingerprint: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LastSeen {
    line: usize,
    column: usize,
}

#[derive(Debug, Deserialize)]
struct NonRustAllowlist {
    schema_version: String,
    #[serde(default)]
    allow: Vec<NonRustAllow>,
}

#[derive(Debug, Deserialize)]
struct NonRustAllow {
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    glob: Option<String>,
    kind: String,
    owner: String,
    reason: String,
    surface: String,
    classification: String,
    #[serde(default)]
    covered_by: Vec<String>,
    #[serde(default)]
    expires: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct PanicOccurrence {
    path: String,
    line: usize,
    column: usize,
    family: String,
    selector_kind: String,
    callee: String,
}

pub fn check_lint_policy() -> Result<()> {
    let root = repo_root()?;
    let mut errors = Vec::new();

    let cargo_text = read_to_string(&root.join(ROOT_CARGO), &mut errors);
    let cargo = parse_toml(&cargo_text, ROOT_CARGO, &mut errors);
    let ledger: Option<ClippyLedger> = read_toml(&root.join(CLIPPY_LEDGER), &mut errors);
    let debt: Option<ClippyDebtLedger> = read_toml(&root.join(CLIPPY_DEBT), &mut errors);

    if let (Some(cargo), Some(ledger)) = (&cargo, &ledger) {
        check_msrv(cargo, ledger, &mut errors);
        check_ledger_policy(ledger, &mut errors);
        check_active_lints(cargo, ledger, &mut errors);
        check_planned_lints(cargo, ledger, &mut errors);
    }

    check_workspace_lint_inheritance(&root, &mut errors)?;
    check_clippy_toml(&root, &mut errors);

    if let Some(debt) = &debt {
        check_debt(debt, &mut errors);
        check_allow_suppressions(&root, debt, &mut errors)?;
    }

    finish("lint policy", errors)
}

pub fn check_no_panic_family() -> Result<()> {
    let root = repo_root()?;
    let mut errors = Vec::new();
    let allowlist: Option<NoPanicAllowlist> =
        read_toml(&root.join(NO_PANIC_ALLOWLIST), &mut errors);
    let occurrences = find_panic_occurrences(&root)?;

    if let Some(allowlist) = &allowlist {
        if allowlist.schema_version.trim().is_empty() {
            errors.push("policy/no-panic-allowlist.toml: schema_version is required".to_string());
        }
        let today = Utc::now().date_naive();
        let mut allowed = BTreeSet::new();
        for entry in &allowlist.allow {
            validate_panic_allow(entry, today, &mut errors);
            allowed.insert(panic_identity(entry));
        }

        let mut seen = BTreeSet::new();
        for occurrence in &occurrences {
            let identity = occurrence_identity(occurrence);
            if allowed.contains(&identity) {
                seen.insert(identity);
            } else {
                errors.push(format!(
                    "{}:{}:{}: unallowlisted panic-family `{}` occurrence",
                    occurrence.path, occurrence.line, occurrence.column, occurrence.family
                ));
            }
        }

        for identity in allowed.difference(&seen) {
            errors.push(format!("{}: stale no-panic allowlist selector", identity));
        }
    }

    finish("no-panic family", errors)
}

pub fn check_file_policy() -> Result<()> {
    let root = repo_root()?;
    let mut errors = Vec::new();
    let allowlist: Option<NonRustAllowlist> =
        read_toml(&root.join(NON_RUST_ALLOWLIST), &mut errors);
    let files = find_non_rust_programming_files(&root)?;

    if let Some(allowlist) = &allowlist {
        if allowlist.schema_version.trim().is_empty() {
            errors.push("policy/non-rust-allowlist.toml: schema_version is required".to_string());
        }
        let today = Utc::now().date_naive();
        let compiled = compile_non_rust_allows(allowlist, today, &mut errors);
        for file in files {
            let rel = rel_path(&root, &file);
            if !compiled.iter().any(|allow| allow.matches(&rel)) {
                errors.push(format!("{rel}: non-Rust programming file lacks policy entry"));
            }
        }
    }

    finish("file policy", errors)
}

pub fn policy_report() -> Result<()> {
    let root = repo_root()?;
    let ledger: ClippyLedger = read_toml_required(&root.join(CLIPPY_LEDGER))?;
    let debt: ClippyDebtLedger = read_toml_required(&root.join(CLIPPY_DEBT))?;
    let panic_allow: NoPanicAllowlist = read_toml_required(&root.join(NO_PANIC_ALLOWLIST))?;
    let non_rust: NonRustAllowlist = read_toml_required(&root.join(NON_RUST_ALLOWLIST))?;

    let active = ledger.lint.iter().filter(|lint| lint.status == "active").count();
    let planned = ledger.lint.iter().filter(|lint| lint.status == "planned").count();
    let expired_debt = debt.debt.iter().filter(|entry| is_expired(&entry.expires)).count();
    let expired_panic = panic_allow
        .allow
        .iter()
        .filter(|entry| entry.expires.as_deref().is_some_and(is_expired))
        .count();
    let expired_non_rust = non_rust
        .allow
        .iter()
        .filter(|entry| entry.expires.as_deref().is_some_and(is_expired))
        .count();

    println!("{}", "Policy report".bold());
    println!("clippy lints: {active} active, {planned} planned");
    println!("clippy debt: {} active, {expired_debt} expired", debt.debt.len());
    println!("panic exceptions: {} active, {expired_panic} expired", panic_allow.allow.len());
    println!("non-rust exceptions: {} active, {expired_non_rust} expired", non_rust.allow.len());
    Ok(())
}

fn check_msrv(cargo: &Value, ledger: &ClippyLedger, errors: &mut Vec<String>) {
    let rust_version = cargo
        .get("workspace")
        .and_then(|v| v.get("package"))
        .and_then(|v| v.get("rust-version"))
        .and_then(Value::as_str);
    if rust_version != Some(ledger.msrv.as_str()) {
        errors.push(format!(
            "workspace.package.rust-version must equal policy msrv {}; found {:?}",
            ledger.msrv, rust_version
        ));
    }
}

fn check_ledger_policy(ledger: &ClippyLedger, errors: &mut Vec<String>) {
    if !ledger.policy.panic_free_tests {
        errors.push("policy/clippy-lints.toml: panic_free_tests must be true".to_string());
    }
    if ledger.policy.allow_test_carveouts {
        errors.push("policy/clippy-lints.toml: allow_test_carveouts must be false".to_string());
    }
    if ledger.policy.suppression_style != "expect-with-reason" {
        errors.push(
            "policy/clippy-lints.toml: suppression_style must be expect-with-reason".to_string(),
        );
    }
    if ledger.policy.blanket_categories {
        errors.push("policy/clippy-lints.toml: blanket_categories must be false".to_string());
    }
    for lint in &ledger.lint {
        if lint.reason.trim().is_empty() {
            errors.push(format!("{}: lint reason is required", lint.name));
        }
        if lint.class.as_deref().unwrap_or_default().trim().is_empty() {
            errors.push(format!("{}: lint class is required", lint.name));
        }
    }
}

fn check_active_lints(cargo: &Value, ledger: &ClippyLedger, errors: &mut Vec<String>) {
    let root_lints = collect_workspace_lints(cargo);
    for lint in ledger.lint.iter().filter(|lint| lint.status == "active") {
        match root_lints.get(&lint.name) {
            Some(level) if level == &lint.level => {}
            Some(level) => errors.push(format!(
                "{}: ledger level `{}` does not match Cargo.toml level `{}`",
                lint.name, lint.level, level
            )),
            None => {
                errors.push(format!("{}: active ledger lint missing from Cargo.toml", lint.name))
            }
        }
    }
}

fn check_planned_lints(cargo: &Value, ledger: &ClippyLedger, errors: &mut Vec<String>) {
    let root_lints = collect_workspace_lints(cargo);
    for lint in ledger.lint.iter().filter(|lint| lint.status == "planned") {
        let Some(activate_when_msrv) = &lint.activate_when_msrv else {
            errors.push(format!("{}: planned lint missing activate_when_msrv", lint.name));
            continue;
        };
        if version_lt(&ledger.msrv, activate_when_msrv) && root_lints.contains_key(&lint.name) {
            errors.push(format!(
                "{}: planned for MSRV {}, but already active at MSRV {}",
                lint.name, activate_when_msrv, ledger.msrv
            ));
        }
    }
}

fn collect_workspace_lints(cargo: &Value) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let Some(lints) = cargo.get("workspace").and_then(|v| v.get("lints")) else {
        return out;
    };
    for section in ["rust", "clippy"] {
        if let Some(table) = lints.get(section).and_then(Value::as_table) {
            for (key, value) in table {
                let name = if section == "clippy" { format!("clippy::{key}") } else { key.clone() };
                if let Some(level) = value.as_str() {
                    out.insert(name, level.to_string());
                } else if let Some(level) = value.get("level").and_then(Value::as_str) {
                    out.insert(name, level.to_string());
                }
            }
        }
    }
    out
}

fn check_workspace_lint_inheritance(root: &Path, errors: &mut Vec<String>) -> Result<()> {
    for manifest in workspace_member_manifests(root)? {
        let text = fs::read_to_string(&manifest)
            .with_context(|| format!("reading {}", manifest.display()))?;
        if !text.contains("[lints]") || !text.contains("workspace = true") {
            errors.push(format!(
                "{}: workspace member must inherit `[lints] workspace = true`",
                rel_path(root, &manifest)
            ));
        }
    }
    Ok(())
}

fn workspace_member_manifests(root: &Path) -> Result<Vec<PathBuf>> {
    let crates = root.join("crates");
    let mut manifests = Vec::new();
    for entry in fs::read_dir(&crates).with_context(|| format!("reading {}", crates.display()))? {
        let entry = entry?;
        let manifest = entry.path().join("Cargo.toml");
        if manifest.exists() {
            manifests.push(manifest);
        }
    }
    manifests.sort();
    Ok(manifests)
}

fn check_clippy_toml(root: &Path, errors: &mut Vec<String>) {
    let clippy_toml = root.join("clippy.toml");
    let Ok(text) = fs::read_to_string(&clippy_toml) else {
        errors.push("clippy.toml is required for repo-specific disallowed policy".to_string());
        return;
    };
    for carveout in TEST_CARVEOUTS {
        if text.contains(carveout) {
            errors.push(format!("clippy.toml: test carveout `{carveout}` is forbidden"));
        }
    }
}

fn check_debt(debt: &ClippyDebtLedger, errors: &mut Vec<String>) {
    if debt.schema != 1 {
        errors.push(format!("policy/clippy-debt.toml: schema must be 1, found {}", debt.schema));
    }
    let today = Utc::now().date_naive();
    for entry in &debt.debt {
        if entry.lint.trim().is_empty() {
            errors.push("policy/clippy-debt.toml: debt lint is required".to_string());
        }
        if entry.path.trim().is_empty() {
            errors.push(format!("{}: debt path is required", entry.lint));
        }
        if entry.owner.trim().is_empty() {
            errors.push(format!("{} {}: debt owner is required", entry.path, entry.lint));
        }
        if entry.reason.trim().is_empty() {
            errors.push(format!("{} {}: debt reason is required", entry.path, entry.lint));
        }
        match NaiveDate::parse_from_str(&entry.expires, "%Y-%m-%d") {
            Ok(expires) if expires < today => {
                errors.push(format!(
                    "{} {}: debt expired on {}",
                    entry.path, entry.lint, entry.expires
                ));
            }
            Ok(_) => {}
            Err(_) => errors
                .push(format!("{} {}: debt expires must use YYYY-MM-DD", entry.path, entry.lint)),
        }
    }
}

fn check_allow_suppressions(
    root: &Path,
    debt: &ClippyDebtLedger,
    errors: &mut Vec<String>,
) -> Result<()> {
    let debt_keys: BTreeSet<_> =
        debt.debt.iter().map(|entry| (entry.path.as_str(), entry.lint.as_str())).collect();
    for file in rust_files(root)? {
        let rel = rel_path(root, &file);
        let text = fs::read_to_string(&file).with_context(|| format!("reading {rel}"))?;
        let lines: Vec<_> = text.lines().collect();
        let mut idx = 0;
        while idx < lines.len() {
            let line = lines[idx];
            if let Some(lints) = parse_attr_lints(line, "allow") {
                for lint in lints {
                    if !debt_keys.contains(&(rel.as_str(), lint.as_str())) {
                        errors.push(format!(
                            "{rel}:{}: #[allow({lint})] requires policy/clippy-debt.toml entry",
                            idx + 1
                        ));
                    }
                }
            }
            if is_attr_start(line, "expect") {
                let start = idx;
                let mut attr = String::from(line);
                while !attr.contains(']') && idx + 1 < lines.len() {
                    idx += 1;
                    attr.push_str(lines[idx]);
                }
                if !attr.contains("reason") {
                    errors.push(format!(
                        "{rel}:{}: #[expect] must include an explicit reason",
                        start + 1
                    ));
                }
            }
            idx += 1;
        }
    }
    Ok(())
}

fn parse_attr_lints(line: &str, attr: &str) -> Option<Vec<String>> {
    if !is_attr_start(line, attr) {
        return None;
    }
    let rest = line.trim_start();
    let open = rest.find('(')?;
    let close = rest[open + 1..].find(')')? + open + 1;
    Some(
        rest[open + 1..close]
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect(),
    )
}

fn is_attr_start(line: &str, attr: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with(&format!("#[{attr}")) || trimmed.starts_with(&format!("#![{attr}"))
}

fn validate_panic_allow(entry: &PanicAllow, today: NaiveDate, errors: &mut Vec<String>) {
    if entry.path.trim().is_empty()
        || entry.family.trim().is_empty()
        || entry.owner.trim().is_empty()
    {
        errors.push(
            "policy/no-panic-allowlist.toml: path, family, and owner are required".to_string(),
        );
    }
    if entry.classification.trim().is_empty() || entry.explanation.trim().is_empty() {
        errors.push(format!(
            "{} {}: classification and explanation are required",
            entry.path, entry.family
        ));
    }
    if !PANIC_FAMILIES.contains(&entry.family.as_str()) {
        errors.push(format!("{}: unknown panic family `{}`", entry.path, entry.family));
    }
    if entry.selector.kind.trim().is_empty() {
        errors.push(format!("{} {}: selector.kind is required", entry.path, entry.family));
    }
    let _ = (&entry.selector.container, &entry.selector.receiver_fingerprint);
    if let Some(last_seen) = &entry.last_seen {
        let _ = (last_seen.line, last_seen.column);
    }
    if let Some(expires) = &entry.expires {
        match NaiveDate::parse_from_str(expires, "%Y-%m-%d") {
            Ok(date) if date < today => {
                errors.push(format!(
                    "{} {}: panic allow expired on {expires}",
                    entry.path, entry.family
                ));
            }
            Ok(_) => {}
            Err(_) => {
                errors.push(format!("{} {}: expires must use YYYY-MM-DD", entry.path, entry.family))
            }
        }
    }
}

fn find_panic_occurrences(root: &Path) -> Result<Vec<PanicOccurrence>> {
    let mut occurrences = Vec::new();
    for file in rust_files(root)? {
        let rel = rel_path(root, &file);
        let text = fs::read_to_string(&file).with_context(|| format!("reading {rel}"))?;
        for (idx, line) in text.lines().enumerate() {
            for family in PANIC_FAMILIES {
                let method = format!(".{family}(");
                let mac = format!("{family}!(");
                if let Some(col) = line.find(&method) {
                    occurrences.push(PanicOccurrence {
                        path: rel.clone(),
                        line: idx + 1,
                        column: col + 1,
                        family: (*family).to_string(),
                        selector_kind: "method_call".to_string(),
                        callee: (*family).to_string(),
                    });
                }
                if let Some(col) = line.find(&mac) {
                    occurrences.push(PanicOccurrence {
                        path: rel.clone(),
                        line: idx + 1,
                        column: col + 1,
                        family: (*family).to_string(),
                        selector_kind: "macro_call".to_string(),
                        callee: (*family).to_string(),
                    });
                }
            }
        }
    }
    Ok(occurrences)
}

fn find_non_rust_programming_files(root: &Path) -> Result<Vec<PathBuf>> {
    let extensions = ["py", "sh", "js", "ts", "tsx", "jsx", "go", "rb", "java", "kt", "swift"];
    let mut files = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_entry(|entry| !is_ignored(entry.path())) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()).is_some_and(|ext| extensions.contains(&ext)) {
            files.push(path.to_path_buf());
        }
    }
    files.sort();
    Ok(files)
}

struct CompiledNonRustAllow<'a> {
    entry: &'a NonRustAllow,
    pattern: Option<Pattern>,
}

impl<'a> CompiledNonRustAllow<'a> {
    fn matches(&self, rel: &str) -> bool {
        if self.entry.path.as_deref() == Some(rel) {
            return true;
        }
        self.pattern.as_ref().is_some_and(|pattern| pattern.matches(rel))
    }
}

fn compile_non_rust_allows<'a>(
    allowlist: &'a NonRustAllowlist,
    today: NaiveDate,
    errors: &mut Vec<String>,
) -> Vec<CompiledNonRustAllow<'a>> {
    let mut compiled = Vec::new();
    for entry in &allowlist.allow {
        if entry.path.is_none() && entry.glob.is_none() {
            errors
                .push("policy/non-rust-allowlist.toml: each entry needs path or glob".to_string());
        }
        for (field, value) in [
            ("kind", &entry.kind),
            ("owner", &entry.owner),
            ("reason", &entry.reason),
            ("surface", &entry.surface),
            ("classification", &entry.classification),
        ] {
            if value.trim().is_empty() {
                errors.push(format!("policy/non-rust-allowlist.toml: {field} is required"));
            }
        }
        if ["production", "test", "tooling"].contains(&entry.classification.as_str())
            && entry.covered_by.is_empty()
        {
            errors.push(format!(
                "policy/non-rust-allowlist.toml: {} requires covered_by",
                entry.path.as_deref().or(entry.glob.as_deref()).unwrap_or("<unknown>")
            ));
        }
        if let Some(expires) = &entry.expires {
            match NaiveDate::parse_from_str(expires, "%Y-%m-%d") {
                Ok(date) if date < today => errors.push(format!(
                    "policy/non-rust-allowlist.toml: {} expired on {expires}",
                    entry.path.as_deref().or(entry.glob.as_deref()).unwrap_or("<unknown>")
                )),
                Ok(_) => {}
                Err(_) => errors.push(
                    "policy/non-rust-allowlist.toml: expires must use YYYY-MM-DD".to_string(),
                ),
            }
        }
        let pattern = match &entry.glob {
            Some(glob) => match Pattern::new(glob) {
                Ok(pattern) => Some(pattern),
                Err(err) => {
                    errors.push(format!(
                        "policy/non-rust-allowlist.toml: invalid glob `{glob}`: {err}"
                    ));
                    None
                }
            },
            None => None,
        };
        compiled.push(CompiledNonRustAllow { entry, pattern });
    }
    compiled
}

fn panic_identity(entry: &PanicAllow) -> String {
    format!(
        "{}|{}|{}|{}",
        entry.path,
        entry.family,
        entry.selector.kind,
        entry.selector.callee.as_deref().unwrap_or("")
    )
}

fn occurrence_identity(occurrence: &PanicOccurrence) -> String {
    format!(
        "{}|{}|{}|{}",
        occurrence.path, occurrence.family, occurrence.selector_kind, occurrence.callee
    )
}

fn rust_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_entry(|entry| !is_ignored(entry.path())) {
        let entry = entry?;
        if entry.file_type().is_file()
            && entry.path().extension().and_then(|e| e.to_str()) == Some("rs")
        {
            files.push(entry.path().to_path_buf());
        }
    }
    files.sort();
    Ok(files)
}

fn is_ignored(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| matches!(name, ".git" | "target" | ".direnv" | "node_modules"))
}

fn read_to_string(path: &Path, errors: &mut Vec<String>) -> String {
    match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) => {
            errors.push(format!("{}: {err}", path.display()));
            String::new()
        }
    }
}

fn parse_toml(text: &str, label: &str, errors: &mut Vec<String>) -> Option<Value> {
    match toml::from_str(text) {
        Ok(value) => Some(value),
        Err(err) => {
            errors.push(format!("{label}: invalid TOML: {err}"));
            None
        }
    }
}

fn read_toml<T: for<'de> Deserialize<'de>>(path: &Path, errors: &mut Vec<String>) -> Option<T> {
    let text = read_to_string(path, errors);
    match toml::from_str(&text) {
        Ok(value) => Some(value),
        Err(err) => {
            errors.push(format!("{}: invalid TOML: {err}", path.display()));
            None
        }
    }
}

fn read_toml_required<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}

fn rel_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

fn repo_root() -> Result<PathBuf> {
    std::env::current_dir().context("resolving current directory")
}

fn finish(label: &str, errors: Vec<String>) -> Result<()> {
    if errors.is_empty() {
        println!("{} {label} checks passed", "✓".green());
        Ok(())
    } else {
        eprintln!("{} {label} checks failed:", "✗".red());
        for error in &errors {
            eprintln!("  - {error}");
        }
        bail!("{label} policy check failed with {} error(s)", errors.len())
    }
}

fn version_lt(left: &str, right: &str) -> bool {
    parse_version(left) < parse_version(right)
}

fn parse_version(version: &str) -> (u64, u64, u64) {
    let mut parts = version.split('.').map(|part| part.parse::<u64>().unwrap_or(0));
    (parts.next().unwrap_or(0), parts.next().unwrap_or(0), parts.next().unwrap_or(0))
}

fn is_expired(date: &str) -> bool {
    NaiveDate::parse_from_str(date, "%Y-%m-%d").is_ok_and(|date| date < Utc::now().date_naive())
}
