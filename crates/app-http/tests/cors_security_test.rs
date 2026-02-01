#[cfg(test)]
mod tests {
    use app_http::middleware::CorsConfig;

    #[test]
    fn test_cors_wildcard_subdomain_takeover() {
        let config = CorsConfig {
            allowed_origins: vec!["https://*.example.com".to_string()],
            ..Default::default()
        };

        // This should be allowed
        assert!(config.is_origin_allowed("https://api.example.com"), "Valid subdomain should be allowed");

        // This was the vulnerability: partial match on domain name
        // Now it should be blocked
        assert!(!config.is_origin_allowed("https://evilexample.com"), "Partial domain match should be BLOCKED");

        // Edge case: matching the domain exactly (without subdomain)
        // With "*.example.com", usually "example.com" is NOT allowed unless explicitly added
        assert!(!config.is_origin_allowed("https://example.com"), "Base domain should not match *.domain");

        // Another subdomain
        assert!(config.is_origin_allowed("https://foo.bar.example.com"), "Nested subdomain should be allowed");
    }
}
