//! Reusable validation for documentation `doc_type` contracts.

/// Inputs required to validate a documentation entry against the `doc_type` contract.
#[derive(Debug, Clone)]
pub struct DocTypeInput<'a> {
    /// Raw doc_type value from spec/frontmatter.
    pub doc_type: &'a str,
    /// Linked stories.
    pub stories: &'a [String],
    /// Linked requirements.
    pub requirements: &'a [String],
    /// Linked acceptance criteria.
    pub acs: &'a [String],
}

/// Validation outcome for the `doc_type` contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult {
    /// Whether the entry passes validation.
    pub valid: bool,
    /// Optional issue description when invalid.
    pub issue: Option<String>,
}

/// Validate doc_type contract for a single documentation entry.
pub fn validate(input: DocTypeInput<'_>) -> ValidationResult {
    validate_with_options(input, false)
}

/// Validate doc_type contract, optionally allowing an empty doc_type.
pub fn validate_with_options(
    input: DocTypeInput<'_>,
    allow_empty_doc_type: bool,
) -> ValidationResult {
    let doc_type = input.doc_type.replace('-', "_");

    let issue = match doc_type.as_str() {
        "how_to" if input.requirements.is_empty() && input.acs.is_empty() => {
            Some("how_to should reference at least one requirement or AC".into())
        }
        "explanation" if input.stories.is_empty() && input.requirements.is_empty() => {
            Some("explanation should reference at least one story or requirement".into())
        }
        "design_doc" if input.requirements.is_empty() => {
            Some("design_doc should reference at least one requirement".into())
        }
        "reference" if input.requirements.is_empty() && input.acs.is_empty() => {
            Some("reference should reference at least one requirement or AC".into())
        }
        "status" if input.requirements.is_empty() || input.acs.is_empty() => {
            Some("status should reference both requirements and ACs".into())
        }
        "adr" if input.requirements.is_empty() => {
            Some("adr should reference at least one requirement".into())
        }
        "guide" if input.requirements.is_empty() && input.acs.is_empty() => {
            Some("guide should reference at least one requirement or AC".into())
        }
        "impl_plan" if input.requirements.is_empty() || input.acs.is_empty() => {
            Some("impl_plan should reference both requirements and ACs".into())
        }
        "requirements_doc" if input.requirements.is_empty() => {
            Some("requirements_doc should reference at least one requirement".into())
        }
        "how_to" | "explanation" | "design_doc" | "reference" | "status" | "adr" | "guide"
        | "impl_plan" | "requirements_doc" | "ci_workflow" => None,
        "" if allow_empty_doc_type => None,
        _ => Some(format!("Unknown doc_type '{}'", input.doc_type)),
    };

    ValidationResult { valid: issue.is_none(), issue }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input<'a>(
        doc_type: &'a str,
        requirements: &'a [String],
        acs: &'a [String],
    ) -> DocTypeInput<'a> {
        DocTypeInput { doc_type, stories: &[], requirements, acs }
    }

    #[test]
    fn validates_how_to_when_links_exist() {
        let requirements = vec!["REQ-1".to_string()];
        let result = validate(input("how_to", &requirements, &[]));
        assert!(result.valid);
        assert_eq!(result.issue, None);
    }

    #[test]
    fn returns_issue_for_unknown_type() {
        let result = validate(input("unknown", &[], &[]));
        assert!(!result.valid);
        assert_eq!(result.issue, Some("Unknown doc_type 'unknown'".to_string()));
    }

    #[test]
    fn supports_optional_empty_doc_type() {
        let result = validate_with_options(input("", &[], &[]), true);
        assert!(result.valid);
    }
}
