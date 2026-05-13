use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};

pub fn run() -> Result<()> {
    let root = workspace_root_path()?;
    let policy_path = root.join("policy/non-rust-allowlist.toml");
    let text = fs::read_to_string(&policy_path)
        .with_context(|| format!("failed to read {}", policy_path.display()))?;

    let entries = parse_allow_entries(&text);
    if entries.is_empty() {
        bail!("{} contains no [[allow]] entries", policy_path.display());
    }

    let mut failures = Vec::new();
    for entry in &entries {
        if let Some(path) = &entry.path {
            if !root.join(path).exists() {
                failures.push(format!("allowlist path does not exist: {path}"));
            }
        }

        if let Some(glob) = &entry.glob {
            let prefix = glob.trim_end_matches("*.json").trim_end_matches('*');
            if !root.join(prefix).exists() {
                failures.push(format!("allowlist glob directory does not exist: {glob}"));
            }
        }

        for required in ["kind", "owner", "surface", "classification", "reason"] {
            if !entry.keys.iter().any(|key| key == required) {
                failures.push(format!("allowlist entry missing `{required}`: {:?}", entry.label()));
            }
        }
    }

    if !failures.is_empty() {
        bail!("file policy check failed:\n{}", failures.join("\n"));
    }

    println!("check-file-policy: {} allowlist entries are valid", entries.len());
    Ok(())
}

#[derive(Default, Debug)]
struct AllowEntry {
    path: Option<String>,
    glob: Option<String>,
    keys: Vec<String>,
}

impl AllowEntry {
    fn label(&self) -> Option<&str> {
        self.path.as_deref().or(self.glob.as_deref())
    }
}

fn parse_allow_entries(text: &str) -> Vec<AllowEntry> {
    let mut entries = Vec::new();
    let mut current: Option<AllowEntry> = None;

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line == "[[allow]]" {
            if let Some(entry) = current.take() {
                entries.push(entry);
            }
            current = Some(AllowEntry::default());
            continue;
        }

        let Some(entry) = current.as_mut() else {
            continue;
        };
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim().to_string();
        let value = value.trim().trim_matches('"').to_string();

        match key.as_str() {
            "path" => entry.path = Some(value),
            "glob" => entry.glob = Some(value),
            _ => {}
        }
        entry.keys.push(key);
    }

    if let Some(entry) = current.take() {
        entries.push(entry);
    }

    entries
}

fn workspace_root_path() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("failed to resolve workspace root from CARGO_MANIFEST_DIR")
}
