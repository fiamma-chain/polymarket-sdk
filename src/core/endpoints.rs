//! Polymarket API Endpoints.
//!
//! Centralized management of all Polymarket API URLs and WebSocket addresses.
//!
//! ## REST APIs
//! - **Gamma API**: Market discovery, events, tags (`gamma-api.polymarket.com`)
//! - **Data API**: Trader profiles, activity, positions (`data-api.polymarket.com`)
//! - **CLOB API**: Order book operations, trading (`clob.polymarket.com`)
//! - **Profiles API**: Public user profiles (`polymarket.com/api`)
//! - **Relayer API**: Safe wallet deployment (`relayer-v2.polymarket.com`)
//!
//! ## WebSocket Streams
//! - **RTDS**: Real-Time Data Stream for live trades
//! - **CLOB WSS**: Order book updates, price changes
//!
//! ## Environment Variables
//!
//! All endpoints can be overridden via environment variables:
//! - `POLYMARKET_GAMMA_URL`
//! - `POLYMARKET_DATA_URL`
//! - `POLYMARKET_CLOB_URL`
//! - `POLYMARKET_PROFILES_URL`
//! - `POLYMARKET_RELAYER_URL`
//! - `POLYMARKET_RTDS_URL`
//! - `POLYMARKET_WSS_URL`

// ============================================================================
// REST API Endpoints
// ============================================================================

/// Gamma API base URL - Market discovery, events, and tags.
pub const GAMMA_API_BASE: &str = "https://gamma-api.polymarket.com";

/// Data API base URL - Trader profiles, activity, positions, leaderboards.
pub const DATA_API_BASE: &str = "https://data-api.polymarket.com";

/// CLOB API base URL - Order book operations, trading.
pub const CLOB_API_BASE: &str = "https://clob.polymarket.com";

/// Profiles API base URL - Public user profiles.
pub const PROFILES_API_BASE: &str = "https://polymarket.com/api";

/// Relayer API base URL - Safe wallet deployment and proxy wallet management (v2).
pub const RELAYER_API_BASE: &str = "https://relayer-v2.polymarket.com";

// ============================================================================
// WebSocket Endpoints
// ============================================================================

/// RTDS WebSocket URL - Real-Time Data Stream for live trades.
pub const RTDS_WSS_BASE: &str = "wss://ws-live-data.polymarket.com";

/// CLOB WebSocket URL - Order book updates, price changes, user events.
pub const CLOB_WSS_BASE: &str = "wss://ws-subscriptions-clob.polymarket.com";

// ============================================================================
// Environment Variable Names
// ============================================================================

/// Environment variable name for Gamma API URL.
pub const ENV_GAMMA_URL: &str = "POLYMARKET_GAMMA_URL";

/// Environment variable name for Data API URL.
pub const ENV_DATA_URL: &str = "POLYMARKET_DATA_URL";

/// Environment variable name for CLOB API URL.
pub const ENV_CLOB_URL: &str = "POLYMARKET_CLOB_URL";

/// Environment variable name for Profiles API URL.
pub const ENV_PROFILES_URL: &str = "POLYMARKET_PROFILES_URL";

/// Environment variable name for Relayer API URL.
pub const ENV_RELAYER_URL: &str = "POLYMARKET_RELAYER_URL";

/// Environment variable name for RTDS WebSocket URL.
pub const ENV_RTDS_URL: &str = "POLYMARKET_RTDS_URL";

/// Environment variable name for CLOB WebSocket URL.
pub const ENV_WSS_URL: &str = "POLYMARKET_WSS_URL";

// ============================================================================
// Helper Functions
// ============================================================================

/// Get Gamma API URL from environment or use default.
#[must_use]
pub fn gamma_api_url() -> String {
    std::env::var(ENV_GAMMA_URL).unwrap_or_else(|_| GAMMA_API_BASE.to_string())
}

/// Get Data API URL from environment or use default.
#[must_use]
pub fn data_api_url() -> String {
    std::env::var(ENV_DATA_URL).unwrap_or_else(|_| DATA_API_BASE.to_string())
}

/// Get CLOB API URL from environment or use default.
#[must_use]
pub fn clob_api_url() -> String {
    std::env::var(ENV_CLOB_URL).unwrap_or_else(|_| CLOB_API_BASE.to_string())
}

/// Get Profiles API URL from environment or use default.
#[must_use]
pub fn profiles_api_url() -> String {
    std::env::var(ENV_PROFILES_URL).unwrap_or_else(|_| PROFILES_API_BASE.to_string())
}

/// Get Relayer API URL from environment or use default.
#[must_use]
pub fn relayer_api_url() -> String {
    std::env::var(ENV_RELAYER_URL).unwrap_or_else(|_| RELAYER_API_BASE.to_string())
}

/// Get RTDS WebSocket URL from environment or use default.
#[must_use]
pub fn rtds_wss_url() -> String {
    std::env::var(ENV_RTDS_URL).unwrap_or_else(|_| RTDS_WSS_BASE.to_string())
}

/// Get CLOB WebSocket URL from environment or use default.
#[must_use]
pub fn clob_wss_url() -> String {
    std::env::var(ENV_WSS_URL).unwrap_or_else(|_| CLOB_WSS_BASE.to_string())
}

// ============================================================================
// Endpoint Collection
// ============================================================================

/// All Polymarket endpoints with environment variable overrides.
#[derive(Debug, Clone)]
pub struct Endpoints {
    /// Gamma API base URL
    pub gamma_api: String,
    /// Data API base URL
    pub data_api: String,
    /// CLOB API base URL
    pub clob_api: String,
    /// Profiles API base URL
    pub profiles_api: String,
    /// Relayer API base URL
    pub relayer_api: String,
    /// RTDS WebSocket URL
    pub rtds_wss: String,
    /// CLOB WebSocket URL
    pub clob_wss: String,
}

impl Default for Endpoints {
    fn default() -> Self {
        Self::new()
    }
}

impl Endpoints {
    /// Create endpoints with default values.
    #[must_use]
    pub fn new() -> Self {
        Self {
            gamma_api: GAMMA_API_BASE.to_string(),
            data_api: DATA_API_BASE.to_string(),
            clob_api: CLOB_API_BASE.to_string(),
            profiles_api: PROFILES_API_BASE.to_string(),
            relayer_api: RELAYER_API_BASE.to_string(),
            rtds_wss: RTDS_WSS_BASE.to_string(),
            clob_wss: CLOB_WSS_BASE.to_string(),
        }
    }

    /// Create endpoints from environment variables with defaults.
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            gamma_api: gamma_api_url(),
            data_api: data_api_url(),
            clob_api: clob_api_url(),
            profiles_api: profiles_api_url(),
            relayer_api: relayer_api_url(),
            rtds_wss: rtds_wss_url(),
            clob_wss: clob_wss_url(),
        }
    }

    /// Builder pattern - set Gamma API URL.
    #[must_use]
    pub fn with_gamma_api(mut self, url: impl Into<String>) -> Self {
        self.gamma_api = url.into();
        self
    }

    /// Builder pattern - set Data API URL.
    #[must_use]
    pub fn with_data_api(mut self, url: impl Into<String>) -> Self {
        self.data_api = url.into();
        self
    }

    /// Builder pattern - set CLOB API URL.
    #[must_use]
    pub fn with_clob_api(mut self, url: impl Into<String>) -> Self {
        self.clob_api = url.into();
        self
    }

    /// Builder pattern - set Profiles API URL.
    #[must_use]
    pub fn with_profiles_api(mut self, url: impl Into<String>) -> Self {
        self.profiles_api = url.into();
        self
    }

    /// Builder pattern - set Relayer API URL.
    #[must_use]
    pub fn with_relayer_api(mut self, url: impl Into<String>) -> Self {
        self.relayer_api = url.into();
        self
    }

    /// Builder pattern - set RTDS WebSocket URL.
    #[must_use]
    pub fn with_rtds_wss(mut self, url: impl Into<String>) -> Self {
        self.rtds_wss = url.into();
        self
    }

    /// Builder pattern - set CLOB WebSocket URL.
    #[must_use]
    pub fn with_clob_wss(mut self, url: impl Into<String>) -> Self {
        self.clob_wss = url.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_endpoints() {
        let endpoints = Endpoints::new();
        assert_eq!(endpoints.gamma_api, GAMMA_API_BASE);
        assert_eq!(endpoints.data_api, DATA_API_BASE);
        assert_eq!(endpoints.clob_api, CLOB_API_BASE);
        assert_eq!(endpoints.profiles_api, PROFILES_API_BASE);
        assert_eq!(endpoints.relayer_api, RELAYER_API_BASE);
        assert_eq!(endpoints.rtds_wss, RTDS_WSS_BASE);
        assert_eq!(endpoints.clob_wss, CLOB_WSS_BASE);
    }

    #[test]
    fn test_builder_pattern() {
        let endpoints = Endpoints::new()
            .with_gamma_api("https://custom-gamma.example.com")
            .with_data_api("https://custom-data.example.com");

        assert_eq!(endpoints.gamma_api, "https://custom-gamma.example.com");
        assert_eq!(endpoints.data_api, "https://custom-data.example.com");
        // Others should remain default
        assert_eq!(endpoints.clob_api, CLOB_API_BASE);
    }

    #[test]
    fn test_constants() {
        assert!(GAMMA_API_BASE.starts_with("https://"));
        assert!(DATA_API_BASE.starts_with("https://"));
        assert!(CLOB_API_BASE.starts_with("https://"));
        assert!(PROFILES_API_BASE.starts_with("https://"));
        assert!(RTDS_WSS_BASE.starts_with("wss://"));
        assert!(CLOB_WSS_BASE.starts_with("wss://"));
    }
}
