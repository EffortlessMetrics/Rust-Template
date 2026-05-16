use crate::Claims;
use jsonwebtoken::{EncodingKey, Header, encode};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn create_jwt_token(
    secret: &str,
    subject: &str,
    issuer: &str,
    expires_in_seconds: u64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = unix_timestamp_seconds();
    let claims = Claims {
        sub: subject.to_string(),
        exp: now + expires_in_seconds,
        iat: now,
        iss: issuer.to_string(),
    };

    encode_claims(secret, &claims)
}

pub(crate) fn encode_claims(
    secret: &str,
    claims: &Claims,
) -> Result<String, jsonwebtoken::errors::Error> {
    let encoding_key = EncodingKey::from_secret(secret.as_ref());
    encode(&Header::default(), claims, &encoding_key)
}

pub(crate) fn encode_json_claims(
    secret: &str,
    claims: &serde_json::Value,
) -> Result<String, jsonwebtoken::errors::Error> {
    let encoding_key = EncodingKey::from_secret(secret.as_ref());
    encode(&Header::default(), claims, &encoding_key)
}

pub(crate) fn unix_timestamp_seconds() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}
