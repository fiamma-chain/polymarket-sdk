//! Error types for the Polymarket SDK.
//!
//! Provides structured error handling with retry and recovery support.

use std::time::Duration;
use thiserror::Error;

/// Main error type for the Polymarket SDK.
#[derive(Error, Debug)]
pub enum PolymarketError {
    /// Network-related errors (typically retryable)
    #[error("Network error: {message}")]
    Network {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// API errors from Polymarket
    #[error("API error ({status}): {message}")]
    Api {
        status: u16,
        message: String,
        error_code: Option<String>,
    },

    /// Authentication/authorization errors
    #[error("Auth error: {message}")]
    Auth {
        message: String,
        kind: AuthErrorKind,
    },

    /// Order-related errors
    #[error("Order error: {message}")]
    Order {
        message: String,
        kind: OrderErrorKind,
    },

    /// Market data errors
    #[error("Market data error: {message}")]
    MarketData {
        message: String,
        kind: MarketDataErrorKind,
    },

    /// WebSocket/streaming errors
    #[error("Stream error: {message}")]
    Stream {
        message: String,
        kind: StreamErrorKind,
    },

    /// Configuration errors
    #[error("Config error: {message}")]
    Config { message: String },

    /// Parsing/serialization errors
    #[error("Parse error: {message}")]
    Parse {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Timeout errors
    #[error("Timeout: operation timed out after {duration:?}")]
    Timeout {
        duration: Duration,
        operation: String,
    },

    /// Rate limiting errors
    #[error("Rate limit exceeded: {message}")]
    RateLimit {
        message: String,
        retry_after: Option<Duration>,
    },

    /// Validation errors
    #[error("Validation error: {message}")]
    Validation {
        message: String,
        field: Option<String>,
    },

    /// Internal errors (bugs)
    #[error("Internal error: {message}")]
    Internal {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

/// Authentication error subcategories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthErrorKind {
    /// Invalid API credentials
    InvalidCredentials,
    /// Expired credentials
    ExpiredCredentials,
    /// Insufficient permissions
    InsufficientPermissions,
    /// Signature error
    SignatureError,
    /// Nonce error
    NonceError,
}

/// Order error subcategories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderErrorKind {
    /// Invalid price
    InvalidPrice,
    /// Invalid size
    InvalidSize,
    /// Insufficient balance
    InsufficientBalance,
    /// Market closed
    MarketClosed,
    /// Duplicate order
    DuplicateOrder,
    /// Order not found
    OrderNotFound,
    /// Cancellation failed
    CancellationFailed,
    /// Execution failed
    ExecutionFailed,
    /// Size constraint violation
    SizeConstraint,
    /// Price constraint violation
    PriceConstraint,
}

/// Market data error subcategories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarketDataErrorKind {
    /// Token not found
    TokenNotFound,
    /// Market not found
    MarketNotFound,
    /// Stale data
    StaleData,
    /// Incomplete data
    IncompleteData,
    /// Book unavailable
    BookUnavailable,
}

/// Streaming error subcategories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamErrorKind {
    /// Failed to establish connection
    ConnectionFailed,
    /// Connection was lost
    ConnectionLost,
    /// Subscription request failed
    SubscriptionFailed,
    /// Received corrupted message
    MessageCorrupted,
    /// Currently reconnecting
    Reconnecting,
    /// Unknown error
    Unknown,
}

impl PolymarketError {
    /// Check if this error is retryable.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Network { .. } => true,
            Self::Api { status, .. } => *status >= 500 && *status < 600,
            Self::Timeout { .. } => true,
            Self::RateLimit { .. } => true,
            Self::Stream { kind, .. } => {
                matches!(
                    kind,
                    StreamErrorKind::ConnectionLost | StreamErrorKind::Reconnecting
                )
            }
            _ => false,
        }
    }

    /// Get suggested retry delay.
    #[must_use]
    pub fn retry_delay(&self) -> Option<Duration> {
        match self {
            Self::Network { .. } => Some(Duration::from_millis(100)),
            Self::Api { status, .. } if *status >= 500 => Some(Duration::from_millis(500)),
            Self::Timeout { .. } => Some(Duration::from_millis(50)),
            Self::RateLimit { retry_after, .. } => retry_after.or(Some(Duration::from_secs(1))),
            Self::Stream { .. } => Some(Duration::from_millis(250)),
            _ => None,
        }
    }

    /// Check if this error indicates the wallet is not registered with Polymarket CLOB.
    ///
    /// This happens when calling `/auth/derive-api-key` for a wallet that has never
    /// been registered via `/auth/api-key` (create API key).
    ///
    /// # Returns
    /// `true` if the error message indicates wallet not registered
    #[must_use]
    pub fn is_wallet_not_registered(&self) -> bool {
        match self {
            Self::Api {
                status, message, ..
            } => *status == 400 && message.contains("Could not derive api key"),
            _ => false,
        }
    }

    /// Check if this is a critical error that should stop trading.
    #[must_use]
    pub fn is_critical(&self) -> bool {
        match self {
            Self::Auth { .. } => true,
            Self::Config { .. } => true,
            Self::Internal { .. } => true,
            Self::Order { kind, .. } => matches!(kind, OrderErrorKind::InsufficientBalance),
            _ => false,
        }
    }

    /// Get error category for metrics.
    #[must_use]
    pub fn category(&self) -> &'static str {
        match self {
            Self::Network { .. } => "network",
            Self::Api { .. } => "api",
            Self::Auth { .. } => "auth",
            Self::Order { .. } => "order",
            Self::MarketData { .. } => "market_data",
            Self::Stream { .. } => "stream",
            Self::Config { .. } => "config",
            Self::Parse { .. } => "parse",
            Self::Timeout { .. } => "timeout",
            Self::RateLimit { .. } => "rate_limit",
            Self::Validation { .. } => "validation",
            Self::Internal { .. } => "internal",
        }
    }
}

// Convenience constructors
impl PolymarketError {
    /// Create a network error with source.
    pub fn network<E: std::error::Error + Send + Sync + 'static>(
        message: impl Into<String>,
        source: E,
    ) -> Self {
        Self::Network {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a network error without source.
    pub fn network_simple(message: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
            source: None,
        }
    }

    /// Create an API error.
    pub fn api(status: u16, message: impl Into<String>) -> Self {
        Self::Api {
            status,
            message: message.into(),
            error_code: None,
        }
    }

    /// Create an auth error.
    pub fn auth(message: impl Into<String>) -> Self {
        Self::Auth {
            message: message.into(),
            kind: AuthErrorKind::SignatureError,
        }
    }

    /// Create a crypto/signature error (alias for auth).
    pub fn crypto(message: impl Into<String>) -> Self {
        Self::Auth {
            message: message.into(),
            kind: AuthErrorKind::SignatureError,
        }
    }

    /// Create an order error.
    pub fn order(message: impl Into<String>, kind: OrderErrorKind) -> Self {
        Self::Order {
            message: message.into(),
            kind,
        }
    }

    /// Create a market data error.
    pub fn market_data(message: impl Into<String>, kind: MarketDataErrorKind) -> Self {
        Self::MarketData {
            message: message.into(),
            kind,
        }
    }

    /// Create a stream error.
    pub fn stream(message: impl Into<String>, kind: StreamErrorKind) -> Self {
        Self::Stream {
            message: message.into(),
            kind,
        }
    }

    /// Create a config error.
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Create a parse error.
    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse {
            message: message.into(),
            source: None,
        }
    }

    /// Create a parse error with source.
    pub fn parse_with_source<E: std::error::Error + Send + Sync + 'static>(
        message: impl Into<String>,
        source: E,
    ) -> Self {
        Self::Parse {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a timeout error.
    pub fn timeout(duration: Duration, operation: impl Into<String>) -> Self {
        Self::Timeout {
            duration,
            operation: operation.into(),
        }
    }

    /// Create a rate limit error.
    pub fn rate_limit(message: impl Into<String>) -> Self {
        Self::RateLimit {
            message: message.into(),
            retry_after: None,
        }
    }

    /// Create a validation error.
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
            field: None,
        }
    }

    /// Create an internal error.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            source: None,
        }
    }

    /// Create an internal error with source.
    pub fn internal_with_source<E: std::error::Error + Send + Sync + 'static>(
        message: impl Into<String>,
        source: E,
    ) -> Self {
        Self::Internal {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

// Implement From for common external error types
impl From<reqwest::Error> for PolymarketError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::Timeout {
                duration: Duration::from_secs(30),
                operation: "HTTP request".to_string(),
            }
        } else if err.is_connect() || err.is_request() {
            Self::network("HTTP request failed", err)
        } else {
            Self::internal_with_source("Unexpected reqwest error", err)
        }
    }
}

impl From<serde_json::Error> for PolymarketError {
    fn from(err: serde_json::Error) -> Self {
        Self::parse_with_source(format!("JSON parsing failed: {err}"), err)
    }
}

impl From<url::ParseError> for PolymarketError {
    fn from(err: url::ParseError) -> Self {
        Self::config(format!("Invalid URL: {err}"))
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for PolymarketError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        use tokio_tungstenite::tungstenite::Error as WsError;

        let kind = match &err {
            WsError::ConnectionClosed | WsError::AlreadyClosed => StreamErrorKind::ConnectionLost,
            WsError::Io(_) => StreamErrorKind::ConnectionFailed,
            WsError::Protocol(_) => StreamErrorKind::MessageCorrupted,
            _ => StreamErrorKind::ConnectionFailed,
        };

        Self::stream(format!("WebSocket error: {err}"), kind)
    }
}

// Manual Clone implementation since Box<dyn Error> doesn't implement Clone
impl Clone for PolymarketError {
    fn clone(&self) -> Self {
        match self {
            Self::Network { message, .. } => Self::Network {
                message: message.clone(),
                source: None,
            },
            Self::Api {
                status,
                message,
                error_code,
            } => Self::Api {
                status: *status,
                message: message.clone(),
                error_code: error_code.clone(),
            },
            Self::Auth { message, kind } => Self::Auth {
                message: message.clone(),
                kind: kind.clone(),
            },
            Self::Order { message, kind } => Self::Order {
                message: message.clone(),
                kind: kind.clone(),
            },
            Self::MarketData { message, kind } => Self::MarketData {
                message: message.clone(),
                kind: kind.clone(),
            },
            Self::Stream { message, kind } => Self::Stream {
                message: message.clone(),
                kind: kind.clone(),
            },
            Self::Config { message } => Self::Config {
                message: message.clone(),
            },
            Self::Parse { message, .. } => Self::Parse {
                message: message.clone(),
                source: None,
            },
            Self::Timeout {
                duration,
                operation,
            } => Self::Timeout {
                duration: *duration,
                operation: operation.clone(),
            },
            Self::RateLimit {
                message,
                retry_after,
            } => Self::RateLimit {
                message: message.clone(),
                retry_after: *retry_after,
            },
            Self::Validation { message, field } => Self::Validation {
                message: message.clone(),
                field: field.clone(),
            },
            Self::Internal { message, .. } => Self::Internal {
                message: message.clone(),
                source: None,
            },
        }
    }
}

/// Result type alias for convenience.
pub type Result<T> = std::result::Result<T, PolymarketError>;

/// Alias for backward compatibility.
pub type Error = PolymarketError;
