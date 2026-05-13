//! Generated public badge endpoint support.

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BADGE_ENDPOINT_DIR: &str = "badges";
const BADGE_ENDPOINT_TARGET_DIR: &str = "target/xtask/badges";

/// Minimal Shields endpoint JSON used by public README badges.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ShieldsEndpointBadge {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u8,
    pub label: String,
    pub message: String,
    pub color: String,
}

/// Generate or check committed Shields endpoint badge JSON.
pub fn run(check: bool) -> Result<()> {
    let workspace_root = workspace_root_path()?;
    let target_dir = workspace_root.join(BADGE_ENDPOINT_TARGET_DIR);

    fs::create_dir_all(&target_dir).with_context(|| {
        format!("failed to create generated badge target dir {}", target_dir.display())
    })?;

    let ripr_plus = ripr_plus_badge(&workspace_root)?;
    validate_shields_badge(&ripr_plus, Some("ripr+"))?;
    write_json_pretty(&target_dir.join("ripr-plus.json"), &ripr_plus)?;

    if check {
        let committed_dir = workspace_root.join(BADGE_ENDPOINT_DIR);
        compare_files(&committed_dir.join("ripr-plus.json"), &target_dir.join("ripr-plus.json"))?;
        println!("badges: committed endpoints are current");
        return Ok(());
    }

    let committed_dir = workspace_root.join(BADGE_ENDPOINT_DIR);
    fs::create_dir_all(&committed_dir).with_context(|| {
        format!("failed to create committed badge dir {}", committed_dir.display())
    })?;
    fs::copy(target_dir.join("ripr-plus.json"), committed_dir.join("ripr-plus.json"))
        .context("failed to refresh badges/ripr-plus.json")?;

    println!("badges: refreshed public endpoint JSON under badges/");
    Ok(())
}

fn workspace_root_path() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("could not determine workspace root from CARGO_MANIFEST_DIR")
}

fn ripr_plus_badge(workspace_root: &Path) -> Result<ShieldsEndpointBadge> {
    let ripr_bin = std::env::var("RIPR_BIN").unwrap_or_else(|_| "ripr".to_string());

    // Public README badge: repo-scoped, not PR/diff scoped.
    let output = Command::new(&ripr_bin)
        .arg("check")
        .arg("--root")
        .arg(workspace_root)
        .arg("--format")
        .arg("repo-badge-plus-shields")
        .current_dir(workspace_root)
        .output()
        .with_context(|| format!("failed to run {ripr_bin}; set RIPR_BIN to override"))?;

    if !output.status.success() {
        bail!(
            "{ripr_bin} repo-badge-plus-shields failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let value: serde_json::Value = serde_json::from_slice(&output.stdout)
        .with_context(|| format!("{ripr_bin} emitted invalid Shields endpoint JSON"))?;
    validate_minimal_shields_keys(&value)?;
    serde_json::from_value(value)
        .with_context(|| format!("{ripr_bin} emitted invalid Shields endpoint JSON shape"))
}

/// Validate the semantic shape for a Shields endpoint badge.
pub fn validate_shields_badge(
    badge: &ShieldsEndpointBadge,
    expected_label: Option<&str>,
) -> Result<()> {
    if badge.schema_version != 1 {
        bail!("badge `{}` has unsupported schemaVersion", badge.label);
    }

    if let Some(expected_label) = expected_label {
        if badge.label != expected_label {
            bail!("badge label drifted: got `{}`, expected `{expected_label}`", badge.label);
        }
    }

    if badge.message.trim().is_empty() {
        bail!("badge `{}` has empty message", badge.label);
    }

    if badge.color.trim().is_empty() {
        bail!("badge `{}` has empty color", badge.label);
    }

    Ok(())
}

fn validate_minimal_shields_keys(value: &serde_json::Value) -> Result<()> {
    let object = value.as_object().context("badge endpoint JSON must be an object")?;
    let actual = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected =
        ["schemaVersion", "label", "message", "color"].into_iter().collect::<BTreeSet<_>>();
    if actual != expected {
        bail!("badge endpoint JSON must contain only schemaVersion, label, message, and color");
    }
    Ok(())
}

fn write_json_pretty(path: &Path, badge: &ShieldsEndpointBadge) -> Result<()> {
    let json = serde_json::to_string_pretty(badge).context("failed to serialize badge JSON")?;
    fs::write(path, format!("{json}\n"))
        .with_context(|| format!("failed to write {}", path.display()))
}

fn compare_files(committed: &Path, generated: &Path) -> Result<()> {
    let committed_bytes = fs::read(committed)
        .with_context(|| format!("missing committed badge endpoint {}", committed.display()))?;
    let generated_bytes = fs::read(generated)
        .with_context(|| format!("missing generated badge endpoint {}", generated.display()))?;

    if committed_bytes != generated_bytes {
        bail!(
            "badge endpoint drift: {} differs from generated {} (run `cargo xtask badges`)",
            committed.display(),
            generated.display()
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ripr_plus_badge_shape_is_stable() {
        let badge = ShieldsEndpointBadge {
            schema_version: 1,
            label: "ripr+".to_string(),
            message: "0".to_string(),
            color: "brightgreen".to_string(),
        };

        validate_shields_badge(&badge, Some("ripr+")).unwrap();
    }

    #[test]
    fn badge_shape_rejects_empty_message() {
        let badge = ShieldsEndpointBadge {
            schema_version: 1,
            label: "ripr+".to_string(),
            message: "".to_string(),
            color: "brightgreen".to_string(),
        };

        validate_shields_badge(&badge, Some("ripr+")).unwrap_err();
    }

    #[test]
    fn badge_json_shape_stays_minimal() {
        let value = serde_json::json!({
            "schemaVersion": 1,
            "label": "ripr+",
            "message": "0",
            "color": "brightgreen"
        });

        validate_minimal_shields_keys(&value).unwrap();
    }
}
