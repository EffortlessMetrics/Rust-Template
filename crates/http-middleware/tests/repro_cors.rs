use http_middleware::CorsConfig;

#[test]
fn test_cors_vulnerability_repro() {
    let config = CorsConfig {
        allowed_origins: vec!["https://*.example.com".to_string()],
        ..Default::default()
    };

    // The vulnerability (evilexample.com matching *.example.com) should be gone
    assert!(!config.is_origin_allowed("https://evilexample.com"), "Vulnerability fixed: evilexample.com should NOT match *.example.com");

    // Legitimate subdomains should still work
    assert!(config.is_origin_allowed("https://api.example.com"), "Valid subdomain api.example.com should match");
    assert!(config.is_origin_allowed("https://sub.test.example.com"), "Nested subdomain sub.test.example.com should match");

    // The base domain example.com does NOT match *.example.com with this logic (standard behavior unless explicitly allowed)
    // If example.com was intended, it should be in allowed_origins separately or logic adjusted.
    // For now, we verify strict subdomain matching behavior.
    assert!(!config.is_origin_allowed("https://example.com"), "Base domain example.com should NOT match *.example.com");
}
