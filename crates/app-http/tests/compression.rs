#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    use app_http::{app, resolve_workspace_root};
    use adapters_spec_fs::FsGovernanceRepository;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_compression_enabled() {
        let root = resolve_workspace_root();
        let repo = Arc::new(FsGovernanceRepository::new(root.clone()));
        let app = app(repo).expect("valid config");

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .header("Accept-Encoding", "gzip")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // The response body is small (~38 bytes), but should be compressed as default min size is often small (32 bytes).
        // We assert that the header is present.
        assert!(response.headers().contains_key("content-encoding"), "Content-Encoding header missing");

        let encoding = response.headers().get("content-encoding").unwrap().to_str().unwrap();
        assert_eq!(encoding, "gzip");
    }
}
