use anyhow::{Context, Result};
use colored::Colorize;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub(super) fn check_orphaned_versions() -> Result<()> {
    use regex::Regex;
    use std::collections::HashSet;
    use walkdir::WalkDir;

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().expect("workspace root").parent().expect("repo root");

    // Load version manifest
    let manifest_path = root.join("specs/version_manifest.yaml");
    if !manifest_path.exists() {
        // No manifest means no version governance - skip check
        return Ok(());
    }

    let manifest = crate::commands::versioning::VersionManifest::load_from_path(&manifest_path)
        .context("Failed to load version manifest")?;

    // Build set of "owned" file paths and their patterns
    // A version string is "owned" if it appears in a file covered by the manifest
    // and matches one of the declared patterns for that file
    let mut owned_patterns: HashSet<String> = HashSet::new();

    for target in &manifest.files {
        // Skip glob patterns (e.g., release_evidence/v*.md)
        if target.path.contains('*') {
            continue;
        }

        for pattern in &target.patterns {
            // Track the marker as an owned pattern
            owned_patterns.insert(pattern.marker.clone());

            // Also track the line_pattern regex if available
            if let Some(line_pat) = &pattern.line_pattern {
                owned_patterns.insert(line_pat.clone());
            }
        }
    }

    // Version pattern: vX.Y.Z or X.Y.Z (possibly with -kernel suffix)
    let version_re =
        Regex::new(r"v?\d+\.\d+\.\d+(-kernel)?").context("Failed to compile version regex")?;

    // Suppression comment patterns
    let md_suppress = "<!-- doclint:disable orphan-version -->";
    let yaml_suppress = "# doclint:disable orphan-version";

    let mut orphans = Vec::new();

    // Walk docs/ and specs/ directories for markdown and YAML files
    let scan_dirs =
        vec![root.join("docs"), root.join("specs"), root.join("README.md"), root.join("CLAUDE.md")];

    // Files to skip entirely (they contain version examples or non-template versions)
    let skip_files = [
        "specs/version_manifest.yaml",
        "specs/version_manifest.example.yaml",
        "specs/openapi/openapi.yaml",
        "specs/service_policies.yaml",
        "specs/doc_policies.yaml",
        "specs/devex_flows.yaml",
    ];

    for scan_path in scan_dirs {
        if !scan_path.exists() {
            continue;
        }

        let entries: Vec<_> = if scan_path.is_file() {
            vec![scan_path]
        } else {
            WalkDir::new(&scan_path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let path = e.path();
                    path.is_file()
                        && (path.extension().map(|s| s == "md").unwrap_or(false)
                            || path.extension().map(|s| s == "yaml" || s == "yml").unwrap_or(false))
                })
                .map(|e| e.path().to_path_buf())
                .collect()
        };

        for file_path in entries {
            // Get relative path for checks
            let relative_path = file_path.strip_prefix(root).ok().and_then(|p| p.to_str());

            // Skip files that are explicitly excluded
            if let Some(rel) = relative_path
                && skip_files.contains(&rel)
            {
                continue;
            }

            // Skip files covered by manifest
            let is_manifest_file = relative_path
                .and_then(|p| manifest.files.iter().find(|f| Path::new(&f.path) == Path::new(p)))
                .is_some();

            if is_manifest_file {
                // This file is governed by the manifest - its versions are "owned"
                continue;
            }

            let content = fs::read_to_string(&file_path)
                .with_context(|| format!("Failed to read {}", file_path.display()))?;

            // Check for file-wide suppression comment in first 25 lines (covers frontmatter + comment)
            let lines: Vec<&str> = content.lines().collect();
            let file_suppressed = lines
                .iter()
                .take(25) // Check first 25 lines for suppression (covers typical frontmatter)
                .any(|line| line.contains(md_suppress) || line.contains(yaml_suppress));

            if file_suppressed {
                // Entire file is suppressed from orphan-version checking
                continue;
            }

            for (line_num, line) in lines.iter().enumerate() {
                // Check for inline suppression comment (per-line)
                if line.contains(md_suppress) || line.contains(yaml_suppress) {
                    continue;
                }

                // Skip lines with schema_version (these are YAML schema versions, not template versions)
                if line.contains("schema_version")
                    || line.contains("openapi:")
                    || line.contains("version:") && line.contains("\"")
                {
                    continue;
                }

                // Look for version patterns
                for cap in version_re.find_iter(line) {
                    let version_str = cap.as_str();

                    // Check if this version occurrence is in an owned context
                    // A version is "owned" if the line contains one of the manifest markers
                    let is_owned =
                        owned_patterns.iter().any(|marker| line.contains(marker.as_str()));

                    if !is_owned {
                        let relative = file_path
                            .strip_prefix(root)
                            .unwrap_or(&file_path)
                            .display()
                            .to_string();

                        orphans.push(format!(
                            "{}:{} - orphaned version '{}' (not in version_manifest.yaml)",
                            relative,
                            line_num + 1,
                            version_str
                        ));
                    }
                }
            }
        }
    }

    if !orphans.is_empty() {
        eprintln!();
        eprintln!("{}", "DOC_ORPHANED_VERSION errors:".yellow().bold());
        for orphan in &orphans {
            eprintln!("  ✗ {}", orphan);
        }
        eprintln!();
        eprintln!("{}", "To fix:".bold());
        eprintln!(
            "  1. Add the file and pattern to {} if this version should be managed",
            "specs/version_manifest.yaml".cyan()
        );
        eprintln!(
            "  2. Or suppress with {} or {}",
            "<!-- doclint:disable orphan-version -->".cyan(),
            "# doclint:disable orphan-version".cyan()
        );
        eprintln!("  3. Or remove the version string if it's stale/duplicated");
        eprintln!(
            "  4. Run {} to see what's managed",
            "cargo xtask release-prepare --dry-run X.Y.Z".cyan()
        );

        anyhow::bail!("{} orphaned version string(s) found", orphans.len());
    }

    Ok(())
}

/// Broken link information for reporting
#[derive(Debug)]
struct BrokenLink {
    /// Path to the file containing the broken link
    file: String,
    /// Line number where the link was found
    line: usize,
    /// The link text (what appears in brackets)
    #[allow(dead_code)]
    text: String,
    /// The target URL/path that is broken
    target: String,
    /// Reason the link is broken
    reason: String,
}

/// Validate markdown links in documentation files.
///
/// Scans markdown files in:
/// - docs/
/// - specs/features/ (only .md files)
/// - README.md, CHANGELOG.md, CLAUDE.md (root)
///
/// Checks that local file links (not http/https) resolve to existing files.
/// Anchor links (#section) are advisory warnings only.
pub(super) fn validate_markdown_links() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().expect("workspace root").parent().expect("repo root");

    // Regex to match markdown links: [text](url)
    // Captures: group 1 = link text, group 2 = url/path
    let link_re =
        Regex::new(r"\[([^\]]*)\]\(([^)]+)\)").context("Failed to compile markdown link regex")?;

    let mut broken_links: Vec<BrokenLink> = Vec::new();
    let mut anchor_warnings: Vec<BrokenLink> = Vec::new();

    // Collect markdown files to scan
    let mut files_to_scan: Vec<PathBuf> = Vec::new();

    // Root-level markdown files
    for name in ["README.md", "CHANGELOG.md", "CLAUDE.md"] {
        let path = root.join(name);
        if path.exists() {
            files_to_scan.push(path);
        }
    }

    // docs/ directory (recursive)
    let docs_dir = root.join("docs");
    if docs_dir.exists() {
        for entry in WalkDir::new(&docs_dir).into_iter().filter_map(|e| e.ok()).filter(|e| {
            e.path().is_file() && e.path().extension().map(|s| s == "md").unwrap_or(false)
        }) {
            files_to_scan.push(entry.path().to_path_buf());
        }
    }

    // specs/features/ directory (only .md files)
    let specs_features_dir = root.join("specs/features");
    if specs_features_dir.exists() {
        for entry in
            WalkDir::new(&specs_features_dir).into_iter().filter_map(|e| e.ok()).filter(|e| {
                e.path().is_file() && e.path().extension().map(|s| s == "md").unwrap_or(false)
            })
        {
            files_to_scan.push(entry.path().to_path_buf());
        }
    }

    // Process each file
    for file_path in &files_to_scan {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue, // Skip files we can't read
        };

        let file_dir = file_path.parent().unwrap_or(root);
        let relative_file = file_path.strip_prefix(root).unwrap_or(file_path).display().to_string();

        for (line_num, line) in content.lines().enumerate() {
            // Skip code blocks (simple heuristic: lines starting with ``` or indented by 4 spaces)
            if line.trim_start().starts_with("```") || line.starts_with("    ") {
                continue;
            }

            for cap in link_re.captures_iter(line) {
                let link_text = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let target = cap.get(2).map(|m| m.as_str()).unwrap_or("");

                // Skip external URLs
                if target.starts_with("http://") || target.starts_with("https://") {
                    continue;
                }

                // Skip mailto links
                if target.starts_with("mailto:") {
                    continue;
                }

                // Skip pure anchor links (links to sections in the same document)
                if target.starts_with('#') {
                    // These are self-referential anchors, we could validate them
                    // but for now we skip as they're lower priority
                    continue;
                }

                // Handle links with anchors (e.g., "file.md#section")
                let (path_part, anchor_part) = if let Some(hash_pos) = target.find('#') {
                    (&target[..hash_pos], Some(&target[hash_pos + 1..]))
                } else {
                    (target, None)
                };

                // Skip empty paths (just anchor was handled above)
                if path_part.is_empty() {
                    continue;
                }

                // Resolve the target path relative to the file's directory
                let resolved_path = if let Some(stripped) = path_part.strip_prefix('/') {
                    // Absolute path from repo root
                    root.join(stripped)
                } else {
                    // Relative path from file's directory
                    file_dir.join(path_part)
                };

                // Normalize the path (resolve .. and .)
                let normalized_path = normalize_path(&resolved_path);

                // Check if the file/directory exists
                if !normalized_path.exists() {
                    broken_links.push(BrokenLink {
                        file: relative_file.clone(),
                        line: line_num + 1,
                        text: link_text.to_string(),
                        target: target.to_string(),
                        reason: "File not found".to_string(),
                    });
                    continue;
                }

                // If it's a directory link without trailing slash, that's OK
                // (some markdown renderers handle this)

                // If there's an anchor, try to validate it (advisory only)
                // Note: Nested ifs kept for clarity - each level has distinct semantics
                #[allow(clippy::collapsible_if)]
                if let Some(anchor) = anchor_part {
                    if !anchor.is_empty() && normalized_path.is_file() {
                        if let Ok(target_content) = fs::read_to_string(&normalized_path) {
                            if !anchor_exists_in_markdown(&target_content, anchor) {
                                anchor_warnings.push(BrokenLink {
                                    file: relative_file.clone(),
                                    line: line_num + 1,
                                    text: link_text.to_string(),
                                    target: target.to_string(),
                                    reason: format!("Anchor '{}' not found in target file", anchor),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Report results
    if !broken_links.is_empty() {
        eprintln!();
        eprintln!("{}", "Broken markdown links:".red().bold());
        for link in &broken_links {
            eprintln!(
                "  {}:{} - {} ({})",
                link.file.cyan(),
                link.line,
                link.target.red(),
                link.reason
            );
        }
    }

    if !anchor_warnings.is_empty() {
        eprintln!();
        eprintln!("{}", "Anchor warnings (advisory):".yellow().bold());
        for link in &anchor_warnings {
            eprintln!(
                "  {}:{} - {} ({})",
                link.file.cyan(),
                link.line,
                link.target.yellow(),
                link.reason
            );
        }
    }

    if !broken_links.is_empty() {
        eprintln!();
        eprintln!("{}", "To fix:".bold());
        eprintln!("  • Update broken links to point to existing files");
        eprintln!("  • Or remove references to deleted documents");
        eprintln!("  • Check relative paths are correct from the file's location");
        anyhow::bail!("{} broken link(s) found", broken_links.len());
    }

    Ok(())
}

/// Normalize a path by resolving . and .. components.
pub(super) fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::CurDir => {
                // Skip current directory markers
            }
            _ => {
                normalized.push(component);
            }
        }
    }
    normalized
}

/// Check if an anchor (heading ID) exists in a markdown file.
///
/// Markdown heading anchors are typically generated by:
/// 1. Converting to lowercase
/// 2. Replacing spaces with hyphens
/// 3. Removing special characters
pub(super) fn anchor_exists_in_markdown(content: &str, anchor: &str) -> bool {
    let anchor_lower = anchor.to_lowercase();

    for line in content.lines() {
        // Check for markdown headings (# Heading)
        if let Some(heading_text) = line.strip_prefix('#') {
            // Remove leading # and whitespace
            let heading_text = heading_text.trim_start_matches('#').trim();

            // Generate anchor from heading
            let generated_anchor: String = heading_text
                .to_lowercase()
                .chars()
                .map(|c| {
                    if c.is_alphanumeric() || c == '-' || c == '_' {
                        c
                    } else if c.is_whitespace() {
                        '-'
                    } else {
                        // Skip other characters
                        '\0'
                    }
                })
                .filter(|&c| c != '\0')
                .collect();

            // Remove consecutive hyphens
            let normalized: String =
                generated_anchor.split('-').filter(|s| !s.is_empty()).collect::<Vec<_>>().join("-");

            if normalized == anchor_lower {
                return true;
            }
        }

        // Also check for explicit anchor tags: {#anchor-id}
        if line.contains(&format!("{{#{}}}", anchor)) {
            return true;
        }

        // Check for HTML anchor tags: <a name="anchor-id">
        if line.contains(&format!("name=\"{}\"", anchor))
            || line.contains(&format!("id=\"{}\"", anchor))
        {
            return true;
        }
    }

    false
}
