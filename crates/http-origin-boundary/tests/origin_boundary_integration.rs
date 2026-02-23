use http_origin_boundary::{has_label_boundary_before_suffix, has_path_boundary_or_empty};

#[test]
fn integration_label_boundary_matches_valid_subdomain() {
    assert!(has_label_boundary_before_suffix("api.example.com", "example.com"));
}

#[test]
fn integration_label_boundary_rejects_boundaryless_suffix() {
    assert!(!has_label_boundary_before_suffix("notexample.com", "example.com"));
}

#[test]
fn integration_label_boundary_rejects_equal_length() {
    assert!(!has_label_boundary_before_suffix("example.com", "example.com"));
}

#[test]
fn integration_path_boundary_accepts_empty_and_slash() {
    assert!(has_path_boundary_or_empty(""));
    assert!(has_path_boundary_or_empty("/path"));
}

#[test]
fn integration_path_boundary_rejects_non_boundary_prefixes() {
    assert!(!has_path_boundary_or_empty(".evil/path"));
    assert!(!has_path_boundary_or_empty("?debug=true"));
}
