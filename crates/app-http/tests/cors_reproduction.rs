
#[cfg(test)]
mod tests {
    use app_http::middleware::cors::CorsConfig;

    #[test]
    fn test_cors_subdomain_security_fix() {
        let config = CorsConfig {
            allowed_origins: vec!["https://*.example.com".to_string()],
            ..Default::default()
        };

        // These should be allowed
        assert!(config.is_origin_allowed("https://api.example.com"));
        assert!(config.is_origin_allowed("https://app.example.com"));

        // This should NOT be allowed.
        // Previously, "evilexample.com" would match because it ends with "example.com".
        // The fix ensures that there is a dot separator.
        assert!(!config.is_origin_allowed("https://evilexample.com"), "Security regression: evilexample.com matched *.example.com");
    }
}
