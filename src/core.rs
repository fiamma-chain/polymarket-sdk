//! Core infrastructure for the Polymarket SDK.
//!
//! This module provides fundamental building blocks used across the SDK:
//!
//! - `error` - Unified error handling with retry support
//! - `endpoints` - API endpoint management and configuration

mod endpoints;
mod error;

pub use endpoints::*;
pub use error::*;
