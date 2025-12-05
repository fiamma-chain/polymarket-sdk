//! # Polymarket SDK
//!
//! A comprehensive Rust SDK for [Polymarket](https://polymarket.com) prediction markets.
//!
//! ## Attribution
//!
//! Parts of this SDK are derived from official Polymarket open-source projects:
//!
//! - **Builder API signing**: Derived from [`polymarket-rs-sdk`](https://github.com/polymarket/polymarket-rs-sdk)
//! - **API patterns**: Inspired by [`py-clob-client`](https://github.com/Polymarket/py-clob-client)
//!   and [`clob-client`](https://github.com/Polymarket/clob-client)
//!
//! ## Features
//!
//! - **Market Discovery** - Query markets, events, and metadata
//! - **Real-time Data** - WebSocket streams for trades and order books
//! - **Order Management** - Create, sign, and submit orders
//! - **Authentication** - EIP-712 and HMAC-based auth
//! - **Safe Wallet** - Gnosis Safe proxy wallet integration
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use polymarket_sdk::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Market discovery (no auth required)
//!     let client = GammaClient::new(Default::default())?;
//!     let markets = client.get_markets(None).await?;
//!     println!("Found {} markets", markets.len());
//!     Ok(())
//! }
//! ```
//!
//! ## Module Organization
//!
//! - [`core`] - Error handling and endpoint configuration
//! - [`types`] - Common types (Side, Market, Order, etc.)
//! - [`auth`] - Authentication (EIP-712, HMAC, Builder API)
//! - [`client`] - REST API clients (Gamma, Data, CLOB, Profiles)
//! - [`order`] - Order creation and signing
//! - [`stream`] - WebSocket streaming (RTDS, CLOB WSS)
//! - [`safe`] - Safe wallet deployment and management

#![cfg_attr(docsrs, feature(doc_cfg))]

// Core infrastructure
pub mod core;

// Type definitions
pub mod types;

// Authentication
#[cfg(feature = "auth")]
#[cfg_attr(docsrs, doc(cfg(feature = "auth")))]
pub mod auth;

// API clients
#[cfg(feature = "client")]
#[cfg_attr(docsrs, doc(cfg(feature = "client")))]
pub mod client;

// Order building
#[cfg(feature = "order")]
#[cfg_attr(docsrs, doc(cfg(feature = "order")))]
pub mod order;

// WebSocket streams
#[cfg(feature = "stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
pub mod stream;

// Safe wallet
#[cfg(feature = "safe")]
#[cfg_attr(docsrs, doc(cfg(feature = "safe")))]
pub mod safe;

// Prelude for convenient imports
pub mod prelude;

// ============================================================================
// Core Re-exports (always available)
// ============================================================================

pub use core::{Error, Endpoints, PolymarketError, Result};
pub use core::{
    AuthErrorKind, MarketDataErrorKind, OrderErrorKind, StreamErrorKind,
    CLOB_API_BASE, CLOB_WSS_BASE, DATA_API_BASE, GAMMA_API_BASE,
    PROFILES_API_BASE, RELAYER_API_BASE, RTDS_WSS_BASE,
};

// ============================================================================
// Type Re-exports (always available)
// ============================================================================

pub use types::{
    ApiCredentials, BiggestWinner, BiggestWinnersQuery, BookLevel, ClosedPosition,
    ConnectionStats, DataApiActivity, DataApiPosition, DataApiTrade, DataApiTrader,
    Event, EventMarket, LeaderboardEntry, ListParams, Market, MarketOrderArgs,
    NewOrder, NewOrderData, OrderOptions, OrderType, PaginationParams, SearchEvent,
    SearchProfile, SearchRequest, SearchResponse, SearchTag, Side, SignedOrderRequest,
    Tag, Token, TraderProfile,
};

#[cfg(feature = "auth")]
pub use types::ExtraOrderArgs;

// ============================================================================
// Auth Re-exports
// ============================================================================

#[cfg(feature = "auth")]
pub use auth::{
    build_clob_auth_typed_data, build_hmac_signature, build_hmac_signature_from_string,
    create_l1_headers, create_l2_headers, create_l2_headers_with_address,
    create_l2_headers_with_body_string, get_current_unix_time_secs, sign_clob_auth_message,
    sign_order_message, BuilderApiKeyCreds, BuilderSigner, ClobAuth, Headers, Order,
};

// ============================================================================
// Client Re-exports
// ============================================================================

#[cfg(feature = "client")]
pub use client::{
    ApiKeyResponse, CancelResponse, ClobClient, ClobConfig, DataClient, DataConfig,
    DeriveApiKeyResponse, GammaClient, GammaConfig, OpenOrder, OrderResponse,
    ProfilesClient, ProfilesConfig,
};

// ============================================================================
// Order Re-exports
// ============================================================================

#[cfg(feature = "order")]
pub use order::{get_contract_config, ContractConfig, OrderArgs, OrderBuilder, SigType};

// ============================================================================
// Stream Re-exports
// ============================================================================

#[cfg(feature = "stream")]
pub use stream::{
    LastTradeMessage, MarketBook, MarketStream, MockStream, PriceChangeEntry,
    PriceChangeMessage, RtdsClient, RtdsConfig, RtdsEvent, RtdsMessage,
    RtdsSubscription, RtdsSubscriptionMessage, StreamManager, StreamMessage,
    StreamStats, Subscription, TickSizeChangeMessage, TradePayload, WebSocketStream,
    WssAuth, WssMarketClient, WssMarketEvent, WssStats, WssSubscription,
    WssUserClient, WssUserEvent, WssUserOrderMessage, WssUserTradeMessage,
};

// ============================================================================
// Safe Re-exports
// ============================================================================

#[cfg(feature = "safe")]
pub use safe::{
    build_ctf_approve_typed_data, build_safe_create_typed_data, build_safe_tx_request,
    build_token_approve_typed_data, build_usdc_transfer_typed_data, compute_safe_tx_digest,
    derive_safe_address, encode_erc1155_set_approval_for_all, encode_erc20_allowance_query,
    encode_erc20_approve, encode_erc20_transfer, pack_signature, pack_signature_for_safe_tx,
    ApprovalStatus, DeploySafeResponse, NonceType, RelayerClient, RelayerConfig,
    SafeCreateTypedData, SafeTxDomain, SafeTxMessage, SafeTxTypedData, SafeTxTypes,
    SignatureParams, TransactionReceipt, TransactionRequest, TransactionState, TransactionType,
    CONDITIONAL_TOKENS_ADDRESS, CTF_EXCHANGE_ADDRESS, EXCHANGE_ADDRESS,
    NATIVE_USDC_CONTRACT_ADDRESS, NEG_RISK_CTF_EXCHANGE_ADDRESS, SAFE_FACTORY,
    SAFE_INIT_CODE_HASH, USDC_CONTRACT_ADDRESS,
};

// ============================================================================
// Backward-compatible Module Aliases (for ride-service migration)
// ============================================================================

/// Backward-compatible alias for `client` module
#[cfg(feature = "client")]
pub mod clob {
    //! Backward-compatible module alias for CLOB client.
    //! Use `polymarket_sdk::client` or top-level re-exports instead.
    pub use crate::client::{ClobClient, ClobConfig};
}

/// Backward-compatible alias for Gamma API client
#[cfg(feature = "client")]
pub mod gamma {
    //! Backward-compatible module alias for Gamma API client.
    //! Use `polymarket_sdk::client` or top-level re-exports instead.
    pub use crate::client::{GammaClient, GammaConfig};
}

/// Backward-compatible alias for Data API client
#[cfg(feature = "client")]
pub mod data {
    //! Backward-compatible module alias for Data API client.
    //! Use `polymarket_sdk::client` or top-level re-exports instead.
    pub use crate::client::{DataClient, DataConfig};
}

/// Backward-compatible alias for `order` module
#[cfg(feature = "order")]
pub mod orders {
    //! Backward-compatible module alias for order building.
    //! Use `polymarket_sdk::order` or top-level re-exports instead.
    pub use crate::order::{get_contract_config, ContractConfig, OrderArgs, OrderBuilder, SigType};
}

/// Backward-compatible alias for CLOB WebSocket client
#[cfg(feature = "stream")]
pub mod wss {
    //! Backward-compatible module alias for CLOB WebSocket functionality.
    //! Use `polymarket_sdk::stream` or top-level re-exports instead.
    pub use crate::stream::{
        LastTradeMessage, MarketBook, OrderSummary, PriceChangeEntry, PriceChangeMessage,
        TickSizeChangeMessage, WssMarketClient, WssMarketEvent, WssStats, WssUserClient,
        WssUserEvent, WssUserOrderMessage, WssUserTradeMessage,
    };
}

/// Backward-compatible alias for `safe` module
#[cfg(feature = "safe")]
pub mod relayer {
    //! Backward-compatible module alias for relayer/safe functionality.
    //! Use `polymarket_sdk::safe` or top-level re-exports instead.

    pub use crate::safe::{
        build_ctf_approve_typed_data, build_safe_create_typed_data, build_safe_tx_request,
        build_token_approve_typed_data, build_usdc_transfer_typed_data, compute_safe_tx_digest,
        derive_safe_address, encode_erc1155_set_approval_for_all, encode_erc20_allowance_query,
        encode_erc20_approve, encode_erc20_transfer, pack_signature, pack_signature_for_safe_tx,
        ApprovalStatus, BuilderApiCredentials, DeploySafeResponse, NonceType, RelayerClient,
        RelayerConfig, SafeCreateTypedData, SafeTxDomain, SafeTxMessage, SafeTxTypedData,
        SafeTxTypes, SignatureParams, TransactionReceipt, TransactionRequest, TransactionState,
        TransactionType, CONDITIONAL_TOKENS_ADDRESS, CTF_EXCHANGE_ADDRESS, EXCHANGE_ADDRESS,
        NATIVE_USDC_CONTRACT_ADDRESS, NEG_RISK_CTF_EXCHANGE_ADDRESS, SAFE_FACTORY,
        SAFE_INIT_CODE_HASH, USDC_CONTRACT_ADDRESS,
    };
}

/// Backward-compatible alias for `core` error types
pub mod errors {
    //! Backward-compatible module alias for error types.
    //! Use `polymarket_sdk::core` or top-level re-exports instead.
    pub use crate::core::{
        AuthErrorKind, Error, MarketDataErrorKind, OrderErrorKind, PolymarketError, Result,
        StreamErrorKind,
    };
}
