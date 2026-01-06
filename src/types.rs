//! Common types for Polymarket SDK
//!
//! This module contains shared types used across different API clients,
//! including trading types, market data structures, and authentication types.

#[cfg(feature = "auth")]
use alloy_primitives::U256;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::str::FromStr;

// ============================================================================
// Trading Types
// ============================================================================

/// Trading side for orders
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    /// Get string representation
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Buy => "BUY",
            Self::Sell => "SELL",
        }
    }

    /// Get opposite side
    #[must_use]
    pub fn opposite(&self) -> Self {
        match self {
            Self::Buy => Self::Sell,
            Self::Sell => Self::Buy,
        }
    }
}

/// Order book level (price/size pair)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookLevel {
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub size: Decimal,
}

// ============================================================================
// Authentication Types
// ============================================================================

/// API credentials for Polymarket authentication
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiCredentials {
    /// API key
    #[serde(rename = "apiKey")]
    pub api_key: String,
    /// API secret (base64 encoded)
    pub secret: String,
    /// API passphrase
    pub passphrase: String,
}

impl ApiCredentials {
    /// Create new API credentials
    #[must_use]
    pub fn new(
        api_key: impl Into<String>,
        secret: impl Into<String>,
        passphrase: impl Into<String>,
    ) -> Self {
        Self {
            api_key: api_key.into(),
            secret: secret.into(),
            passphrase: passphrase.into(),
        }
    }

    /// Check if credentials are configured
    #[must_use]
    pub fn is_configured(&self) -> bool {
        !self.api_key.is_empty() && !self.secret.is_empty() && !self.passphrase.is_empty()
    }
}

// ============================================================================
// Order Types
// ============================================================================

/// Configuration options for order creation
#[derive(Debug, Clone, Default)]
pub struct OrderOptions {
    /// Tick size for price rounding
    pub tick_size: Option<Decimal>,
    /// Whether to use negative risk contracts
    pub neg_risk: Option<bool>,
    /// Fee rate in basis points
    pub fee_rate_bps: Option<u32>,
}

impl OrderOptions {
    /// Create new order options
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set tick size
    #[must_use]
    pub fn with_tick_size(mut self, tick_size: Decimal) -> Self {
        self.tick_size = Some(tick_size);
        self
    }

    /// Set negative risk flag
    #[must_use]
    pub fn with_neg_risk(mut self, neg_risk: bool) -> Self {
        self.neg_risk = Some(neg_risk);
        self
    }

    /// Set fee rate in basis points
    #[must_use]
    pub fn with_fee_rate_bps(mut self, fee_rate_bps: u32) -> Self {
        self.fee_rate_bps = Some(fee_rate_bps);
        self
    }
}

/// Extra arguments for order creation
#[cfg(feature = "auth")]
#[derive(Debug, Clone)]
pub struct ExtraOrderArgs {
    /// Fee rate in basis points
    pub fee_rate_bps: u32,
    /// Nonce for replay protection
    pub nonce: U256,
    /// Taker address (usually zero address)
    pub taker: String,
}

#[cfg(feature = "auth")]
impl Default for ExtraOrderArgs {
    fn default() -> Self {
        Self {
            fee_rate_bps: 0,
            nonce: U256::ZERO,
            taker: "0x0000000000000000000000000000000000000000".to_string(),
        }
    }
}

#[cfg(feature = "auth")]
impl ExtraOrderArgs {
    /// Create new extra order args
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set fee rate in basis points
    #[must_use]
    pub fn with_fee_rate_bps(mut self, fee_rate_bps: u32) -> Self {
        self.fee_rate_bps = fee_rate_bps;
        self
    }

    /// Set nonce
    #[must_use]
    pub fn with_nonce(mut self, nonce: U256) -> Self {
        self.nonce = nonce;
        self
    }

    /// Set taker address
    #[must_use]
    pub fn with_taker(mut self, taker: impl Into<String>) -> Self {
        self.taker = taker.into();
        self
    }
}

/// Market order arguments
#[derive(Debug, Clone)]
pub struct MarketOrderArgs {
    /// Token ID (condition token)
    pub token_id: String,
    /// Amount to trade
    pub amount: Decimal,
}

impl MarketOrderArgs {
    /// Create new market order args
    #[must_use]
    pub fn new(token_id: impl Into<String>, amount: Decimal) -> Self {
        Self {
            token_id: token_id.into(),
            amount,
        }
    }
}

/// Signed order request ready for submission
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignedOrderRequest {
    /// Random salt for uniqueness
    pub salt: u64,
    /// Maker/funder address
    pub maker: String,
    /// Signer address
    pub signer: String,
    /// Taker address (usually zero)
    pub taker: String,
    /// Token ID
    pub token_id: String,
    /// Maker amount in token units
    pub maker_amount: String,
    /// Taker amount in token units
    pub taker_amount: String,
    /// Expiration timestamp
    pub expiration: String,
    /// Nonce for replay protection
    pub nonce: String,
    /// Fee rate in basis points
    pub fee_rate_bps: String,
    /// Order side (BUY/SELL)
    pub side: String,
    /// Signature type
    pub signature_type: u8,
    /// EIP-712 signature
    pub signature: String,
}

/// Order type for CLOB orders
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderType {
    /// Good Till Cancelled (default for limit orders)
    GTC,
    /// Fill Or Kill (for market orders)
    FOK,
    /// Good Till Date
    GTD,
    /// Fill And Kill
    FAK,
}

impl Default for OrderType {
    fn default() -> Self {
        OrderType::GTC
    }
}

/// NewOrder is the payload structure for posting orders to the Polymarket API
/// It wraps order data with orderType, owner, and deferExec fields
/// IMPORTANT: Field order MUST match TypeScript SDK for HMAC signature compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewOrder {
    /// Whether to defer execution (MUST be first field for JSON field order)
    #[serde(default)]
    pub defer_exec: bool,
    /// The order data
    pub order: NewOrderData,
    /// Owner - should be the API key, NOT the wallet address
    pub owner: String,
    /// Order type (GTC, FOK, etc.)
    pub order_type: OrderType,
}

/// NewOrderData contains the actual order fields
/// Note: salt must be a number (i64) in JSON, not a string
/// IMPORTANT: Field order MUST match TypeScript SDK for HMAC signature compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewOrderData {
    /// Random salt for uniqueness - MUST be a number in JSON
    pub salt: i64,
    /// Maker/funder address
    pub maker: String,
    /// Signer address
    pub signer: String,
    /// Taker address (usually zero)
    pub taker: String,
    /// Token ID
    pub token_id: String,
    /// Maker amount in token units
    pub maker_amount: String,
    /// Taker amount in token units
    pub taker_amount: String,
    /// Order side (BUY/SELL) - MUST come after takerAmount, before expiration
    pub side: String,
    /// Expiration timestamp
    pub expiration: String,
    /// Nonce for replay protection
    pub nonce: String,
    /// Fee rate in basis points
    pub fee_rate_bps: String,
    /// Signature type (0=EOA, 1=PolyProxy, 2=PolyGnosisSafe)
    pub signature_type: u8,
    /// EIP-712 signature
    pub signature: String,
}

impl NewOrder {
    /// Convert SignedOrderRequest to NewOrder format for API submission
    /// Field initialization order matches struct field order for consistency
    pub fn from_signed_order(
        order: &SignedOrderRequest,
        api_key: &str,
        order_type: OrderType,
        defer_exec: bool,
    ) -> Self {
        NewOrder {
            defer_exec,
            order: NewOrderData {
                // Salt must be i64 for JSON serialization as number
                salt: order.salt as i64,
                maker: order.maker.clone(),
                signer: order.signer.clone(),
                taker: order.taker.clone(),
                token_id: order.token_id.clone(),
                maker_amount: order.maker_amount.clone(),
                taker_amount: order.taker_amount.clone(),
                side: order.side.clone(),
                expiration: order.expiration.clone(),
                nonce: order.nonce.clone(),
                fee_rate_bps: order.fee_rate_bps.clone(),
                signature_type: order.signature_type,
                signature: order.signature.clone(),
            },
            owner: api_key.to_string(),
            order_type,
        }
    }
}

// ============================================================================
// Market Types
// ============================================================================

/// Market token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// Token ID (condition token)
    pub token_id: String,
    /// Outcome name (e.g., "Yes", "No")
    pub outcome: String,
    /// Current price if available
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "deserialize_decimal_opt")]
    pub price: Option<Decimal>,
}

/// Market information from Gamma API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Market {
    /// Condition ID (market identifier)
    pub condition_id: String,
    /// Market slug for URL
    pub slug: String,
    /// Market question/title
    #[serde(default)]
    pub question: Option<String>,
    /// Market description
    #[serde(default)]
    pub description: Option<String>,
    /// Category/tag
    #[serde(default)]
    pub category: Option<String>,
    /// Whether market is active
    pub active: bool,
    /// Whether market is closed
    pub closed: bool,
    /// Market end date
    #[serde(default)]
    pub end_date: Option<String>,
    /// Market icon URL
    #[serde(default)]
    pub icon: Option<String>,
    /// CLOB token IDs (JSON string array)
    #[serde(default)]
    pub clob_token_ids: Option<String>,
    /// Outcomes (JSON string array)
    #[serde(default)]
    pub outcomes: Option<String>,
    /// Outcome prices (JSON string array, e.g. "[\"0.95\", \"0.05\"]")
    #[serde(default)]
    pub outcome_prices: Option<String>,
    /// Liquidity
    #[serde(default, deserialize_with = "deserialize_decimal_opt")]
    pub liquidity_num: Option<Decimal>,
    /// 24-hour volume
    #[serde(
        default,
        rename = "volume24hr",
        deserialize_with = "deserialize_decimal_opt"
    )]
    pub volume_24hr: Option<Decimal>,
    /// Total volume
    #[serde(default, deserialize_with = "deserialize_decimal_opt")]
    pub volume_num: Option<Decimal>,
    /// Minimum order size
    #[serde(default, deserialize_with = "deserialize_decimal_opt")]
    pub order_min_size: Option<Decimal>,
    /// Price tick size
    #[serde(
        default,
        rename = "orderPriceMinTickSize",
        deserialize_with = "deserialize_decimal_opt"
    )]
    pub order_tick_size: Option<Decimal>,
}

impl Market {
    /// Parse CLOB token IDs from JSON string
    #[must_use]
    pub fn parse_token_ids(&self) -> Vec<String> {
        self.clob_token_ids
            .as_ref()
            .and_then(|raw| serde_json::from_str(raw).ok())
            .unwrap_or_default()
    }

    /// Parse outcomes from JSON string
    #[must_use]
    pub fn parse_outcomes(&self) -> Vec<String> {
        self.outcomes
            .as_ref()
            .and_then(|raw| serde_json::from_str(raw).ok())
            .unwrap_or_else(|| vec!["Yes".to_string(), "No".to_string()])
    }

    /// Parse outcome prices from JSON string
    /// Returns (yes_price, no_price) as `Option<f64>` values
    #[must_use]
    pub fn parse_outcome_prices(&self) -> (Option<f64>, Option<f64>) {
        let prices: Vec<String> = self
            .outcome_prices
            .as_ref()
            .and_then(|raw| serde_json::from_str(raw).ok())
            .unwrap_or_default();

        let yes_price = prices.first().and_then(|s| s.parse::<f64>().ok());
        let no_price = prices.get(1).and_then(|s| s.parse::<f64>().ok());

        (yes_price, no_price)
    }
}

/// Deserialize Option<Decimal> from string/number/null.
fn deserialize_decimal_opt<'de, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
where
    D: Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::Null => Ok(None),
        Value::String(s) => {
            if s.is_empty() {
                Ok(None)
            } else {
                Decimal::from_str(&s).map(Some).map_err(de::Error::custom)
            }
        }
        Value::Number(n) => Decimal::from_str(&n.to_string())
            .map(Some)
            .map_err(de::Error::custom),
        other => Err(de::Error::custom(format!("expected decimal, got {other}"))),
    }
}

/// Event metadata from Gamma API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    /// Event ID
    pub id: String,
    /// Event ticker
    #[serde(default)]
    pub ticker: Option<String>,
    /// Event slug
    pub slug: String,
    /// Event title
    #[serde(default)]
    pub title: Option<String>,
    /// Event description
    #[serde(default)]
    pub description: Option<String>,
    /// Resolution source URL or description
    #[serde(default)]
    pub resolution_source: Option<String>,
    /// Start date (ISO format)
    #[serde(default)]
    pub start_date: Option<String>,
    /// Creation date (ISO format)
    #[serde(default)]
    pub creation_date: Option<String>,
    /// End date (ISO format)
    #[serde(default)]
    pub end_date: Option<String>,
    /// Event image URL
    #[serde(default)]
    pub image: Option<String>,
    /// Event icon URL
    #[serde(default)]
    pub icon: Option<String>,
    /// Whether event is active
    #[serde(default)]
    pub active: Option<bool>,
    /// Whether event is closed
    #[serde(default)]
    pub closed: Option<bool>,
    /// Whether event is archived
    #[serde(default)]
    pub archived: Option<bool>,
    /// Whether event is new
    #[serde(default)]
    pub new: Option<bool>,
    /// Whether event is featured
    #[serde(default)]
    pub featured: Option<bool>,
    /// Whether event is restricted
    #[serde(default)]
    pub restricted: Option<bool>,
    /// Total liquidity
    #[serde(default)]
    pub liquidity: Option<f64>,
    /// Total volume
    #[serde(default)]
    pub volume: Option<f64>,
    /// Open interest
    #[serde(default)]
    pub open_interest: Option<f64>,
    /// Sort by field
    #[serde(default)]
    pub sort_by: Option<String>,
    /// Created at timestamp
    #[serde(default)]
    pub created_at: Option<String>,
    /// Updated at timestamp
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Competitive score
    #[serde(default)]
    pub competitive: Option<f64>,
    /// 24-hour volume
    #[serde(default)]
    pub volume24hr: Option<f64>,
    /// 1-week volume
    #[serde(default)]
    pub volume1wk: Option<f64>,
    /// 1-month volume
    #[serde(default)]
    pub volume1mo: Option<f64>,
    /// 1-year volume
    #[serde(default)]
    pub volume1yr: Option<f64>,
    /// Whether order book is enabled
    #[serde(default)]
    pub enable_order_book: Option<bool>,
    /// CLOB liquidity
    #[serde(default)]
    pub liquidity_clob: Option<f64>,
    /// Whether negative risk is enabled
    #[serde(default)]
    pub neg_risk: Option<bool>,
    /// Negative risk market ID
    #[serde(default)]
    pub neg_risk_market_id: Option<String>,
    /// Comment count
    #[serde(default)]
    pub comment_count: Option<i32>,
    /// Associated markets
    #[serde(default)]
    pub markets: Vec<EventMarket>,
    /// Associated tags
    #[serde(default)]
    pub tags: Vec<EventTag>,
    /// Whether this is a CYOM (Create Your Own Market)
    #[serde(default)]
    pub cyom: Option<bool>,
    /// Whether to show all outcomes
    #[serde(default)]
    pub show_all_outcomes: Option<bool>,
    /// Whether to show market images
    #[serde(default)]
    pub show_market_images: Option<bool>,
    /// Whether negative risk is enabled for this event
    #[serde(default)]
    pub enable_neg_risk: Option<bool>,
    /// Whether automatically active
    #[serde(default)]
    pub automatically_active: Option<bool>,
    /// GMP chart mode
    #[serde(default)]
    pub gmp_chart_mode: Option<String>,
    /// Whether negative risk is augmented
    #[serde(default)]
    pub neg_risk_augmented: Option<bool>,
    /// Whether markets are cumulative
    #[serde(default)]
    pub cumulative_markets: Option<bool>,
    /// Whether pending deployment
    #[serde(default)]
    pub pending_deployment: Option<bool>,
    /// Whether currently deploying
    #[serde(default)]
    pub deploying: Option<bool>,
    /// Deploying timestamp
    #[serde(default)]
    pub deploying_timestamp: Option<String>,
    /// Whether requires translation
    #[serde(default)]
    pub requires_translation: Option<bool>,
}

/// Tag associated with an event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventTag {
    /// Tag ID
    #[serde(default)]
    pub id: Option<String>,
    /// Tag label (display name)
    #[serde(default)]
    pub label: Option<String>,
    /// Tag slug
    #[serde(default)]
    pub slug: Option<String>,
    /// Whether to force show
    #[serde(default)]
    pub force_show: Option<bool>,
    /// Whether to force hide
    #[serde(default)]
    pub force_hide: Option<bool>,
    /// Published at timestamp
    #[serde(default)]
    pub published_at: Option<String>,
    /// Updated by user ID
    #[serde(default)]
    pub updated_by: Option<i32>,
    /// Created at timestamp
    #[serde(default)]
    pub created_at: Option<String>,
    /// Updated at timestamp
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Whether requires translation
    #[serde(default)]
    pub requires_translation: Option<bool>,
}

/// Market info within an event (full details from events endpoint)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventMarket {
    /// Market ID
    #[serde(default)]
    pub id: Option<String>,
    /// Market question
    #[serde(default)]
    pub question: Option<String>,
    /// Condition ID
    pub condition_id: String,
    /// Market slug
    #[serde(default)]
    pub slug: Option<String>,
    /// End date (ISO format)
    #[serde(default)]
    pub end_date: Option<String>,
    /// Start date (ISO format)
    #[serde(default)]
    pub start_date: Option<String>,
    /// Liquidity (string format from API)
    #[serde(default)]
    pub liquidity: Option<String>,
    /// Market image URL
    #[serde(default)]
    pub image: Option<String>,
    /// Market icon URL
    #[serde(default)]
    pub icon: Option<String>,
    /// Market description
    #[serde(default)]
    pub description: Option<String>,
    /// Outcomes JSON string (e.g., "[\"Yes\", \"No\"]")
    #[serde(default)]
    pub outcomes: Option<String>,
    /// Outcome prices JSON string (e.g., "[\"0.5\", \"0.5\"]")
    #[serde(default)]
    pub outcome_prices: Option<String>,
    /// Volume (string format from API)
    #[serde(default)]
    pub volume: Option<String>,
    /// Whether market is active
    #[serde(default)]
    pub active: Option<bool>,
    /// Whether market is closed
    #[serde(default)]
    pub closed: Option<bool>,
    /// Market maker address
    #[serde(default)]
    pub market_maker_address: Option<String>,
    /// Created at timestamp
    #[serde(default)]
    pub created_at: Option<String>,
    /// Updated at timestamp
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Whether market is new
    #[serde(default)]
    pub new: Option<bool>,
    /// Whether market is featured
    #[serde(default)]
    pub featured: Option<bool>,
    /// Submitted by address
    #[serde(default)]
    pub submitted_by: Option<String>,
    /// Whether market is archived
    #[serde(default)]
    pub archived: Option<bool>,
    /// Resolved by address
    #[serde(default)]
    pub resolved_by: Option<String>,
    /// Whether market is restricted
    #[serde(default)]
    pub restricted: Option<bool>,
    /// Group item title (for multi-outcome events)
    #[serde(default)]
    pub group_item_title: Option<String>,
    /// Group item threshold
    #[serde(default)]
    pub group_item_threshold: Option<String>,
    /// Question ID (for negative risk markets)
    #[serde(default, rename = "questionID")]
    pub question_id: Option<String>,
    /// Whether order book is enabled
    #[serde(default)]
    pub enable_order_book: Option<bool>,
    /// Order price minimum tick size
    #[serde(default)]
    pub order_price_min_tick_size: Option<f64>,
    /// Order minimum size
    #[serde(default)]
    pub order_min_size: Option<f64>,
    /// Volume as number
    #[serde(default)]
    pub volume_num: Option<f64>,
    /// Liquidity as number
    #[serde(default)]
    pub liquidity_num: Option<f64>,
    /// End date ISO format (YYYY-MM-DD)
    #[serde(default)]
    pub end_date_iso: Option<String>,
    /// Start date ISO format (YYYY-MM-DD)
    #[serde(default)]
    pub start_date_iso: Option<String>,
    /// Whether dates have been reviewed
    #[serde(default)]
    pub has_reviewed_dates: Option<bool>,
    /// 24-hour volume
    #[serde(default)]
    pub volume24hr: Option<f64>,
    /// 1-week volume
    #[serde(default)]
    pub volume1wk: Option<f64>,
    /// 1-month volume
    #[serde(default)]
    pub volume1mo: Option<f64>,
    /// 1-year volume
    #[serde(default)]
    pub volume1yr: Option<f64>,
    /// CLOB token IDs JSON string
    #[serde(default)]
    pub clob_token_ids: Option<String>,
    /// UMA bond amount
    #[serde(default)]
    pub uma_bond: Option<String>,
    /// UMA reward amount
    #[serde(default)]
    pub uma_reward: Option<String>,
    /// 24-hour CLOB volume
    #[serde(default)]
    pub volume24hr_clob: Option<f64>,
    /// 1-week CLOB volume
    #[serde(default)]
    pub volume1wk_clob: Option<f64>,
    /// 1-month CLOB volume
    #[serde(default)]
    pub volume1mo_clob: Option<f64>,
    /// 1-year CLOB volume
    #[serde(default)]
    pub volume1yr_clob: Option<f64>,
    /// CLOB volume
    #[serde(default)]
    pub volume_clob: Option<f64>,
    /// CLOB liquidity
    #[serde(default)]
    pub liquidity_clob: Option<f64>,
    /// Whether accepting orders
    #[serde(default)]
    pub accepting_orders: Option<bool>,
    /// Whether negative risk is enabled
    #[serde(default)]
    pub neg_risk: Option<bool>,
    /// Negative risk market ID
    #[serde(default, rename = "negRiskMarketID")]
    pub neg_risk_market_id: Option<String>,
    /// Negative risk request ID
    #[serde(default, rename = "negRiskRequestID")]
    pub neg_risk_request_id: Option<String>,
    /// Whether market is ready
    #[serde(default)]
    pub ready: Option<bool>,
    /// Whether market is funded
    #[serde(default)]
    pub funded: Option<bool>,
    /// Accepting orders timestamp
    #[serde(default)]
    pub accepting_orders_timestamp: Option<String>,
    /// Whether this is a CYOM market
    #[serde(default)]
    pub cyom: Option<bool>,
    /// Competitive score
    #[serde(default)]
    pub competitive: Option<f64>,
    /// Whether PagerDuty notification is enabled
    #[serde(default)]
    pub pager_duty_notification_enabled: Option<bool>,
    /// Whether market is approved
    #[serde(default)]
    pub approved: Option<bool>,
    /// Rewards minimum size
    #[serde(default)]
    pub rewards_min_size: Option<f64>,
    /// Rewards maximum spread
    #[serde(default)]
    pub rewards_max_spread: Option<f64>,
    /// Current spread
    #[serde(default)]
    pub spread: Option<f64>,
    /// One day price change
    #[serde(default)]
    pub one_day_price_change: Option<f64>,
    /// One hour price change
    #[serde(default)]
    pub one_hour_price_change: Option<f64>,
    /// One week price change
    #[serde(default)]
    pub one_week_price_change: Option<f64>,
    /// One month price change
    #[serde(default)]
    pub one_month_price_change: Option<f64>,
    /// One year price change
    #[serde(default)]
    pub one_year_price_change: Option<f64>,
    /// Last trade price
    #[serde(default)]
    pub last_trade_price: Option<f64>,
    /// Best bid price
    #[serde(default)]
    pub best_bid: Option<f64>,
    /// Best ask price
    #[serde(default)]
    pub best_ask: Option<f64>,
    /// Whether automatically active
    #[serde(default)]
    pub automatically_active: Option<bool>,
    /// Whether to clear book on start
    #[serde(default)]
    pub clear_book_on_start: Option<bool>,
    /// Whether to show GMP series
    #[serde(default)]
    pub show_gmp_series: Option<bool>,
    /// Whether to show GMP outcome
    #[serde(default)]
    pub show_gmp_outcome: Option<bool>,
    /// Whether manual activation is required
    #[serde(default)]
    pub manual_activation: Option<bool>,
    /// Whether this is the "other" option in negative risk
    #[serde(default)]
    pub neg_risk_other: Option<bool>,
    /// UMA resolution statuses JSON string
    #[serde(default)]
    pub uma_resolution_statuses: Option<String>,
    /// Whether pending deployment
    #[serde(default)]
    pub pending_deployment: Option<bool>,
    /// Whether currently deploying
    #[serde(default)]
    pub deploying: Option<bool>,
    /// Deploying timestamp
    #[serde(default)]
    pub deploying_timestamp: Option<String>,
    /// Whether RFQ is enabled
    #[serde(default)]
    pub rfq_enabled: Option<bool>,
    /// Whether holding rewards are enabled
    #[serde(default)]
    pub holding_rewards_enabled: Option<bool>,
    /// Whether fees are enabled
    #[serde(default)]
    pub fees_enabled: Option<bool>,
    /// Whether requires translation
    #[serde(default)]
    pub requires_translation: Option<bool>,
}

impl EventMarket {
    /// Parse outcome prices from JSON string to vector of f64
    ///
    /// Returns a tuple of (yes_price, no_price) for binary markets
    #[must_use]
    pub fn parse_outcome_prices(&self) -> (Option<f64>, Option<f64>) {
        let Some(prices_str) = &self.outcome_prices else {
            return (None, None);
        };

        let prices: Vec<String> = serde_json::from_str(prices_str).unwrap_or_default();
        let yes_price = prices.first().and_then(|s| s.parse::<f64>().ok());
        let no_price = prices.get(1).and_then(|s| s.parse::<f64>().ok());
        (yes_price, no_price)
    }

    /// Parse outcomes from JSON string to vector of strings
    #[must_use]
    pub fn parse_outcomes(&self) -> Vec<String> {
        self.outcomes
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default()
    }

    /// Parse CLOB token IDs from JSON string to vector of strings
    #[must_use]
    pub fn parse_clob_token_ids(&self) -> Vec<String> {
        self.clob_token_ids
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default()
    }

    /// Get the Yes token ID (first token in CLOB token IDs)
    #[must_use]
    pub fn yes_token_id(&self) -> Option<String> {
        self.parse_clob_token_ids().first().cloned()
    }

    /// Get the No token ID (second token in CLOB token IDs)
    #[must_use]
    pub fn no_token_id(&self) -> Option<String> {
        self.parse_clob_token_ids().get(1).cloned()
    }

    /// Check if this market is tradeable (active, accepting orders, not closed)
    #[must_use]
    pub fn is_tradeable(&self) -> bool {
        self.active.unwrap_or(false)
            && self.accepting_orders.unwrap_or(false)
            && !self.closed.unwrap_or(false)
    }
}

impl Event {
    /// Get the display name for this event (title or slug as fallback)
    #[must_use]
    pub fn display_name(&self) -> &str {
        self.title.as_deref().unwrap_or(&self.slug)
    }

    /// Get active markets only
    #[must_use]
    pub fn active_markets(&self) -> Vec<&EventMarket> {
        self.markets
            .iter()
            .filter(|m| m.active.unwrap_or(false) && !m.closed.unwrap_or(false))
            .collect()
    }

    /// Get tradeable markets only
    #[must_use]
    pub fn tradeable_markets(&self) -> Vec<&EventMarket> {
        self.markets.iter().filter(|m| m.is_tradeable()).collect()
    }

    /// Check if this event is active and has tradeable markets
    #[must_use]
    pub fn is_tradeable(&self) -> bool {
        self.active.unwrap_or(false)
            && !self.closed.unwrap_or(false)
            && self.markets.iter().any(|m| m.is_tradeable())
    }

    /// Get tag slugs for this event
    #[must_use]
    pub fn tag_slugs(&self) -> Vec<&str> {
        self.tags
            .iter()
            .filter_map(|t| t.slug.as_deref())
            .collect()
    }

    /// Check if this event has a specific tag
    #[must_use]
    pub fn has_tag(&self, tag_slug: &str) -> bool {
        self.tags
            .iter()
            .any(|t| t.slug.as_deref() == Some(tag_slug))
    }
}

/// Tag metadata from Gamma API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    /// Tag ID
    #[serde(default)]
    pub id: Option<String>,
    /// Tag slug
    #[serde(default)]
    pub slug: Option<String>,
    /// Tag label (display name)
    #[serde(default)]
    pub label: Option<String>,
    /// Tag name (legacy field, use label instead)
    #[serde(default)]
    pub name: Option<String>,
    /// Tag description
    #[serde(default)]
    pub description: Option<String>,
    /// Whether to force show
    #[serde(default)]
    pub force_show: Option<bool>,
    /// Whether to force hide
    #[serde(default)]
    pub force_hide: Option<bool>,
    /// Published at timestamp
    #[serde(default)]
    pub published_at: Option<String>,
    /// Updated by user ID
    #[serde(default)]
    pub updated_by: Option<i32>,
    /// Created at timestamp
    #[serde(default)]
    pub created_at: Option<String>,
    /// Updated at timestamp
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Whether requires translation
    #[serde(default)]
    pub requires_translation: Option<bool>,
}

// ============================================================================
// Profile Types
// ============================================================================

/// Trader profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraderProfile {
    /// Wallet address
    pub address: String,
    /// Display name
    #[serde(default)]
    pub name: Option<String>,
    /// Username/pseudonym
    #[serde(default)]
    pub username: Option<String>,
    /// Profile image URL
    #[serde(default, rename = "profileImage")]
    pub profile_image: Option<String>,
    /// Bio/description
    #[serde(default)]
    pub bio: Option<String>,
    /// Total volume traded
    #[serde(default)]
    pub volume: Option<Decimal>,
    /// Number of markets traded
    #[serde(default, rename = "marketsTraded")]
    pub markets_traded: Option<i32>,
    /// Profit and loss
    #[serde(default)]
    pub pnl: Option<Decimal>,
}

/// Leaderboard entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    /// Rank position
    pub rank: i32,
    /// Wallet address
    pub address: String,
    /// Display name
    #[serde(default)]
    pub name: Option<String>,
    /// Username
    #[serde(default)]
    pub username: Option<String>,
    /// Profile image
    #[serde(default, rename = "profileImage")]
    pub profile_image: Option<String>,
    /// Volume for the period
    pub volume: Decimal,
    /// Profit for the period
    #[serde(default)]
    pub profit: Option<Decimal>,
}

// ============================================================================
// Query Parameter Types
// ============================================================================

/// Pagination parameters
#[derive(Debug, Clone, Default)]
pub struct PaginationParams {
    /// Maximum number of results
    pub limit: Option<u32>,
    /// Offset for pagination
    pub offset: Option<u32>,
    /// Cursor for cursor-based pagination
    pub cursor: Option<String>,
}

impl PaginationParams {
    /// Create new pagination params
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set limit
    #[must_use]
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set offset
    #[must_use]
    pub fn with_offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Set cursor
    #[must_use]
    pub fn with_cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }
}

/// Common query parameters for listing endpoints
#[derive(Debug, Clone, Default)]
pub struct ListParams {
    /// Pagination
    pub pagination: PaginationParams,
    /// Filter by closed status
    pub closed: Option<bool>,
    /// Filter by active status
    pub active: Option<bool>,
    /// Sort field
    pub order: Option<String>,
    /// Sort ascending
    pub ascending: Option<bool>,
}

impl ListParams {
    /// Create new list params
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set limit
    #[must_use]
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.pagination.limit = Some(limit);
        self
    }

    /// Set offset
    #[must_use]
    pub fn with_offset(mut self, offset: u32) -> Self {
        self.pagination.offset = Some(offset);
        self
    }

    /// Filter by closed status
    #[must_use]
    pub fn with_closed(mut self, closed: bool) -> Self {
        self.closed = Some(closed);
        self
    }

    /// Filter by active status
    #[must_use]
    pub fn with_active(mut self, active: bool) -> Self {
        self.active = Some(active);
        self
    }

    /// Set sort order
    #[must_use]
    pub fn with_order(mut self, field: impl Into<String>, ascending: bool) -> Self {
        self.order = Some(field.into());
        self.ascending = Some(ascending);
        self
    }
}

// ============================================================================
// Connection Statistics
// ============================================================================

/// WebSocket connection statistics
#[derive(Debug, Clone, Default)]
pub struct ConnectionStats {
    /// Number of messages received
    pub messages_received: u64,
    /// Number of reconnection attempts
    pub reconnect_attempts: u32,
    /// Last message timestamp
    pub last_message_at: Option<DateTime<Utc>>,
    /// Connection established timestamp
    pub connected_at: Option<DateTime<Utc>>,
}

impl ConnectionStats {
    /// Create new stats
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a message received
    pub fn record_message(&mut self) {
        self.messages_received += 1;
        self.last_message_at = Some(Utc::now());
    }

    /// Record a reconnection attempt
    pub fn record_reconnect(&mut self) {
        self.reconnect_attempts += 1;
    }

    /// Record connection established
    pub fn record_connected(&mut self) {
        self.connected_at = Some(Utc::now());
        self.reconnect_attempts = 0;
    }
}

// ============================================================================
// Data API Types (for data-api.polymarket.com)
// ============================================================================

/// Polymarket trader profile from Data API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataApiTrader {
    /// Wallet address
    pub address: String,
    /// Display name
    #[serde(rename = "displayName", default)]
    pub display_name: Option<String>,
    /// Profile image URL
    #[serde(rename = "profileImage", default)]
    pub profile_image: Option<String>,
    /// Total PnL (as string)
    #[serde(rename = "totalPnl", default)]
    pub total_pnl: Option<String>,
    /// Total volume (as string)
    #[serde(rename = "totalVolume", default)]
    pub total_volume: Option<String>,
    /// Number of markets traded
    #[serde(rename = "marketsTraded", default)]
    pub markets_traded: Option<i32>,
    /// Win rate (0.0-1.0)
    #[serde(rename = "winRate", default)]
    pub win_rate: Option<f64>,
}

/// Polymarket position from Data API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataApiPosition {
    /// User proxy wallet address
    #[serde(rename = "proxyWallet")]
    pub proxy_wallet: String,
    /// Asset token ID
    pub asset: String,
    /// Market condition ID
    #[serde(rename = "conditionId")]
    pub condition_id: String,
    /// Position size (number of tokens)
    pub size: f64,
    /// Average entry price
    #[serde(rename = "avgPrice")]
    pub avg_price: f64,
    /// Initial value of position
    #[serde(rename = "initialValue")]
    pub initial_value: f64,
    /// Current value of position
    #[serde(rename = "currentValue")]
    pub current_value: f64,
    /// Cash PnL
    #[serde(rename = "cashPnl")]
    pub cash_pnl: f64,
    /// Percentage PnL
    #[serde(rename = "percentPnl")]
    pub percent_pnl: f64,
    /// Total amount bought
    #[serde(rename = "totalBought")]
    pub total_bought: f64,
    /// Realized PnL
    #[serde(rename = "realizedPnl")]
    pub realized_pnl: f64,
    /// Percentage realized PnL
    #[serde(rename = "percentRealizedPnl")]
    pub percent_realized_pnl: f64,
    /// Current market price
    #[serde(rename = "curPrice")]
    pub cur_price: f64,
    /// Whether position is redeemable
    pub redeemable: bool,
    /// Whether position is mergeable
    pub mergeable: bool,
    /// Market title
    pub title: String,
    /// Market slug
    pub slug: String,
    /// Market icon URL
    #[serde(default)]
    pub icon: Option<String>,
    /// Event slug
    #[serde(rename = "eventSlug")]
    pub event_slug: String,
    /// Outcome name (Yes/No)
    pub outcome: String,
    /// Outcome index
    #[serde(rename = "outcomeIndex")]
    pub outcome_index: i32,
    /// Opposite outcome name
    #[serde(rename = "oppositeOutcome")]
    pub opposite_outcome: String,
    /// Opposite asset token ID
    #[serde(rename = "oppositeAsset")]
    pub opposite_asset: String,
    /// Market end date
    #[serde(rename = "endDate")]
    pub end_date: String,
    /// Whether this is a negative risk market
    #[serde(rename = "negativeRisk")]
    pub negative_risk: bool,
}

/// Polymarket trade from Data API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataApiTrade {
    /// Trade ID
    pub id: String,
    /// Market condition ID
    #[serde(rename = "conditionId")]
    pub condition_id: String,
    /// Maker address
    pub maker: String,
    /// Taker address
    pub taker: String,
    /// Trade side
    pub side: String,
    /// Outcome
    pub outcome: String,
    /// Trade size
    pub size: String,
    /// Trade price
    pub price: String,
    /// Trade timestamp
    pub timestamp: String,
    /// Transaction hash
    #[serde(rename = "transactionHash", default)]
    pub transaction_hash: Option<String>,
}

/// User activity (trade/position change) from Data API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataApiActivity {
    /// Transaction hash
    #[serde(rename = "transactionHash")]
    pub transaction_hash: String,
    /// Activity timestamp (unix)
    pub timestamp: i64,
    /// User's proxy wallet address
    #[serde(rename = "proxyWallet")]
    pub proxy_wallet: String,
    /// User display name
    #[serde(default)]
    pub name: Option<String>,
    /// User pseudonym
    #[serde(default)]
    pub pseudonym: Option<String>,
    /// User bio
    #[serde(default)]
    pub bio: Option<String>,
    /// User profile image
    #[serde(rename = "profileImage", default)]
    pub profile_image: Option<String>,
    /// Optimized profile image
    #[serde(rename = "profileImageOptimized", default)]
    pub profile_image_optimized: Option<String>,
    /// Trade side (BUY/SELL)
    pub side: String,
    /// Outcome (Yes/No)
    pub outcome: String,
    /// Outcome index (0 or 1)
    #[serde(rename = "outcomeIndex")]
    pub outcome_index: i32,
    /// Trade price
    pub price: f64,
    /// Trade size
    pub size: f64,
    /// USDC size
    #[serde(rename = "usdcSize", default)]
    pub usdc_size: Option<f64>,
    /// Asset/token ID
    pub asset: String,
    /// Market condition ID
    #[serde(rename = "conditionId")]
    pub condition_id: String,
    /// Market title
    pub title: String,
    /// Market slug
    pub slug: String,
    /// Event slug
    #[serde(rename = "eventSlug")]
    pub event_slug: String,
    /// Market icon
    #[serde(default)]
    pub icon: Option<String>,
}

/// Biggest Winner entry from Data API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiggestWinner {
    /// Rank (string format)
    #[serde(rename = "winRank")]
    pub win_rank: String,
    /// Wallet address (0x...)
    #[serde(rename = "proxyWallet")]
    pub proxy_wallet: String,
    /// User name
    #[serde(rename = "userName", default)]
    pub user_name: String,
    /// Event slug
    #[serde(rename = "eventSlug")]
    pub event_slug: String,
    /// Event title
    #[serde(rename = "eventTitle")]
    pub event_title: String,
    /// Initial value (USD)
    #[serde(rename = "initialValue")]
    pub initial_value: f64,
    /// Final value (USD)
    #[serde(rename = "finalValue")]
    pub final_value: f64,
    /// Realized profit (USD)
    pub pnl: f64,
    /// Profile image URL
    #[serde(rename = "profileImage", default)]
    pub profile_image: String,
}

/// Query parameters for biggest winners API
#[derive(Debug, Clone)]
pub struct BiggestWinnersQuery {
    /// Time period: day, week, month, all_time
    pub time_period: String,
    /// Max results (max 100 per request)
    pub limit: usize,
    /// Pagination offset
    pub offset: usize,
    /// Category filter (lowercase): all, politics, sports, crypto, etc.
    pub category: String,
}

impl Default for BiggestWinnersQuery {
    fn default() -> Self {
        Self {
            time_period: "all_time".to_string(),
            limit: 100,
            offset: 0,
            category: "all".to_string(),
        }
    }
}

impl BiggestWinnersQuery {
    /// Create new query with defaults
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set time period
    #[must_use]
    pub fn with_time_period(mut self, period: impl Into<String>) -> Self {
        self.time_period = period.into();
        self
    }

    /// Set limit
    #[must_use]
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Set offset
    #[must_use]
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    /// Set category
    #[must_use]
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = category.into();
        self
    }
}

/// Sort field for positions query
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionSortBy {
    /// Sort by current value
    Current,
    /// Sort by initial value
    Initial,
    /// Sort by number of tokens
    Tokens,
    /// Sort by cash PnL
    CashPnl,
    /// Sort by percent PnL
    PercentPnl,
    /// Sort by title
    Title,
    /// Sort by resolving date
    Resolving,
    /// Sort by current price
    Price,
    /// Sort by average price
    AvgPrice,
}

impl PositionSortBy {
    /// Convert to API string
    pub fn as_str(&self) -> &str {
        match self {
            Self::Current => "CURRENT",
            Self::Initial => "INITIAL",
            Self::Tokens => "TOKENS",
            Self::CashPnl => "CASHPNL",
            Self::PercentPnl => "PERCENTPNL",
            Self::Title => "TITLE",
            Self::Resolving => "RESOLVING",
            Self::Price => "PRICE",
            Self::AvgPrice => "AVGPRICE",
        }
    }
}

/// Sort direction for positions query
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    /// Ascending
    Asc,
    /// Descending
    Desc,
}

impl SortDirection {
    /// Convert to API string
    pub fn as_str(&self) -> &str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

/// Query parameters for positions API
#[derive(Debug, Clone, Default)]
pub struct PositionsQuery {
    /// User address (required)
    pub user: String,
    /// Comma-separated list of condition IDs (mutually exclusive with event_ids)
    pub markets: Option<Vec<String>>,
    /// Comma-separated list of event IDs (mutually exclusive with markets)
    pub event_ids: Option<Vec<i64>>,
    /// Minimum position size threshold
    pub size_threshold: Option<f64>,
    /// Filter by redeemable positions
    pub redeemable: Option<bool>,
    /// Filter by mergeable positions
    pub mergeable: Option<bool>,
    /// Maximum number of results (0-500)
    pub limit: Option<u32>,
    /// Pagination offset (0-10000)
    pub offset: Option<u32>,
    /// Sort field
    pub sort_by: Option<PositionSortBy>,
    /// Sort direction
    pub sort_direction: Option<SortDirection>,
    /// Filter by title (partial match)
    pub title: Option<String>,
}

impl PositionsQuery {
    /// Create new query with user address
    #[must_use]
    pub fn new(user: impl Into<String>) -> Self {
        Self {
            user: user.into(),
            ..Default::default()
        }
    }

    /// Set markets filter (condition IDs)
    #[must_use]
    pub fn with_markets(mut self, markets: Vec<String>) -> Self {
        self.markets = Some(markets);
        self
    }

    /// Set event IDs filter
    #[must_use]
    pub fn with_event_ids(mut self, event_ids: Vec<i64>) -> Self {
        self.event_ids = Some(event_ids);
        self
    }

    /// Set size threshold
    #[must_use]
    pub fn with_size_threshold(mut self, threshold: f64) -> Self {
        self.size_threshold = Some(threshold);
        self
    }

    /// Filter redeemable positions only
    #[must_use]
    pub fn redeemable_only(mut self) -> Self {
        self.redeemable = Some(true);
        self
    }

    /// Filter mergeable positions only
    #[must_use]
    pub fn mergeable_only(mut self) -> Self {
        self.mergeable = Some(true);
        self
    }

    /// Set limit
    #[must_use]
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set offset
    #[must_use]
    pub fn with_offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Set sort field
    #[must_use]
    pub fn sort_by(mut self, sort_by: PositionSortBy) -> Self {
        self.sort_by = Some(sort_by);
        self
    }

    /// Set sort direction
    #[must_use]
    pub fn sort_direction(mut self, direction: SortDirection) -> Self {
        self.sort_direction = Some(direction);
        self
    }

    /// Filter by title
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Build URL query string
    pub fn to_query_string(&self) -> String {
        let mut params = vec![format!("user={}", self.user)];

        if let Some(markets) = &self.markets {
            if !markets.is_empty() {
                params.push(format!("market={}", markets.join(",")));
            }
        }

        if let Some(event_ids) = &self.event_ids {
            if !event_ids.is_empty() {
                let ids: Vec<String> = event_ids.iter().map(|id| id.to_string()).collect();
                params.push(format!("eventId={}", ids.join(",")));
            }
        }

        if let Some(threshold) = self.size_threshold {
            params.push(format!("sizeThreshold={}", threshold));
        }

        if let Some(redeemable) = self.redeemable {
            params.push(format!("redeemable={}", redeemable));
        }

        if let Some(mergeable) = self.mergeable {
            params.push(format!("mergeable={}", mergeable));
        }

        if let Some(limit) = self.limit {
            params.push(format!("limit={}", limit));
        }

        if let Some(offset) = self.offset {
            params.push(format!("offset={}", offset));
        }

        if let Some(sort_by) = &self.sort_by {
            params.push(format!("sortBy={}", sort_by.as_str()));
        }

        if let Some(direction) = &self.sort_direction {
            params.push(format!("sortDirection={}", direction.as_str()));
        }

        if let Some(title) = &self.title {
            params.push(format!("title={}", urlencoding::encode(title)));
        }

        params.join("&")
    }
}

// ============================================================================
// Public Search API Types
// ============================================================================

/// Search request parameters for /public-search endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    /// Search query string
    pub q: String,
    /// Limit per result type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_per_type: Option<u32>,
    /// Whether to search profiles (traders)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_profiles: Option<bool>,
    /// Whether to search tags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_tags: Option<bool>,
}

impl SearchRequest {
    /// Create a new search request
    #[must_use]
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            q: query.into(),
            limit_per_type: None,
            search_profiles: None,
            search_tags: None,
        }
    }

    /// Set limit per type
    #[must_use]
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit_per_type = Some(limit);
        self
    }

    /// Set whether to search profiles
    #[must_use]
    pub fn with_profiles(mut self, search_profiles: bool) -> Self {
        self.search_profiles = Some(search_profiles);
        self
    }

    /// Set whether to search tags
    #[must_use]
    pub fn with_tags(mut self, search_tags: bool) -> Self {
        self.search_tags = Some(search_tags);
        self
    }
}

/// Search response from /public-search endpoint
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchResponse {
    /// Matching events/markets
    #[serde(default)]
    pub events: Vec<SearchEvent>,
    /// Matching profiles/traders
    #[serde(default)]
    pub profiles: Vec<SearchProfile>,
    /// Matching tags
    #[serde(default)]
    pub tags: Vec<SearchTag>,
}

/// Search event result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEvent {
    /// Event ID
    #[serde(default)]
    pub id: String,
    /// Event slug
    #[serde(default)]
    pub slug: String,
    /// Event question/title
    #[serde(default)]
    pub question: Option<String>,
    /// Event image
    #[serde(default)]
    pub image: Option<String>,
    /// Whether event is active
    #[serde(default)]
    pub active: bool,
    /// Whether event is closed
    #[serde(default)]
    pub closed: bool,
    /// Total volume
    #[serde(default)]
    pub volume: f64,
    /// 24-hour volume
    #[serde(rename = "volume24hr", default)]
    pub volume_24hr: Option<f64>,
    /// End date
    #[serde(rename = "endDate", default)]
    pub end_date: Option<String>,
}

/// Search profile/trader result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchProfile {
    /// Profile ID
    #[serde(default)]
    pub id: Option<String>,
    /// Display name
    #[serde(default)]
    pub name: Option<String>,
    /// Old API field: imageURI
    #[serde(rename = "imageURI", default)]
    pub image_uri: Option<String>,
    /// New API field: profileImage
    #[serde(rename = "profileImage", default)]
    pub profile_image: Option<String>,
    /// Bio/description
    #[serde(default)]
    pub bio: Option<String>,
    /// Pseudonym
    #[serde(default)]
    pub pseudonym: Option<String>,
    /// Whether to display username publicly
    #[serde(rename = "displayUsernamePublic", default)]
    pub display_username_public: bool,
    /// Old API field: walletAddress
    #[serde(rename = "walletAddress", default)]
    pub wallet_address: Option<String>,
    /// New API field: proxyWallet
    #[serde(rename = "proxyWallet", default)]
    pub proxy_wallet: Option<String>,
}

impl SearchProfile {
    /// Get wallet address (prefer proxy_wallet, fallback to wallet_address)
    #[must_use]
    pub fn get_wallet_address(&self) -> Option<String> {
        self.proxy_wallet
            .clone()
            .or_else(|| self.wallet_address.clone())
    }

    /// Get profile image (prefer profile_image, fallback to image_uri)
    #[must_use]
    pub fn get_profile_image(&self) -> Option<String> {
        self.profile_image
            .clone()
            .or_else(|| self.image_uri.clone())
    }

    /// Get display name (prefer name, fallback to pseudonym)
    #[must_use]
    pub fn get_display_name(&self) -> Option<String> {
        self.name.clone().or_else(|| self.pseudonym.clone())
    }
}

/// Search tag result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchTag {
    /// Tag ID
    pub id: String,
    /// Tag label
    pub label: String,
    /// Tag slug
    #[serde(default)]
    pub slug: Option<String>,
}

/// Closed position from Data API (for PnL calculation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosedPosition {
    /// Position ID
    #[serde(default)]
    pub id: Option<String>,
    /// Proxy wallet address
    #[serde(rename = "proxyWallet", default)]
    pub proxy_wallet: Option<String>,
    /// Token asset ID
    #[serde(default)]
    pub asset: Option<String>,
    /// Market condition ID
    #[serde(rename = "conditionId")]
    pub condition_id: String,
    /// Market title
    pub title: String,
    /// Market slug
    pub slug: String,
    /// Event slug
    #[serde(rename = "eventSlug")]
    pub event_slug: String,
    /// Outcome (Yes/No)
    pub outcome: String,
    /// Outcome index
    #[serde(rename = "outcomeIndex")]
    pub outcome_index: i32,
    /// Entry price
    #[serde(rename = "avgPrice")]
    pub avg_price: f64,
    /// Current price
    #[serde(rename = "curPrice", default)]
    pub cur_price: Option<f64>,
    /// Exit price
    #[serde(rename = "exitPrice", default)]
    pub exit_price: Option<f64>,
    /// Position size (shares)
    #[serde(default)]
    pub size: Option<f64>,
    /// Total bought amount (USDC)
    #[serde(rename = "totalBought", default)]
    pub total_bought: Option<f64>,
    /// Realized PnL
    #[serde(rename = "realizedPnl", default)]
    pub realized_pnl: Option<f64>,
    /// Cash out amount
    #[serde(rename = "cashOut", default)]
    pub cash_out: Option<f64>,
    /// Is winning position
    #[serde(rename = "isWinner", default)]
    pub is_winner: Option<bool>,
    /// Closed timestamp (unix)
    #[serde(default)]
    pub timestamp: Option<i64>,
    /// Closed timestamp (ISO string)
    #[serde(rename = "closedAt", default)]
    pub closed_at: Option<String>,
    /// End date
    #[serde(rename = "endDate", default)]
    pub end_date: Option<String>,
    /// Market icon
    #[serde(default)]
    pub icon: Option<String>,
    /// Opposite outcome name
    #[serde(rename = "oppositeOutcome", default)]
    pub opposite_outcome: Option<String>,
    /// Opposite asset ID
    #[serde(rename = "oppositeAsset", default)]
    pub opposite_asset: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that NewOrder JSON serialization matches TypeScript SDK field order
    /// This is critical for HMAC signature compatibility
    #[test]
    fn test_new_order_json_field_order() {
        let order = NewOrder {
            defer_exec: false,
            order: NewOrderData {
                salt: 2915952280710976,
                maker: "0xc2ca793cf057d48a054bedabf625f301b40d38aa".to_string(),
                signer: "0xd13765b3e68431bf2b6e9994a0f4c3d2495799e9".to_string(),
                taker: "0x0000000000000000000000000000000000000000".to_string(),
                token_id: "21489772516410038586556744342392982044189999368638682594741395650226594484811".to_string(),
                maker_amount: "10000".to_string(),
                taker_amount: "1000000".to_string(),
                side: "BUY".to_string(),
                expiration: "0".to_string(),
                nonce: "0".to_string(),
                fee_rate_bps: "0".to_string(),
                signature_type: 1,
                signature: "0x0cfb0e318afe33e1189f23d4b11a1092963865d7ff7f7a035110d50d71a2ab484ae4828b3fcfcac2ada92fbd825eedfe4eb21d4e1cdd5aa1a47e23bf5d539b781c".to_string(),
            },
            owner: "fe9fb6b1-9ae6-6c5b-3cca-1ace6a8b1f29".to_string(),
            order_type: OrderType::GTC,
        };

        let json = serde_json::to_string(&order).unwrap();

        // Expected format from TypeScript SDK (field order matters for HMAC)
        let expected = r#"{"deferExec":false,"order":{"salt":2915952280710976,"maker":"0xc2ca793cf057d48a054bedabf625f301b40d38aa","signer":"0xd13765b3e68431bf2b6e9994a0f4c3d2495799e9","taker":"0x0000000000000000000000000000000000000000","tokenId":"21489772516410038586556744342392982044189999368638682594741395650226594484811","makerAmount":"10000","takerAmount":"1000000","side":"BUY","expiration":"0","nonce":"0","feeRateBps":"0","signatureType":1,"signature":"0x0cfb0e318afe33e1189f23d4b11a1092963865d7ff7f7a035110d50d71a2ab484ae4828b3fcfcac2ada92fbd825eedfe4eb21d4e1cdd5aa1a47e23bf5d539b781c"},"owner":"fe9fb6b1-9ae6-6c5b-3cca-1ace6a8b1f29","orderType":"GTC"}"#;

        assert_eq!(
            json, expected,
            "\nJSON field order mismatch!\n\nGot:\n{}\n\nExpected:\n{}\n",
            json, expected
        );
    }
}
