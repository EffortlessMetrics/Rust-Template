# Design: `ac-report` Consumer Command

**Status:** Proposed
**Date:** 2025-12-05
**Author:** Agent
**Tags:** ac-coverage, governance, cli, consumer

## Summary

A thin CLI consumer that treats `ac-status --json` as its data source, providing human-readable views of AC governance state for dashboards, PRs, and agent workflows.

## Motivation

`ac-status --json` provides machine-readable AC coverage data (schema v2.0), but consumers need different views:

- **PR summaries**: Compact markdown for review comments
- **Dashboard reports**: HTML/markdown for portals
- **Agent guidance**: Structured backlog by priority
- **Trend analysis**: Historical coverage diffs

Rather than extending `ac-status`, we create a dedicated consumer command.

## Design Principles

1. **Data source**: Calls `ac-status --json` internally (or reuses the same logic)
2. **Schema-aware**: Keys on `schema_version` for forward compatibility
3. **Output-focused**: Multiple output formats for different use cases
4. **Composable**: Can be piped, filtered, and combined with other tools

## Proposed Interface

```bash
# Default: human-readable summary (backlog by requirement)
cargo xtask ac-report

# Kernel-only view (must_have_ac=true)
cargo xtask ac-report --must-have

# Filter by status
cargo xtask ac-report --status unknown
cargo xtask ac-report --status fail

# Group by story instead of requirement
cargo xtask ac-report --by-story

# Output formats
cargo xtask ac-report --format markdown  # For PR comments
cargo xtask ac-report --format html      # For portals
cargo xtask ac-report --format json      # Pass-through to ac-status

# Combine filters
cargo xtask ac-report --must-have --status unknown --format markdown
```

## Output Views

### 1. Default View: Backlog by Requirement

```
AC Governance Report
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Summary:
  Must-have ACs: 48 total (46 passing, 1 failing, 1 unknown)
  Optional ACs:  17 total (15 passing, 0 failing, 2 unknown)
  Coverage:      93.8%

Kernel Backlog (must_have_ac=true, status=unknown):
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  REQ-PLT-ONBOARDING
    [ ] AC-PLT-001 - xtask doctor validates environment
        → cargo xtask ac-suggest-scenarios AC-PLT-001

Failing ACs:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  REQ-PLT-RELEASE
    [✗] AC-PLT-012 - release-bundle generates valid SBOM
        Source: coverage (from coverage.jsonl)
        → cargo xtask test-ac AC-PLT-012

Next Steps:
  1. Fix failing AC:  cargo xtask test-ac AC-PLT-012
  2. Add coverage for: cargo xtask ac-suggest-scenarios AC-PLT-001
```

### 2. Markdown View (for PRs)

```markdown
## AC Coverage Report

| Category | Total | Pass | Fail | Unknown |
|----------|-------|------|------|---------|
| Must-have | 48 | 46 | 1 | 1 |
| Optional | 17 | 15 | 0 | 2 |

**Coverage:** 93.8%

### Blockers

- **AC-PLT-012** (fail): release-bundle generates valid SBOM
  - Requirement: REQ-PLT-RELEASE
  - Source: coverage.jsonl

### Missing Coverage (Kernel)

- **AC-PLT-001**: xtask doctor validates environment
```

### 3. Heatmap View (for dashboards)

```
Kernel AC Heatmap by Story
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

US-TPL-PLT-001: Platform DevEx & Governance
  ████████████████████░  21/22 (95%)
  └─ 1 unknown: AC-PLT-001

US-TPL-PLT-002: Release Safety
  ████████████████░░░░  16/20 (80%)
  └─ 1 fail: AC-PLT-012
  └─ 3 unknown: AC-PLT-018, AC-PLT-019, AC-PLT-020
```

## Implementation Sketch

### Data Types (reuse from ac_status.rs)

```rust
// Already exists in ac_status.rs - just re-export
pub use super::ac_status::{AcStatusJson, AcCategoryStats, AcJson, AcSource};

/// Deserialized AC status report
#[derive(Debug, Deserialize)]
pub struct AcReport {
    pub schema_version: String,
    pub timestamp: String,
    pub must_have_acs: AcCategoryStats,
    pub optional_acs: AcCategoryStats,
    pub coverage_percent: f64,
    pub acs: Vec<AcJson>,
}

impl AcReport {
    /// Load from ac-status --json output
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).context("Failed to parse ac-status JSON")
    }

    /// Load by calling ac-status internally
    pub fn load() -> Result<Self> {
        let output = Command::new("cargo")
            .args(["xtask", "ac-status", "--json"])
            .output()?;
        Self::from_json(&String::from_utf8_lossy(&output.stdout))
    }

    /// Filter ACs by criteria
    pub fn filter(&self, opts: &FilterOpts) -> Vec<&AcJson> {
        self.acs.iter()
            .filter(|ac| opts.matches(ac))
            .collect()
    }

    /// Group ACs by requirement
    pub fn by_requirement(&self) -> BTreeMap<String, Vec<&AcJson>> {
        let mut groups = BTreeMap::new();
        for ac in &self.acs {
            groups.entry(ac.req_id.clone()).or_insert_with(Vec::new).push(ac);
        }
        groups
    }

    /// Group ACs by story
    pub fn by_story(&self) -> BTreeMap<String, Vec<&AcJson>> {
        let mut groups = BTreeMap::new();
        for ac in &self.acs {
            groups.entry(ac.story_id.clone()).or_insert_with(Vec::new).push(ac);
        }
        groups
    }
}
```

### CLI Arguments

```rust
#[derive(Debug, Parser)]
pub struct AcReportArgs {
    /// Only show must_have_ac=true ACs
    #[arg(long)]
    pub must_have: bool,

    /// Filter by status
    #[arg(long, value_parser = ["pass", "fail", "unknown"])]
    pub status: Option<String>,

    /// Group by story instead of requirement
    #[arg(long)]
    pub by_story: bool,

    /// Output format
    #[arg(long, value_parser = ["text", "markdown", "html", "json"], default_value = "text")]
    pub format: String,
}
```

### Module Structure

```
crates/xtask/src/commands/
├── ac_report.rs           # New consumer command
├── ac_report/
│   ├── mod.rs
│   ├── data.rs            # AcReport struct + filtering
│   ├── views/
│   │   ├── mod.rs
│   │   ├── text.rs        # Default terminal view
│   │   ├── markdown.rs    # PR/docs view
│   │   ├── html.rs        # Portal view
│   │   └── heatmap.rs     # Dashboard view
│   └── tests.rs
```

## Integration with main.rs

```rust
// In Commands enum
#[command(next_help_heading = "📋 Acceptance Criteria")]
AcReport {
    /// Only show must_have_ac=true ACs
    #[arg(long)]
    must_have: bool,

    /// Filter by status (pass, fail, unknown)
    #[arg(long)]
    status: Option<String>,

    /// Group by story instead of requirement
    #[arg(long)]
    by_story: bool,

    /// Output format (text, markdown, html, json)
    #[arg(long, default_value = "text")]
    format: String,
},

// In match arm
Commands::AcReport { must_have, status, by_story, format } => {
    commands::ac_report::run(AcReportArgs {
        must_have,
        status,
        by_story,
        format,
    })
}
```

## Alternative: Pure Pipeline Approach

Instead of a Rust command, we could use jq pipelines:

```bash
# Kernel backlog
cargo xtask ac-status --json | jq '
  .acs
  | map(select(.must_have_ac and .status == "unknown"))
  | group_by(.req_id)
  | map({req: .[0].req_id, acs: [.[].id]})
'

# Failing ACs
cargo xtask ac-status --json | jq '
  .acs | map(select(.status == "fail")) | .[].id
'
```

This works but is less portable and harder to extend with rich formatting.

## Decision: Start Minimal

Recommend starting with:

1. **Phase 1**: Minimal `ac-report` command with text + markdown output
2. **Phase 2**: Add HTML output for portal integration
3. **Phase 3**: Add historical diff support (requires caching previous reports)

## Open Questions

1. **Caching**: Should we cache `ac-status --json` output, or regenerate each time?
   - Proposal: Regenerate by default, add `--from-cache` flag later

2. **CI artifact**: Should `ac-report` output be saved as a CI artifact?
   - Proposal: Let CI workflows handle this (`cargo xtask ac-report > report.md`)

3. **Schema evolution**: How do we handle schema version mismatches?
   - Proposal: Warn on unknown schema_version, fail on missing required fields

## Related

- [ac-status-json-schema.md](../reference/ac-status-json-schema.md) - JSON schema reference
- [xtask-commands.md](../reference/xtask-commands.md) - CLI reference
- `ac_status.rs` - Source of JSON output
