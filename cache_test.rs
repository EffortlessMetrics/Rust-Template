use axum::{http::{HeaderValue, header::HeaderName}, response::Response};
use std::collections::HashMap;

#[derive(Clone)]
pub struct CachedSecurityHeaders {
    enabled: bool,
    headers: Vec<(HeaderName, HeaderValue)>,
}

impl CachedSecurityHeaders {
    pub fn apply_headers(&self, response: &mut Response<()>) {
        if !self.enabled {
            return;
        }
        for (name, value) in &self.headers {
            response.headers_mut().insert(name.clone(), value.clone());
        }
    }
}

fn main() {
}
