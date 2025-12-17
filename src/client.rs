//! REST API clients for Polymarket services.
//!
//! This module provides clients for different Polymarket APIs:
//!
//! - [`GammaClient`] - Market discovery and metadata
//! - [`DataClient`] - Trader data, positions, and leaderboards
//! - [`ProfilesClient`] - User profiles
//! - [`ClobClient`] - Order book operations and trading

pub mod clob;
pub mod data;
pub mod gamma;
pub mod profiles;

pub use clob::{
    ApiKeyResponse, CancelResponse, ClobClient, ClobConfig, DeriveApiKeyResponse, OpenOrder,
    OrderBookLevel, OrderBookSummary, OrderResponse, PaginatedResponse,
};
pub use data::{DataClient, DataConfig};
pub use gamma::{GammaClient, GammaConfig};
pub use profiles::{ProfilesClient, ProfilesConfig};
