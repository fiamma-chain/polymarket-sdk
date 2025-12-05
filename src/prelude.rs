//! Commonly used types for quick imports.
//!
//! This module re-exports the most commonly used types from the SDK
//! for convenient importing.
//!
//! # Example
//!
//! ```rust,ignore
//! use polymarket_sdk::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let client = GammaClient::new(Default::default())?;
//!     let markets = client.get_markets(None).await?;
//!     Ok(())
//! }
//! ```

// Core
pub use crate::core::{Endpoints, Error, PolymarketError, Result};

// Common types
pub use crate::types::{
    ApiCredentials, BookLevel, Event, ListParams, Market, OrderOptions, OrderType,
    PaginationParams, SearchRequest, SearchResponse, Side, SignedOrderRequest, Token,
    TraderProfile,
};

// Clients (if enabled)
#[cfg(feature = "client")]
pub use crate::client::{
    ClobClient, ClobConfig, DataClient, DataConfig, GammaClient, GammaConfig,
    ProfilesClient, ProfilesConfig,
};

// Auth (if enabled)
#[cfg(feature = "auth")]
pub use crate::auth::{
    create_l1_headers, create_l2_headers, sign_clob_auth_message, sign_order_message,
    BuilderApiKeyCreds, BuilderSigner, Headers,
};

// Orders (if enabled)
#[cfg(feature = "order")]
pub use crate::order::{ContractConfig, OrderArgs, OrderBuilder, SigType};

// Streams (if enabled)
#[cfg(feature = "stream")]
pub use crate::stream::{
    RtdsClient, RtdsConfig, RtdsEvent, TradePayload, WssMarketClient, WssMarketEvent,
};

// Safe (if enabled)
#[cfg(feature = "safe")]
pub use crate::safe::{derive_safe_address, RelayerClient, RelayerConfig};
