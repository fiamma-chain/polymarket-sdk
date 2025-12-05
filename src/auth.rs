//! Authentication and signing utilities for Polymarket APIs.
//!
//! This module provides:
//! - EIP-712 typed data signing for wallet authentication
//! - HMAC-SHA256 signing for API key authentication
//! - Builder API authentication (derived from polymarket-rs-sdk)
//!
//! # Authentication Levels
//!
//! | Level | Method | Use Case |
//! |-------|--------|----------|
//! | L1 | EIP-712 wallet signature | Create API credentials |
//! | L2 | HMAC-SHA256 with API key | Daily API operations |
//! | Builder | HMAC with Builder credentials | Relayer operations |
//!
//! # Attribution
//!
//! The Builder API signing code is derived from
//! [`polymarket-rs-sdk`](https://github.com/polymarket/polymarket-rs-sdk).

mod builder;
mod eip712;

// Builder API (from polymarket-rs-sdk)
pub use builder::{
    build_builder_hmac_signature, BuilderApiKeyCreds, BuilderSigner, HmacSha256,
};

// EIP-712 and HMAC signing
pub use eip712::{
    build_clob_auth_typed_data, build_hmac_signature, build_hmac_signature_from_string,
    create_l1_headers, create_l2_headers, create_l2_headers_with_address,
    create_l2_headers_with_body_string, get_current_unix_time_secs, sign_clob_auth_message,
    sign_order_message, ClobAuth, Headers, Order,
};
