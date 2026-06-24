//! Shared helpers for governance question identifiers.

/// Builds the normalized prefix used for question IDs.
///
/// Example: `TPL` -> `Q-TPL-`
pub fn question_prefix(category: &str) -> String {
    format!("Q-{}-", category.to_ascii_uppercase())
}

/// Returns the next numeric sequence for IDs in a category.
///
/// `existing_ids` should contain full IDs like `Q-TPL-001`.
pub fn next_question_sequence<'a>(
    category: &str,
    existing_ids: impl IntoIterator<Item = &'a str>,
) -> u32 {
    let prefix = question_prefix(category);

    existing_ids
        .into_iter()
        .filter_map(|id| id.strip_prefix(&prefix))
        .filter_map(|suffix| suffix.parse::<u32>().ok())
        .max()
        .unwrap_or(0)
        + 1
}

/// Generates the next question ID using a zero-padded, three-digit suffix.
pub fn next_question_id<'a>(
    category: &str,
    existing_ids: impl IntoIterator<Item = &'a str>,
) -> String {
    let prefix = question_prefix(category);
    let sequence = next_question_sequence(category, existing_ids);
    format!("{prefix}{sequence:03}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_first_id_when_no_existing_entries() {
        let id = next_question_id("tpl", std::iter::empty());
        assert_eq!(id, "Q-TPL-001");
    }

    #[test]
    fn increments_based_on_matching_category_only() {
        let existing = ["Q-TPL-002", "Q-TPL-007", "Q-OPS-999", "Q-TPL-not-a-number"];
        let id = next_question_id("TPL", existing);
        assert_eq!(id, "Q-TPL-008");
    }

    #[test]
    fn computes_sequence_with_mixed_case_category_input() {
        let existing = ["Q-BUNDLE-009"];
        assert_eq!(next_question_sequence("bundle", existing), 10);
    }
}
