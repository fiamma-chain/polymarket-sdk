//! Authentication and cryptographic utilities for Polymarket API
//!
//! This module provides EIP-712 signing, HMAC authentication, and header generation
//! for secure communication with the Polymarket CLOB API.
//!
//! ## Features
//!
//! - **L1 Authentication**: EIP-712 signature-based authentication using private keys
//! - **L2 Authentication**: HMAC-based authentication using API credentials
//! - **Order Signing**: EIP-712 order signatures for CLOB trading
//!
//! ## Example
//!
//! ```rust,ignore
//! use polymarket_sdk::auth::{create_l1_headers, create_l2_headers};
//! use alloy_signer_local::PrivateKeySigner;
//!
//! // L1 authentication (wallet-based)
//! let signer: PrivateKeySigner = "0x...".parse()?;
//! let headers = create_l1_headers(&signer, None)?;
//!
//! // L2 authentication (API key-based)
//! let api_creds = ApiCredentials { ... };
//! let headers = create_l2_headers(&signer, &api_creds, "GET", "/orders", None::<&str>)?;
//! ```

use crate::core::{PolymarketError, Result};
use crate::types::ApiCredentials;
use alloy_primitives::{hex::encode_prefixed, Address, U256};
use alloy_signer::SignerSync;
use alloy_signer_local::PrivateKeySigner;
use alloy_sol_types::{eip712_domain, sol};
use base64::engine::general_purpose::{STANDARD, URL_SAFE, URL_SAFE_NO_PAD};
use base64::engine::Engine;
use hmac::{Hmac, Mac};
use serde::Serialize;
use sha2::Sha256;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// Header constants for Polymarket API
const POLY_ADDR_HEADER: &str = "poly_address";
const POLY_SIG_HEADER: &str = "poly_signature";
const POLY_TS_HEADER: &str = "poly_timestamp";
const POLY_NONCE_HEADER: &str = "poly_nonce";
const POLY_API_KEY_HEADER: &str = "poly_api_key";
const POLY_PASS_HEADER: &str = "poly_passphrase";

/// Header map type alias
pub type Headers = HashMap<&'static str, String>;

// Solidity struct definitions for EIP-712 signing

sol! {
    /// CLOB authentication message struct
    struct ClobAuth {
        address address;
        string timestamp;
        uint256 nonce;
        string message;
    }
}

sol! {
    /// Order struct for EIP-712 signing
    struct Order {
        uint256 salt;
        address maker;
        address signer;
        address taker;
        uint256 tokenId;
        uint256 makerAmount;
        uint256 takerAmount;
        uint256 expiration;
        uint256 nonce;
        uint256 feeRateBps;
        uint8 side;
        uint8 signatureType;
    }
}

/// Decode API secret from various base64 formats
fn decode_api_secret(secret: &str) -> Vec<u8> {
    URL_SAFE
        .decode(secret)
        .or_else(|_| URL_SAFE_NO_PAD.decode(secret))
        .or_else(|_| STANDARD.decode(secret))
        .unwrap_or_else(|_| secret.as_bytes().to_vec())
}

/// Format body for signature calculation
fn format_body_for_signature<T>(body: &T) -> Result<String>
where
    T: ?Sized + Serialize,
{
    serde_json::to_string(body).map_err(|e| PolymarketError::Parse {
        message: format!("Failed to serialize body: {}", e),
        source: Some(Box::new(e)),
    })
}

/// Get current Unix timestamp in seconds
#[must_use]
pub fn get_current_unix_time_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

/// Build EIP-712 typed data for CLOB auth (for external signing via Privy ServerWallet)
///
/// Constructs the EIP-712 typed data structure that needs to be signed for `/derive-api-key` endpoint.
/// Use this when you need to sign via Privy ServerWallet's `signTypedData` API.
///
/// # Arguments
///
/// * `address` - Wallet address (without 0x prefix)
/// * `timestamp` - Unix timestamp as string
/// * `nonce` - Nonce value for replay protection
///
/// # Returns
///
/// JSON-serializable EIP-712 typed data ready for Privy ServerWallet API
///
/// # Example
///
/// ```rust,ignore
/// use alloy_primitives::Address;
///
/// let address = Address::from_str("0x1234...").unwrap();
/// let timestamp = get_current_unix_time_secs().to_string();
/// let nonce = U256::ZERO;
///
/// let typed_data = build_clob_auth_typed_data(address, &timestamp, nonce);
///
/// // Then send this to Privy ServerWallet for EIP-712 signing
/// // POST /wallets/{wallet_id}/sign_typed_data
/// // { "params": typed_data }
/// ```
pub fn build_clob_auth_typed_data(
    address: Address,
    timestamp: &str,
    nonce: U256,
) -> serde_json::Value {
    use serde_json::json;

    let message_content = "This message attests that I control the given wallet";
    let polygon_chain_id = 137;

    json!({
        "domain": {
            "name": "ClobAuthDomain",
            "version": "1",
            "chainId": polygon_chain_id
        },
        "types": {
            "EIP712Domain": [
                { "name": "name", "type": "string" },
                { "name": "version", "type": "string" },
                { "name": "chainId", "type": "uint256" }
            ],
            "ClobAuth": [
                { "name": "address", "type": "address" },
                { "name": "timestamp", "type": "string" },
                { "name": "nonce", "type": "uint256" },
                { "name": "message", "type": "string" }
            ]
        },
        "primaryType": "ClobAuth",
        "message": {
            "address": format!("{:?}", address),
            "timestamp": timestamp,
            "nonce": nonce.to_string(),
            "message": message_content
        }
    })
}

/// Sign CLOB authentication message using EIP-712
///
/// This creates a signature that proves ownership of a wallet address
/// for authenticating with the Polymarket CLOB API.
///
/// # Arguments
///
/// * `signer` - Private key signer
/// * `timestamp` - Unix timestamp as string
/// * `nonce` - Nonce value for replay protection
///
/// # Returns
///
/// Hex-encoded signature prefixed with "0x"
pub fn sign_clob_auth_message(
    signer: &PrivateKeySigner,
    timestamp: String,
    nonce: U256,
) -> Result<String> {
    let message = "This message attests that I control the given wallet".to_string();
    let polygon_chain_id = 137;

    let auth_struct = ClobAuth {
        address: signer.address(),
        timestamp,
        nonce,
        message,
    };

    let domain = eip712_domain!(
        name: "ClobAuthDomain",
        version: "1",
        chain_id: polygon_chain_id,
    );

    let signature = signer
        .sign_typed_data_sync(&auth_struct, &domain)
        .map_err(|e| PolymarketError::crypto(format!("EIP-712 signature failed: {}", e)))?;

    Ok(encode_prefixed(signature.as_bytes()))
}

/// Sign order message using EIP-712
///
/// Creates a signature for submitting orders to the Polymarket CLOB.
///
/// # Arguments
///
/// * `signer` - Private key signer
/// * `order` - Order struct to sign
/// * `chain_id` - Blockchain chain ID (137 for Polygon)
/// * `verifying_contract` - Exchange contract address
///
/// # Returns
///
/// Hex-encoded signature prefixed with "0x"
pub fn sign_order_message(
    signer: &PrivateKeySigner,
    order: Order,
    chain_id: u64,
    verifying_contract: Address,
) -> Result<String> {
    let domain = eip712_domain!(
        name: "Polymarket CTF Exchange",
        version: "1",
        chain_id: chain_id,
        verifying_contract: verifying_contract,
    );

    let signature = signer
        .sign_typed_data_sync(&order, &domain)
        .map_err(|e| PolymarketError::crypto(format!("Order signature failed: {}", e)))?;

    Ok(encode_prefixed(signature.as_bytes()))
}

/// Build HMAC signature for L2 authentication
///
/// Creates an HMAC-SHA256 signature using API credentials for
/// authenticating API requests.
///
/// # Arguments
///
/// * `secret` - API secret (base64 encoded)
/// * `timestamp` - Unix timestamp
/// * `method` - HTTP method (GET, POST, etc.)
/// * `request_path` - API endpoint path
/// * `body` - Optional request body
///
/// # Returns
///
/// Base64-encoded HMAC signature
pub fn build_hmac_signature<T>(
    secret: &str,
    timestamp: u64,
    method: &str,
    request_path: &str,
    body: Option<&T>,
) -> Result<String>
where
    T: ?Sized + Serialize,
{
    let mut mac = Hmac::<Sha256>::new_from_slice(&decode_api_secret(secret))
        .map_err(|e| PolymarketError::crypto(format!("Invalid HMAC key: {}", e)))?;

    // Build the message to sign: timestamp + method + path + body
    let body_string = match body {
        Some(b) => format_body_for_signature(b)?,
        None => String::new(),
    };
    let message = format!(
        "{}{}{}{}",
        timestamp,
        method.to_uppercase(),
        request_path,
        body_string
    );

    mac.update(message.as_bytes());
    let result = mac.finalize();
    Ok(URL_SAFE.encode(result.into_bytes()))
}

/// Create L1 headers for authentication (using private key signature)
///
/// L1 authentication uses EIP-712 signatures to prove wallet ownership.
/// This is used for operations that require proof of wallet control.
///
/// # Arguments
///
/// * `signer` - Private key signer
/// * `nonce` - Optional nonce (defaults to 0)
///
/// # Returns
///
/// HashMap containing authentication headers
pub fn create_l1_headers(signer: &PrivateKeySigner, nonce: Option<U256>) -> Result<Headers> {
    let timestamp = get_current_unix_time_secs().to_string();
    let nonce = nonce.unwrap_or(U256::ZERO);
    let signature = sign_clob_auth_message(signer, timestamp.clone(), nonce)?;
    let address = encode_prefixed(signer.address().as_slice());

    Ok(HashMap::from([
        (POLY_ADDR_HEADER, address),
        (POLY_SIG_HEADER, signature),
        (POLY_TS_HEADER, timestamp),
        (POLY_NONCE_HEADER, nonce.to_string()),
    ]))
}

/// Create L2 headers for API calls (using API key and HMAC)
///
/// L2 authentication uses HMAC signatures with API credentials.
/// This is the standard authentication method for most API operations.
///
/// # Arguments
///
/// * `signer` - Private key signer (for address)
/// * `api_creds` - API credentials (key, secret, passphrase)
/// * `method` - HTTP method
/// * `req_path` - Request path
/// * `body` - Optional request body
///
/// # Returns
///
/// HashMap containing authentication headers
pub fn create_l2_headers<T>(
    signer: &PrivateKeySigner,
    api_creds: &ApiCredentials,
    method: &str,
    req_path: &str,
    body: Option<&T>,
) -> Result<Headers>
where
    T: ?Sized + Serialize,
{
    let address = encode_prefixed(signer.address().as_slice());
    create_l2_headers_with_address(&address, api_creds, method, req_path, body)
}

/// Create L2 headers with explicit address (for Builder API auth)
///
/// This variant allows specifying the address directly, useful when using
/// Builder API credentials where the signer and API key owner are different.
///
/// # Arguments
///
/// * `address` - Wallet address (hex string with or without 0x prefix)
/// * `api_creds` - API credentials (key, secret, passphrase)
/// * `method` - HTTP method
/// * `req_path` - Request path
/// * `body` - Optional request body
///
/// # Returns
///
/// HashMap containing authentication headers
pub fn create_l2_headers_with_address<T>(
    address: &str,
    api_creds: &ApiCredentials,
    method: &str,
    req_path: &str,
    body: Option<&T>,
) -> Result<Headers>
where
    T: ?Sized + Serialize,
{
    // Ensure address has 0x prefix
    let address = if address.starts_with("0x") {
        address.to_string()
    } else {
        format!("0x{}", address)
    };

    let timestamp = get_current_unix_time_secs();

    let hmac_signature =
        build_hmac_signature(&api_creds.secret, timestamp, method, req_path, body)?;

    Ok(HashMap::from([
        (POLY_ADDR_HEADER, address),
        (POLY_SIG_HEADER, hmac_signature),
        (POLY_TS_HEADER, timestamp.to_string()),
        (POLY_API_KEY_HEADER, api_creds.api_key.clone()),
        (POLY_PASS_HEADER, api_creds.passphrase.clone()),
    ]))
}

/// Create L2 headers with pre-serialized body string and explicit timestamp
///
/// This variant ensures that:
/// 1. The same JSON string is used for HMAC calculation and HTTP body
/// 2. L2 and Builder headers use the same timestamp
///
/// # Arguments
///
/// * `address` - Wallet address
/// * `api_creds` - API credentials
/// * `method` - HTTP method
/// * `req_path` - Request path
/// * `body_str` - Pre-serialized JSON body string
/// * `timestamp` - Unix timestamp (same timestamp should be passed to Builder headers)
///
/// # Returns
///
/// Tuple of (Headers, timestamp) - use the returned timestamp for Builder headers
pub fn create_l2_headers_with_body_string(
    address: &str,
    api_creds: &ApiCredentials,
    method: &str,
    req_path: &str,
    body_str: &str,
    timestamp: u64,
) -> Result<Headers> {
    // Ensure address has 0x prefix
    let address = if address.starts_with("0x") {
        address.to_string()
    } else {
        format!("0x{}", address)
    };

    let hmac_signature =
        build_hmac_signature_from_string(&api_creds.secret, timestamp, method, req_path, body_str)?;

    Ok(HashMap::from([
        (POLY_ADDR_HEADER, address),
        (POLY_SIG_HEADER, hmac_signature),
        (POLY_TS_HEADER, timestamp.to_string()),
        (POLY_API_KEY_HEADER, api_creds.api_key.clone()),
        (POLY_PASS_HEADER, api_creds.passphrase.clone()),
    ]))
}

/// Build HMAC signature from pre-serialized body string
///
/// This ensures the exact same JSON string is used for signature as HTTP body
pub fn build_hmac_signature_from_string(
    secret: &str,
    timestamp: u64,
    method: &str,
    request_path: &str,
    body_str: &str,
) -> Result<String> {
    let mut mac = Hmac::<Sha256>::new_from_slice(&decode_api_secret(secret))
        .map_err(|e| PolymarketError::crypto(format!("Invalid HMAC key: {}", e)))?;

    let message = format!(
        "{}{}{}{}",
        timestamp,
        method.to_uppercase(),
        request_path,
        body_str
    );

    mac.update(message.as_bytes());
    let result = mac.finalize();
    Ok(URL_SAFE.encode(result.into_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_unix_timestamp() {
        let timestamp = get_current_unix_time_secs();
        assert!(timestamp > 1_600_000_000); // Should be after 2020
    }

    #[test]
    fn test_hmac_signature() {
        let result =
            build_hmac_signature::<String>("test_secret", 1234567890, "GET", "/test", None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_hmac_signature_with_body() {
        let body = r#"{"test": "data"}"#;
        let result = build_hmac_signature("test_secret", 1234567890, "POST", "/orders", Some(body));
        assert!(result.is_ok());
        let signature = result.unwrap();
        assert!(!signature.is_empty());
    }

    #[test]
    fn test_hmac_signature_consistency() {
        let secret = "test_secret";
        let timestamp = 1234567890;
        let method = "GET";
        let path = "/test";

        let sig1 = build_hmac_signature::<String>(secret, timestamp, method, path, None).unwrap();
        let sig2 = build_hmac_signature::<String>(secret, timestamp, method, path, None).unwrap();

        // Same inputs should produce same signature
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_hmac_signature_different_inputs() {
        let secret = "test_secret";
        let timestamp = 1234567890;

        let sig1 = build_hmac_signature::<String>(secret, timestamp, "GET", "/test", None).unwrap();
        let sig2 =
            build_hmac_signature::<String>(secret, timestamp, "POST", "/test", None).unwrap();
        let sig3 =
            build_hmac_signature::<String>(secret, timestamp, "GET", "/other", None).unwrap();

        // Different inputs should produce different signatures
        assert_ne!(sig1, sig2);
        assert_ne!(sig1, sig3);
        assert_ne!(sig2, sig3);
    }

    #[test]
    fn test_decode_api_secret_with_urlsafe_padding() {
        assert_eq!(decode_api_secret("cQ=="), b"q".to_vec());
    }

    #[test]
    fn test_format_body_for_signature_json() {
        let body = json!({ "order": { "foo": 1 } });
        let formatted = format_body_for_signature(&body).expect("Formatting should succeed");
        assert_eq!(formatted, r#"{"order":{"foo":1}}"#);
    }

    #[test]
    fn test_create_l1_headers() {
        let private_key = "0x1234567890123456789012345678901234567890123456789012345678901234";
        let signer: PrivateKeySigner = private_key.parse().expect("Valid private key");

        let result = create_l1_headers(&signer, Some(U256::from(12345)));
        assert!(result.is_ok());

        let headers = result.unwrap();
        assert!(headers.contains_key("poly_address"));
        assert!(headers.contains_key("poly_signature"));
        assert!(headers.contains_key("poly_timestamp"));
        assert!(headers.contains_key("poly_nonce"));
    }

    #[test]
    fn test_create_l1_headers_different_nonces() {
        let private_key = "0x1234567890123456789012345678901234567890123456789012345678901234";
        let signer: PrivateKeySigner = private_key.parse().expect("Valid private key");

        let headers_1 = create_l1_headers(&signer, Some(U256::from(12345))).unwrap();
        let headers_2 = create_l1_headers(&signer, Some(U256::from(54321))).unwrap();

        // Different nonces should produce different signatures
        assert_ne!(
            headers_1.get("poly_signature"),
            headers_2.get("poly_signature")
        );

        // But same address
        assert_eq!(headers_1.get("poly_address"), headers_2.get("poly_address"));
    }

    #[test]
    fn test_create_l2_headers() {
        let private_key = "0x1234567890123456789012345678901234567890123456789012345678901234";
        let signer: PrivateKeySigner = private_key.parse().expect("Valid private key");

        let api_creds = ApiCredentials {
            api_key: "test_key".to_string(),
            secret: "test_secret".to_string(),
            passphrase: "test_passphrase".to_string(),
        };

        let result = create_l2_headers::<String>(&signer, &api_creds, "GET", "/test", None);
        assert!(result.is_ok());

        let headers = result.unwrap();
        assert!(headers.contains_key("poly_api_key"));
        assert!(headers.contains_key("poly_signature"));
        assert!(headers.contains_key("poly_timestamp"));
        assert!(headers.contains_key("poly_passphrase"));

        assert_eq!(headers.get("poly_api_key").unwrap(), "test_key");
        assert_eq!(headers.get("poly_passphrase").unwrap(), "test_passphrase");
    }

    #[test]
    fn test_eip712_signature_format() {
        let private_key = "0x1234567890123456789012345678901234567890123456789012345678901234";
        let signer: PrivateKeySigner = private_key.parse().expect("Valid private key");

        let result = create_l1_headers(&signer, Some(U256::from(12345)));
        assert!(result.is_ok());

        let headers = result.unwrap();
        let signature = headers.get("poly_signature").unwrap();

        // EIP-712 signatures should be hex strings starting with 0x
        assert!(signature.starts_with("0x"));
        // 65 bytes = 130 hex chars + 2 for "0x" = 132 total
        assert_eq!(signature.len(), 132);
    }
}
