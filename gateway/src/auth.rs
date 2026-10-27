//! Request signing: `hex(HMAC-SHA256(secret, timestamp + METHOD + path_and_query + body))`.

use std::collections::HashMap;

use axum::http::HeaderMap;
use hmac::{Hmac, Mac};
use sha2::Sha256;

pub const HEADER_API_KEY: &str = "x-api-key";
pub const HEADER_TIMESTAMP: &str = "x-timestamp";
pub const HEADER_SIGNATURE: &str = "x-signature";

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, PartialEq, Eq)]
pub enum AuthError {
    MissingHeader(&'static str),
    UnknownApiKey,
    InvalidTimestamp,
    TimestampOutsideWindow,
    InvalidSignature,
}

impl AuthError {
    pub fn message(&self) -> String {
        match self {
            AuthError::MissingHeader(h) => format!("missing header {h}"),
            AuthError::UnknownApiKey => "unknown API key".into(),
            AuthError::InvalidTimestamp => "timestamp must be milliseconds since epoch".into(),
            AuthError::TimestampOutsideWindow => "timestamp outside recvWindow".into(),
            AuthError::InvalidSignature => "signature verification failed".into(),
        }
    }
}

pub fn signing_payload(
    timestamp: &str,
    method: &str,
    path_and_query: &str,
    body: &[u8],
) -> Vec<u8> {
    let mut payload =
        Vec::with_capacity(timestamp.len() + method.len() + path_and_query.len() + body.len());
    payload.extend_from_slice(timestamp.as_bytes());
    payload.extend_from_slice(method.to_ascii_uppercase().as_bytes());
    payload.extend_from_slice(path_and_query.as_bytes());
    payload.extend_from_slice(body);
    payload
}

pub fn sign(secret: &str, payload: &[u8]) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key length");
    mac.update(payload);
    hex::encode(mac.finalize().into_bytes())
}

/// Constant-time signature check.
pub fn verify(secret: &str, payload: &[u8], signature_hex: &str) -> bool {
    let Ok(signature) = hex::decode(signature_hex) else {
        return false;
    };
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key length");
    mac.update(payload);
    mac.verify_slice(&signature).is_ok()
}

fn header<'a>(headers: &'a HeaderMap, name: &'static str) -> Result<&'a str, AuthError> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .filter(|v| !v.is_empty())
        .ok_or(AuthError::MissingHeader(name))
}

/// Verifies the request and returns the authenticated API key.
pub fn authenticate(
    keys: &HashMap<String, String>,
    recv_window_ms: u64,
    now_ms: u64,
    headers: &HeaderMap,
    method: &str,
    path_and_query: &str,
    body: &[u8],
) -> Result<String, AuthError> {
    let api_key = header(headers, HEADER_API_KEY)?;
    let timestamp = header(headers, HEADER_TIMESTAMP)?;
    let signature = header(headers, HEADER_SIGNATURE)?;

    let secret = keys.get(api_key).ok_or(AuthError::UnknownApiKey)?;
    let ts: u64 = timestamp.parse().map_err(|_| AuthError::InvalidTimestamp)?;
    if now_ms.abs_diff(ts) > recv_window_ms {
        return Err(AuthError::TimestampOutsideWindow);
    }
    if !verify(
        secret,
        &signing_payload(timestamp, method, path_and_query, body),
        signature,
    ) {
        return Err(AuthError::InvalidSignature);
    }
    Ok(api_key.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn signed_headers(
        key: &str,
        secret: &str,
        ts: u64,
        method: &str,
        paq: &str,
        body: &[u8],
    ) -> HeaderMap {
        let ts = ts.to_string();
        let mut h = HeaderMap::new();
        h.insert(HEADER_API_KEY, HeaderValue::from_str(key).unwrap());
        h.insert(HEADER_TIMESTAMP, HeaderValue::from_str(&ts).unwrap());
        let sig = sign(secret, &signing_payload(&ts, method, paq, body));
        h.insert(HEADER_SIGNATURE, HeaderValue::from_str(&sig).unwrap());
        h
    }

    fn keys() -> HashMap<String, String> {
        HashMap::from([("k1".to_string(), "s1".to_string())])
    }

    #[test]
    fn accepts_valid_signature() {
        let h = signed_headers("k1", "s1", 1_000, "POST", "/api/v1/wallet/x?a=1", b"{}");
        assert_eq!(
            authenticate(
                &keys(),
                5_000,
                2_000,
                &h,
                "POST",
                "/api/v1/wallet/x?a=1",
                b"{}"
            ),
            Ok("k1".to_string())
        );
    }

    #[test]
    fn rejects_tampered_request() {
        let h = signed_headers("k1", "s1", 1_000, "POST", "/api/v1/wallet/x?a=1", b"{}");
        assert_eq!(
            authenticate(
                &keys(),
                5_000,
                2_000,
                &h,
                "POST",
                "/api/v1/wallet/x?a=2",
                b"{}"
            ),
            Err(AuthError::InvalidSignature)
        );
        assert_eq!(
            authenticate(
                &keys(),
                5_000,
                2_000,
                &h,
                "POST",
                "/api/v1/wallet/x?a=1",
                b"{\"x\":1}"
            ),
            Err(AuthError::InvalidSignature)
        );
    }

    #[test]
    fn rejects_wrong_secret_unknown_key_and_stale_timestamp() {
        let h = signed_headers("k1", "wrong", 1_000, "GET", "/p", b"");
        assert_eq!(
            authenticate(&keys(), 5_000, 1_000, &h, "GET", "/p", b""),
            Err(AuthError::InvalidSignature)
        );

        let h = signed_headers("nope", "s1", 1_000, "GET", "/p", b"");
        assert_eq!(
            authenticate(&keys(), 5_000, 1_000, &h, "GET", "/p", b""),
            Err(AuthError::UnknownApiKey)
        );

        let h = signed_headers("k1", "s1", 1_000, "GET", "/p", b"");
        assert_eq!(
            authenticate(&keys(), 5_000, 7_000, &h, "GET", "/p", b""),
            Err(AuthError::TimestampOutsideWindow)
        );
    }

    #[test]
    fn rejects_missing_headers() {
        assert_eq!(
            authenticate(&keys(), 5_000, 0, &HeaderMap::new(), "GET", "/p", b""),
            Err(AuthError::MissingHeader(HEADER_API_KEY))
        );
    }
}
