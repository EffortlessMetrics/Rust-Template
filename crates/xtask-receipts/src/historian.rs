//! Historian LLM integration for quality receipts.
//!
//! This module provides utilities for:
//! - Extracting JSON appendix from historian markdown output
//! - Parsing historian appendix into structured types
//! - Handling historian command execution

use anyhow::{Context, Result};

/// Markers for historian appendix extraction.
pub const HISTORIAN_APPENDIX_START: &str = "<!-- historian:appendix:start -->";
pub const HISTORIAN_APPENDIX_END: &str = "<!-- historian:appendix:end -->";

/// Structured appendix from Historian LLM analysis.
/// All fields are optional - partial appendices are valid.
#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct HistorianQualityAppendix {
    /// Boundary integrity rating (improved/neutral/degraded)
    pub boundary_rating: Option<String>,
    /// Notes about boundary integrity
    #[serde(default)]
    pub boundary_notes: Vec<String>,

    /// Test depth rating (hardened/mixed/shallow)
    pub test_depth_rating: Option<String>,
    /// Notes about test depth
    #[serde(default)]
    pub test_depth_notes: Vec<String>,

    /// Risk notes from LLM analysis
    #[serde(default)]
    pub risk_notes: Vec<String>,

    /// Assumptions that materially affected analysis
    #[serde(default)]
    pub assumptions: Vec<String>,
    /// Evidence pointers (paths, commits, receipts)
    #[serde(default)]
    pub evidence_pointers: Vec<String>,

    /// Confidence level (high/medium/low)
    pub confidence: Option<String>,
}

/// Extract the JSON appendix from historian markdown output.
/// Returns the raw JSON string between markers.
///
/// # Errors
/// - Returns error if start marker not found
/// - Returns error if end marker not found
/// - Returns error if JSON is wrapped in code fences
pub fn extract_historian_appendix_json(markdown: &str) -> Result<&str> {
    let start = markdown
        .find(HISTORIAN_APPENDIX_START)
        .ok_or_else(|| anyhow::anyhow!("Historian appendix start marker not found"))?;
    let after_start = start + HISTORIAN_APPENDIX_START.len();

    let end = markdown[after_start..]
        .find(HISTORIAN_APPENDIX_END)
        .ok_or_else(|| anyhow::anyhow!("Historian appendix end marker not found"))?
        + after_start;

    let json = markdown[after_start..end].trim();

    // Guardrail: refuse fenced blocks
    if json.starts_with("```") {
        return Err(anyhow::anyhow!("Historian appendix must be raw JSON (not in a code fence)"));
    }

    Ok(json)
}

/// Parse historian appendix JSON into a structured type.
pub fn parse_historian_appendix(json: &str) -> Result<HistorianQualityAppendix> {
    serde_json::from_str(json).context("Failed to parse historian appendix JSON")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_historian_appendix_valid() {
        let markdown = r#"
# Quality Assessment

This is narrative content.

<!-- historian:appendix:start -->
{
  "boundary_rating": "improved",
  "boundary_notes": ["Note 1", "Note 2"],
  "confidence": "high"
}
<!-- historian:appendix:end -->
"#;

        let json = extract_historian_appendix_json(markdown).unwrap();
        assert!(json.contains("boundary_rating"));
        assert!(json.contains("improved"));
    }

    #[test]
    fn extract_historian_appendix_missing_start_marker() {
        let markdown = r#"
# Quality Assessment
No markers here.
"#;

        let result = extract_historian_appendix_json(markdown);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("start marker not found"));
    }

    #[test]
    fn extract_historian_appendix_missing_end_marker() {
        let markdown = r#"
<!-- historian:appendix:start -->
{ "boundary_rating": "improved" }
"#;

        let result = extract_historian_appendix_json(markdown);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("end marker not found"));
    }

    #[test]
    fn extract_historian_appendix_rejects_fenced_json() {
        let markdown = r#"
<!-- historian:appendix:start -->
```json
{ "boundary_rating": "improved" }
```
<!-- historian:appendix:end -->
"#;

        let result = extract_historian_appendix_json(markdown);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("code fence"));
    }

    #[test]
    fn parse_historian_appendix_full() {
        let json = r#"{
            "boundary_rating": "improved",
            "boundary_notes": ["Note 1", "Note 2"],
            "test_depth_rating": "hardened",
            "test_depth_notes": ["Test note"],
            "risk_notes": ["Risk 1"],
            "assumptions": ["Assumption 1"],
            "evidence_pointers": ["path:lib.rs:42"],
            "confidence": "high"
        }"#;

        let appendix = parse_historian_appendix(json).unwrap();
        assert_eq!(appendix.boundary_rating, Some("improved".to_string()));
        assert_eq!(appendix.boundary_notes.len(), 2);
        assert_eq!(appendix.test_depth_rating, Some("hardened".to_string()));
        assert_eq!(appendix.test_depth_notes.len(), 1);
        assert_eq!(appendix.risk_notes.len(), 1);
        assert_eq!(appendix.assumptions.len(), 1);
        assert_eq!(appendix.evidence_pointers.len(), 1);
        assert_eq!(appendix.confidence, Some("high".to_string()));
    }

    #[test]
    fn parse_historian_appendix_partial() {
        let json = r#"{
            "boundary_rating": "neutral",
            "confidence": "low"
        }"#;

        let appendix = parse_historian_appendix(json).unwrap();
        assert_eq!(appendix.boundary_rating, Some("neutral".to_string()));
        assert!(appendix.boundary_notes.is_empty());
        assert!(appendix.test_depth_rating.is_none());
        assert!(appendix.test_depth_notes.is_empty());
        assert!(appendix.risk_notes.is_empty());
        assert!(appendix.assumptions.is_empty());
        assert!(appendix.evidence_pointers.is_empty());
        assert_eq!(appendix.confidence, Some("low".to_string()));
    }

    #[test]
    fn parse_historian_appendix_empty() {
        let json = "{}";

        let appendix = parse_historian_appendix(json).unwrap();
        assert!(appendix.boundary_rating.is_none());
        assert!(appendix.boundary_notes.is_empty());
        assert!(appendix.test_depth_rating.is_none());
        assert!(appendix.test_depth_notes.is_empty());
        assert!(appendix.risk_notes.is_empty());
        assert!(appendix.assumptions.is_empty());
        assert!(appendix.evidence_pointers.is_empty());
        assert!(appendix.confidence.is_none());
    }
}
