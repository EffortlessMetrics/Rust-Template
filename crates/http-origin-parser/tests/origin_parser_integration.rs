use http_origin_parser::{HttpOriginScheme, parse_http_origin};

#[test]
fn integration_parses_https_origin() {
    let parsed = parse_http_origin("https://api.example.com").unwrap();
    assert_eq!(parsed.scheme, HttpOriginScheme::Https);
    assert_eq!(parsed.authority, "api.example.com");
}

#[test]
fn integration_parses_http_origin_with_port() {
    let parsed = parse_http_origin("http://localhost:3000").unwrap();
    assert_eq!(parsed.scheme, HttpOriginScheme::Http);
    assert_eq!(parsed.authority, "localhost:3000");
}

#[test]
fn integration_rejects_origin_with_path() {
    assert_eq!(parse_http_origin("https://api.example.com/path"), None);
}

#[test]
fn integration_rejects_origin_with_userinfo() {
    assert_eq!(parse_http_origin("https://user@api.example.com"), None);
}
