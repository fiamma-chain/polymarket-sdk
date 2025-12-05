//! Builder API authentication utilities.
//!
//! This module provides HMAC-SHA256 signing for Polymarket Builder API.
//!
//! # Attribution
//!
//! This code is derived from [`builder_signing_sdk_rs`](https://github.com/polymarket/polymarket-rs-sdk/tree/main/builder_signing_sdk_rs)
//! originally developed by Polymarket, licensed under MIT OR Apache-2.0.
//!
//! # Example
//!
//! ```rust,ignore
//! use polymarket_sdk::auth::{BuilderSigner, BuilderApiKeyCreds};
//!
//! let creds = BuilderApiKeyCreds {
//!     key: "your-api-key".to_string(),
//!     secret: "base64-encoded-secret".to_string(),
//!     passphrase: "your-passphrase".to_string(),
//! };
//!
//! let signer = BuilderSigner::new(creds);
//! let headers = signer.create_builder_header_payload("POST", "/v1/order", None, None)?;
//! ```

// ============================================================================
// The following code is derived from:
// https://github.com/polymarket/polymarket-rs-sdk/tree/main/builder_signing_sdk_rs
// License: MIT OR Apache-2.0
// ============================================================================

use base64::Engine;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;

use crate::core::PolymarketError;

/// HMAC-SHA256 type alias.
pub type HmacSha256 = Hmac<Sha256>;

/// Builder API credentials for authentication.
#[derive(Clone, Debug)]
pub struct BuilderApiKeyCreds {
    /// API key identifier.
    pub key: String,
    /// Base64-encoded secret for HMAC signing.
    pub secret: String,
    /// Passphrase for additional authentication.
    pub passphrase: String,
}

impl BuilderApiKeyCreds {
    /// Create new Builder API credentials.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        secret: impl Into<String>,
        passphrase: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            secret: secret.into(),
            passphrase: passphrase.into(),
        }
    }
}

/// Build URL-safe base64 HMAC-SHA256 signature.
///
/// Creates a signature compatible with Polymarket Builder API authentication.
///
/// # Errors
///
/// Returns an error if the secret is not valid base64 or HMAC initialization fails.
pub fn build_builder_hmac_signature(
    secret_b64: &str,
    timestamp: i64,
    method: &str,
    request_path: &str,
    body: Option<&str>,
) -> crate::core::Result<String> {
    let mut message = format!("{timestamp}{method}{request_path}");
    if let Some(b) = body {
        message.push_str(b);
    }

    // Polymarket secrets are URL-safe Base64 encoded
    let secret_bytes = base64::engine::general_purpose::URL_SAFE
        .decode(secret_b64)
        .map_err(|e| PolymarketError::auth(format!("Invalid base64 secret: {e}")))?;

    let mut mac = HmacSha256::new_from_slice(&secret_bytes)
        .map_err(|e| PolymarketError::auth(format!("HMAC initialization failed: {e}")))?;

    mac.update(message.as_bytes());
    let sig_bytes = mac.finalize().into_bytes();
    let sig_b64 = base64::engine::general_purpose::STANDARD.encode(sig_bytes);

    // Make URL-safe but keep '=' padding
    Ok(sig_b64.replace('+', "-").replace('/', "_"))
}

/// Builder API request signer.
///
/// Handles authentication header generation for Polymarket Builder API requests.
#[derive(Clone)]
pub struct BuilderSigner {
    creds: BuilderApiKeyCreds,
}

impl BuilderSigner {
    /// Create a new Builder API signer with the given credentials.
    #[must_use]
    pub fn new(creds: BuilderApiKeyCreds) -> Self {
        Self { creds }
    }

    /// Get the API key.
    #[must_use]
    pub fn api_key(&self) -> &str {
        &self.creds.key
    }

    /// Create authentication headers for a Builder API request.
    ///
    /// Returns a map of header names to values that should be included
    /// in the HTTP request.
    ///
    /// # Errors
    ///
    /// Returns an error if signature generation fails.
    pub fn create_builder_header_payload(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
        timestamp: Option<i64>,
    ) -> crate::core::Result<HashMap<String, String>> {
        let ts = timestamp.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64
        });

        let sig = build_builder_hmac_signature(&self.creds.secret, ts, method, path, body)?;

        Ok(HashMap::from([
            ("POLY_BUILDER_API_KEY".to_string(), self.creds.key.clone()),
            (
                "POLY_BUILDER_PASSPHRASE".to_string(),
                self.creds.passphrase.clone(),
            ),
            ("POLY_BUILDER_SIGNATURE".to_string(), sig),
            ("POLY_BUILDER_TIMESTAMP".to_string(), ts.to_string()),
        ]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_signature_url_safe() {
        let secret_b64 = base64::engine::general_purpose::STANDARD.encode(b"secret");
        let sig = build_builder_hmac_signature(
            &secret_b64,
            1_700_000_000,
            "POST",
            "/v1/x",
            Some(r#"{"a":1}"#),
        )
        .expect("signature should succeed");

        // Should not contain '+' or '/'
        assert!(!sig.contains('+'));
        assert!(!sig.contains('/'));
    }

    #[test]
    fn test_builder_signer() {
        let creds = BuilderApiKeyCreds::new(
            "test-key",
            base64::engine::general_purpose::URL_SAFE.encode(b"test-secret"),
            "test-passphrase",
        );

        let signer = BuilderSigner::new(creds);
        let headers = signer
            .create_builder_header_payload("GET", "/v1/test", None, Some(1_700_000_000))
            .expect("header generation should succeed");

        assert_eq!(headers.get("POLY_BUILDER_API_KEY").unwrap(), "test-key");
        assert_eq!(
            headers.get("POLY_BUILDER_PASSPHRASE").unwrap(),
            "test-passphrase"
        );
        assert_eq!(
            headers.get("POLY_BUILDER_TIMESTAMP").unwrap(),
            "1700000000"
        );
        assert!(headers.contains_key("POLY_BUILDER_SIGNATURE"));
    }
}
