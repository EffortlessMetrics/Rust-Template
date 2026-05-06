use crate::run_cmd;
use anyhow::{Context, Result};
use colored::Colorize;
use glob::glob;
use regex::{Captures, Regex};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn project_root() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    std::path::Path::new(manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("Failed to find project root")
        .to_path_buf()
}

pub fn run() -> Result<()> {
    println!("{}", "📌 Pinning GitHub Actions to SHAs...".blue());

    // Git config (best effort)
    let _ = Command::new("git").args(["config", "user.name", "gha-bot"]).output();
    let _ = Command::new("git")
        .args(["config", "user.email", "gha-bot@users.noreply.github.com"])
        .output();

    // Checkout branch
    let branch = "maintenance/pin-actions";
    println!("Checking out branch: {}", branch);
    run_cmd(Command::new("git").args(["checkout", "-B", branch]))?;

    let mut changed = false;
    let root = project_root();
    let workflows_dir = root.join(".github/workflows");

    // Regex to find "uses: owner/repo@ref"
    // Matches: uses: <owner>/<repo>@<ref>
    let uses_re = Regex::new(r"uses:\s*([\w.-]+)/([\w.-]+)@([\w./-]+)").unwrap();

    // Regex to check if it's already a SHA (40 hex chars)
    let sha_re = Regex::new(r"^[0-9a-f]{40}$").unwrap();

    let pattern = workflows_dir.join("*.yml");
    let pattern_str = pattern.to_str().context("Invalid path")?;

    for entry in glob(pattern_str)? {
        let path = entry?;
        let content = fs::read_to_string(&path)?;
        let mut file_changed = false;

        // Replace refs with SHAs
        let new_content = uses_re
            .replace_all(&content, |caps: &Captures<'_>| {
                let owner = &caps[1];
                let repo = &caps[2];
                let ref_ = &caps[3];

                if sha_re.is_match(ref_) {
                    return caps[0].to_string(); // Already a SHA
                }

                print!("Resolving {}/{}@{}... ", owner, repo, ref_);

                if let Some(sha) = resolve_sha(owner, repo, ref_) {
                    println!("{}", sha.green());
                    file_changed = true;
                    format!("uses: {}/{}@{}", owner, repo, sha)
                } else {
                    println!("{}", "Failed".red());
                    eprintln!("Warning: Could not resolve {}/{}@{}", owner, repo, ref_);
                    caps[0].to_string()
                }
            })
            .to_string();

        if file_changed && new_content != content {
            fs::write(&path, new_content)?;
            changed = true;
            println!("Updated {}", path.display());
        }
    }

    if changed {
        println!("{}", "Committing changes...".blue());
        run_cmd(Command::new("git").args(["add", ".github/workflows"]))?;
        run_cmd(Command::new("git").args(["commit", "-m", "Pin GitHub Actions to commit SHAs"]))?;

        println!("{}", "Pushing changes...".blue());
        run_cmd(Command::new("git").args(["push", "-u", "origin", branch, "--force"]))?;

        println!("{}", "Creating PR...".blue());
        // Best effort PR creation
        let mut pr_cmd = Command::new("gh");
        pr_cmd.args([
            "pr",
            "create",
            "--fill",
            "--title",
            "Pin Actions to SHAs",
            "--body",
            "Automated hard-pinning of GitHub Actions.",
        ]);
        if let Err(e) = run_cmd(&mut pr_cmd) {
            println!("{} Failed to create PR (it might already exist): {}", "⚠".yellow(), e);
        }
    } else {
        println!("{}", "No changes needed.".green());
    }

    Ok(())
}

fn resolve_sha(owner: &str, repo: &str, ref_: &str) -> Option<String> {
    // Try commits endpoint
    // gh api repos/:owner/:repo/commits/:ref --jq .sha
    let output = Command::new("gh")
        .args(["api", &format!("repos/{}/{}/commits/{}", owner, repo, ref_), "--jq", ".sha"])
        .output()
        .ok()?;

    if output.status.success() {
        let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !sha.is_empty() {
            return Some(sha);
        }
    }

    // Try tags endpoint
    // gh api repos/:owner/:repo/git/ref/tags/:ref --jq .object.sha
    let output = Command::new("gh")
        .args([
            "api",
            &format!("repos/{}/{}/git/ref/tags/{}", owner, repo, ref_),
            "--jq",
            ".object.sha",
        ])
        .output()
        .ok()?;

    if output.status.success() {
        let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !sha.is_empty() {
            return Some(sha);
        }
    }

    None
}
