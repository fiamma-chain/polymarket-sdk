//! REST API clients for Polymarket services.
//!
//! This module provides clients for different Polymarket APIs:
//!
//! - [`GammaClient`] - Market discovery, metadata, and public profiles
//! - [`DataClient`] - Trader data, positions, and leaderboards
//! - [`ClobClient`] - Order book operations and trading

pub mod clob;
pub mod data;
pub mod gamma;

pub use clob::{
    ApiKeyResponse, CancelResponse, ClobClient, ClobConfig, DeriveApiKeyResponse, OpenOrder,
    OrderBookLevel, OrderBookSummary, OrderResponse, PaginatedResponse,
};
pub use data::{DataClient, DataConfig};
pub use gamma::{GammaClient, GammaConfig};
