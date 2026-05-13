use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

const RIPR_PR_DIR: &str = "target/ripr/pr";
const RIPR_REVIEW_DIR: &str = "target/ripr/review";

pub fn run_pr(check: bool) -> Result<()> {
    let workspace_root = workspace_root_path();
    let out_dir = workspace_root.join(RIPR_PR_DIR);
    let json_path = out_dir.join("repo-exposure.json");
    let markdown_path = out_dir.join("repo-exposure.md");

    if check {
        check_json_file(&json_path)?;
        check_nonempty_file(&markdown_path)?;
        println!("ripr-pr: output contract is intact");
        return Ok(());
    }

    std::fs::create_dir_all(&out_dir).with_context(|| format!("creating {}", out_dir.display()))?;
    let ripr_bin = ripr_bin();
    let output = Command::new(&ripr_bin)
        .arg("check")
        .arg("--root")
        .arg(&workspace_root)
        .arg("--format")
        .arg("repo-exposure-json")
        .current_dir(&workspace_root)
        .output()
        .with_context(|| format!("running {ripr_bin} for PR-scoped repo exposure evidence"))?;

    if !output.status.success() {
        bail!("{ripr_bin} repo-exposure-json failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    let value: Value = serde_json::from_slice(&output.stdout)
        .with_context(|| format!("{ripr_bin} emitted invalid repo exposure JSON"))?;
    std::fs::write(&json_path, format!("{}\n", serde_json::to_string_pretty(&value)?))
        .with_context(|| format!("writing {}", json_path.display()))?;
    write_repo_exposure_markdown(&markdown_path, &value)?;
    println!("ripr-pr: wrote PR-scoped evidence under {RIPR_PR_DIR}");
    Ok(())
}

pub fn run_review_comments(check: bool) -> Result<()> {
    let workspace_root = workspace_root_path();
    let out_dir = workspace_root.join(RIPR_REVIEW_DIR);
    let json_path = out_dir.join("comments.json");
    let markdown_path = out_dir.join("comments.md");

    if check {
        check_json_file(&json_path)?;
        check_nonempty_file(&markdown_path)?;
        println!("ripr-review-comments: output contract is intact");
        return Ok(());
    }

    std::fs::create_dir_all(&out_dir).with_context(|| format!("creating {}", out_dir.display()))?;
    let ripr_bin = ripr_bin();
    let output = Command::new(&ripr_bin)
        .arg("review-comments")
        .arg("--root")
        .arg(&workspace_root)
        .arg("--base")
        .arg(ripr_base())
        .arg("--head")
        .arg(ripr_head())
        .arg("--out")
        .arg(&json_path)
        .current_dir(&workspace_root)
        .output()
        .with_context(|| format!("running {ripr_bin} review-comments"))?;

    if !output.status.success() {
        bail!("{ripr_bin} review-comments failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    check_json_file(&json_path)?;
    if !markdown_path.exists() {
        let value = read_json(&json_path)?;
        write_review_markdown(&markdown_path, &value)?;
    }
    check_nonempty_file(&markdown_path)?;
    println!("ripr-review-comments: wrote guidance under {RIPR_REVIEW_DIR}");
    Ok(())
}

fn workspace_root_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("xtask crate is two levels below the workspace root")
        .to_path_buf()
}

fn ripr_bin() -> String {
    std::env::var("RIPR_BIN").unwrap_or_else(|_| "ripr".to_string())
}

fn ripr_base() -> String {
    std::env::var("RIPR_BASE").unwrap_or_else(|_| "origin/main".to_string())
}

fn ripr_head() -> String {
    std::env::var("RIPR_HEAD").unwrap_or_else(|_| "HEAD".to_string())
}

fn check_json_file(path: &Path) -> Result<()> {
    read_json(path).map(|_| ())
}

fn read_json(path: &Path) -> Result<Value> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("reading required RIPR JSON {}", path.display()))?;
    serde_json::from_str(&contents).with_context(|| format!("parsing {}", path.display()))
}

fn check_nonempty_file(path: &Path) -> Result<()> {
    let metadata = std::fs::metadata(path)
        .with_context(|| format!("reading required RIPR artifact {}", path.display()))?;
    if metadata.len() == 0 {
        bail!("required RIPR artifact {} is empty", path.display());
    }
    Ok(())
}

fn write_repo_exposure_markdown(path: &Path, value: &Value) -> Result<()> {
    let findings = value.get("findings").and_then(Value::as_array).map_or(0, std::vec::Vec::len);
    let markdown = format!(
        "# RIPR PR Evidence\n\n- Scope: diff-scoped pull request evidence\n- Findings: `{findings}`\n\nDetailed JSON: `target/ripr/pr/repo-exposure.json`\n"
    );
    std::fs::write(path, markdown).with_context(|| format!("writing {}", path.display()))
}

fn write_review_markdown(path: &Path, value: &Value) -> Result<()> {
    let comments = value.get("comments").and_then(Value::as_array).map_or(0, std::vec::Vec::len);
    let summary_only =
        value.get("summary_only").and_then(Value::as_array).map_or(0, std::vec::Vec::len);
    let warnings = value.get("warnings").and_then(Value::as_array).map_or(0, std::vec::Vec::len);
    let markdown = format!(
        "# RIPR Review Guidance\n\n- Line-placeable comments: `{comments}`\n- Summary-only items: `{summary_only}`\n- Warnings: `{warnings}`\n\nDetailed JSON: `target/ripr/review/comments.json`\n"
    );
    std::fs::write(path, markdown).with_context(|| format!("writing {}", path.display()))
}
