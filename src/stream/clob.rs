//! WebSocket client for Polymarket CLOB real-time updates
//!
//! This module provides reconnecting WebSocket clients for both public market
//! channel and authenticated user channel streams.
//!
//! ## Features
//!
//! - **Market Channel**: Public market data (order books, price changes, trades)
//! - **User Channel**: Authenticated user events (trades, orders)
//! - **Auto-Reconnection**: Exponential backoff reconnection strategy
//! - **Connection Stats**: Monitoring for connection health
//!
//! ## Example
//!
//! ```rust,ignore
//! use polymarket_sdk::wss::{WssMarketClient, WssMarketEvent};
//!
//! // Market channel for public data
//! let mut client = WssMarketClient::new();
//! client.subscribe(vec!["token_id_1".to_string()]).await?;
//!
//! while let Ok(event) = client.next_event().await {
//!     match event {
//!         WssMarketEvent::Book(book) => println!("Book: {:?}", book),
//!         WssMarketEvent::PriceChange(change) => println!("Price: {:?}", change),
//!         _ => {}
//!     }
//! }
//! ```

use crate::core::CLOB_WSS_BASE;
use crate::core::{PolymarketError, Result, StreamErrorKind};
use crate::types::ApiCredentials;
use chrono::{DateTime, Utc};
use futures::{SinkExt, StreamExt};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::{sleep, timeout};
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use tracing::warn;

use crate::types::Side;

const MARKET_CHANNEL_PATH: &str = "/ws/market";
const USER_CHANNEL_PATH: &str = "/ws/user";
const BASE_RECONNECT_DELAY: Duration = Duration::from_millis(250);
const MAX_RECONNECT_DELAY: Duration = Duration::from_secs(10);
const MAX_RECONNECT_ATTEMPTS: u32 = 8;
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(25);

// ============================================================================
// Event Types
// ============================================================================

/// Parsed market broadcast events from the public market channel
#[derive(Debug, Clone)]
pub enum WssMarketEvent {
    /// Order book snapshot
    Book(MarketBook),
    /// Price level change
    PriceChange(PriceChangeMessage),
    /// Tick size change for a market
    TickSizeChange(TickSizeChangeMessage),
    /// Last trade notification
    LastTrade(LastTradeMessage),
}

/// Events emitted by the authenticated user channel
#[derive(Debug, Clone)]
pub enum WssUserEvent {
    /// Trade execution for the user
    Trade(WssUserTradeMessage),
    /// Order update for the user
    Order(WssUserOrderMessage),
}

// ============================================================================
// Market Channel Types
// ============================================================================

/// Order book summary message
#[derive(Debug, Clone, Deserialize)]
pub struct MarketBook {
    #[serde(rename = "event_type")]
    pub event_type: String,
    pub asset_id: String,
    pub market: String,
    pub timestamp: String,
    pub hash: String,
    pub bids: Vec<OrderSummary>,
    pub asks: Vec<OrderSummary>,
}

/// Order summary (price/size at a level)
#[derive(Debug, Clone, Deserialize)]
pub struct OrderSummary {
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub size: Decimal,
}

/// Price change notification payload
#[derive(Debug, Clone, Deserialize)]
pub struct PriceChangeMessage {
    #[serde(rename = "event_type")]
    pub event_type: String,
    pub market: String,
    #[serde(rename = "price_changes")]
    pub price_changes: Vec<PriceChangeEntry>,
    pub timestamp: String,
}

/// Individual price change entry
#[derive(Debug, Clone, Deserialize)]
pub struct PriceChangeEntry {
    pub asset_id: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub size: Decimal,
    pub side: Side,
    pub hash: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub best_bid: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub best_ask: Decimal,
}

/// Tick size change event
#[derive(Debug, Clone, Deserialize)]
pub struct TickSizeChangeMessage {
    #[serde(rename = "event_type")]
    pub event_type: String,
    pub asset_id: String,
    pub market: String,
    #[serde(rename = "old_tick_size", with = "rust_decimal::serde::str")]
    pub old_tick_size: Decimal,
    #[serde(rename = "new_tick_size", with = "rust_decimal::serde::str")]
    pub new_tick_size: Decimal,
    pub side: String,
    pub timestamp: String,
}

/// Last trade notification
#[derive(Debug, Clone, Deserialize)]
pub struct LastTradeMessage {
    #[serde(rename = "event_type")]
    pub event_type: String,
    pub asset_id: String,
    pub fee_rate_bps: String,
    pub market: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub size: Decimal,
    pub side: Side,
    pub timestamp: String,
}

// ============================================================================
// User Channel Types
// ============================================================================

/// Trade notification for authenticated user
#[derive(Debug, Clone, Deserialize)]
pub struct WssUserTradeMessage {
    #[serde(rename = "event_type")]
    pub event_type: String,
    pub asset_id: String,
    pub id: String,
    pub last_update: String,
    #[serde(default)]
    pub maker_orders: Vec<MakerOrder>,
    pub market: String,
    pub matchtime: String,
    pub outcome: String,
    pub owner: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Decimal,
    pub side: Side,
    #[serde(with = "rust_decimal::serde::str")]
    pub size: Decimal,
    pub status: String,
    pub taker_order_id: String,
    pub timestamp: String,
    pub trade_owner: String,
    #[serde(rename = "type")]
    pub message_type: String,
}

/// Maker order details in trade events
#[derive(Debug, Clone, Deserialize)]
pub struct MakerOrder {
    pub asset_id: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub matched_amount: Decimal,
    pub order_id: String,
    pub outcome: String,
    pub owner: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Decimal,
}

/// Order notification for authenticated user
#[derive(Debug, Clone, Deserialize)]
pub struct WssUserOrderMessage {
    #[serde(rename = "event_type")]
    pub event_type: String,
    #[serde(default)]
    pub associate_trades: Option<Vec<String>>,
    pub asset_id: String,
    pub id: String,
    pub market: String,
    pub order_owner: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub original_size: Decimal,
    pub outcome: String,
    pub owner: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Decimal,
    pub side: Side,
    #[serde(with = "rust_decimal::serde::str")]
    pub size_matched: Decimal,
    pub timestamp: String,
    #[serde(rename = "type")]
    pub message_type: String,
}

// ============================================================================
// Connection Statistics
// ============================================================================

/// Connection health statistics
#[derive(Debug, Clone, Default)]
pub struct WssStats {
    /// Total messages received
    pub messages_received: u64,
    /// Total errors encountered
    pub errors: u64,
    /// Number of reconnections
    pub reconnect_count: u32,
    /// Timestamp of last message
    pub last_message_time: Option<DateTime<Utc>>,
}

// ============================================================================
// Market Channel Client
// ============================================================================

/// Reconnecting WebSocket client for the public market channel
///
/// Provides access to order book snapshots, price changes, tick size changes,
/// and last trade notifications for subscribed markets.
pub struct WssMarketClient {
    connect_url: String,
    connection: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    subscribed_asset_ids: Vec<String>,
    stats: WssStats,
    disconnect_history: VecDeque<DateTime<Utc>>,
    pending_events: VecDeque<WssMarketEvent>,
}

impl Default for WssMarketClient {
    fn default() -> Self {
        Self::new()
    }
}

impl WssMarketClient {
    /// Create a new market client with default Polymarket WSS endpoint
    #[must_use]
    pub fn new() -> Self {
        Self::with_url(CLOB_WSS_BASE)
    }

    /// Create a new market client with custom endpoint
    #[must_use]
    pub fn with_url(url: &str) -> Self {
        let trimmed = url.trim_end_matches('/');
        let connect_url = format!("{}{}", trimmed, MARKET_CHANNEL_PATH);
        Self {
            connection: None,
            subscribed_asset_ids: Vec::new(),
            stats: WssStats::default(),
            disconnect_history: VecDeque::with_capacity(5),
            connect_url,
            pending_events: VecDeque::new(),
        }
    }

    /// Get connection statistics
    #[must_use]
    pub fn stats(&self) -> WssStats {
        self.stats.clone()
    }

    /// Check if connected
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    fn format_subscription(&self) -> Value {
        // Polymarket CLOB WebSocket expects uppercase "MARKET" for the type field
        // Reference: https://docs.polymarket.com/developers/CLOB/websocket/wss-overview
        json!({
            "type": "MARKET",
            "assets_ids": self.subscribed_asset_ids,
        })
    }

    async fn send_subscription(&mut self) -> Result<()> {
        if self.subscribed_asset_ids.is_empty() {
            warn!("No asset_ids to subscribe, skipping subscription");
            return Ok(());
        }

        let message = self.format_subscription();
        tracing::info!(
            asset_count = self.subscribed_asset_ids.len(),
            first_asset = %self.subscribed_asset_ids.first().map(|s| s.chars().take(20).collect::<String>()).unwrap_or_default(),
            "Sending market subscription"
        );
        self.send_raw_message(message).await
    }

    async fn send_raw_message(&mut self, message: Value) -> Result<()> {
        if let Some(connection) = self.connection.as_mut() {
            let text = serde_json::to_string(&message).map_err(|e| PolymarketError::Parse {
                message: format!("Failed to serialize subscription message: {}", e),
                source: Some(Box::new(e)),
            })?;
            tracing::debug!(message = %text, "Sending WebSocket message");
            connection
                .send(Message::Text(text.into()))
                .await
                .map_err(|e| {
                    PolymarketError::stream(
                        format!("Failed to send message: {}", e),
                        StreamErrorKind::MessageCorrupted,
                    )
                })?;
            return Ok(());
        }
        Err(PolymarketError::stream(
            "WebSocket connection not established",
            StreamErrorKind::ConnectionFailed,
        ))
    }

    async fn connect(&mut self) -> Result<()> {
        let mut attempts = 0;
        tracing::info!(url = %self.connect_url, "Connecting to Polymarket WebSocket");
        loop {
            match connect_async(&self.connect_url).await {
                Ok((socket, response)) => {
                    tracing::info!(
                        status = ?response.status(),
                        "WebSocket connection established"
                    );
                    self.connection = Some(socket);
                    if attempts > 0 {
                        self.stats.reconnect_count += 1;
                    }
                    return Ok(());
                }
                Err(err) => {
                    attempts += 1;
                    let delay = self.reconnect_delay(attempts);
                    self.stats.errors += 1;
                    tracing::warn!(
                        error = %err,
                        attempt = attempts,
                        max_attempts = MAX_RECONNECT_ATTEMPTS,
                        "WebSocket connection failed, retrying"
                    );
                    if attempts >= MAX_RECONNECT_ATTEMPTS {
                        return Err(PolymarketError::stream(
                            format!("Failed to connect after {} attempts: {}", attempts, err),
                            StreamErrorKind::ConnectionFailed,
                        ));
                    }
                    sleep(delay).await;
                }
            }
        }
    }

    fn reconnect_delay(&self, attempts: u32) -> Duration {
        let millis = BASE_RECONNECT_DELAY.as_millis() * attempts as u128;
        Duration::from_millis(millis.min(MAX_RECONNECT_DELAY.as_millis()) as u64)
    }

    async fn ensure_connection(&mut self) -> Result<()> {
        if self.connection.is_none() {
            self.connect().await?;
            self.send_subscription().await?;
        }
        Ok(())
    }

    /// Subscribe to market channel for specified asset IDs
    ///
    /// # Arguments
    ///
    /// * `asset_ids` - List of token/asset IDs to subscribe to
    pub async fn subscribe(&mut self, asset_ids: Vec<String>) -> Result<()> {
        self.subscribed_asset_ids = asset_ids;
        self.ensure_connection().await?;
        self.send_subscription().await
    }

    /// Get next market event, reconnecting transparently on connection drop
    pub async fn next_event(&mut self) -> Result<WssMarketEvent> {
        loop {
            if let Some(evt) = self.pending_events.pop_front() {
                return Ok(evt);
            }
            self.ensure_connection().await?;

            match timeout(KEEPALIVE_INTERVAL, self.connection.as_mut().unwrap().next()).await {
                Ok(Some(Ok(Message::Text(text)))) => {
                    let trimmed = text.trim();
                    if trimmed.eq_ignore_ascii_case("ping") || trimmed.eq_ignore_ascii_case("pong")
                    {
                        continue;
                    }
                    let first_char = trimmed.chars().next();
                    if first_char != Some('{') && first_char != Some('[') {
                        warn!("ignoring unexpected text frame: {}", trimmed);
                        continue;
                    }
                    let events = parse_market_events(&text)?;
                    self.stats.messages_received += events.len() as u64;
                    self.stats.last_message_time = Some(Utc::now());
                    for evt in events {
                        self.pending_events.push_back(evt);
                    }
                    if let Some(evt) = self.pending_events.pop_front() {
                        return Ok(evt);
                    }
                    continue;
                }
                Ok(Some(Ok(Message::Ping(payload)))) => {
                    if let Some(connection) = self.connection.as_mut() {
                        let _ = connection.send(Message::Pong(payload)).await;
                    }
                }
                Ok(Some(Ok(Message::Pong(_)))) => {}
                Ok(Some(Ok(Message::Close(_)))) => {
                    self.disconnect_history.push_back(Utc::now());
                    if self.disconnect_history.len() > 5 {
                        self.disconnect_history.pop_front();
                    }
                    self.connection = None;
                }
                Ok(Some(Ok(_))) => {}
                Ok(Some(Err(err))) => {
                    warn!("WebSocket error: {}", err);
                    self.connection = None;
                    self.stats.errors += 1;
                    continue;
                }
                Ok(None) => {
                    self.connection = None;
                }
                Err(_) => {
                    // Keepalive timeout - send proper WebSocket ping frame to keep connection alive
                    if let Some(connection) = self.connection.as_mut() {
                        let _ = connection.send(Message::Ping(vec![].into())).await;
                    }
                }
            }
        }
    }

    /// Close the connection
    pub async fn close(&mut self) {
        if let Some(mut conn) = self.connection.take() {
            let _ = conn.close(None).await;
        }
    }
}

// ============================================================================
// User Channel Client
// ============================================================================

/// Reconnecting WebSocket client for the authenticated user channel
///
/// Provides access to user-specific trade and order events.
/// Requires API credentials for authentication.
pub struct WssUserClient {
    connect_url: String,
    connection: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    subscribed_markets: Vec<String>,
    stats: WssStats,
    disconnect_history: VecDeque<DateTime<Utc>>,
    pending_events: VecDeque<WssUserEvent>,
    auth: ApiCredentials,
}

impl WssUserClient {
    /// Create a new user client with default Polymarket WSS endpoint
    #[must_use]
    pub fn new(auth: ApiCredentials) -> Self {
        Self::with_url(CLOB_WSS_BASE, auth)
    }

    /// Create a new user client with custom endpoint
    #[must_use]
    pub fn with_url(url: &str, auth: ApiCredentials) -> Self {
        let trimmed = url.trim_end_matches('/');
        let connect_url = format!("{}{}", trimmed, USER_CHANNEL_PATH);
        Self {
            connection: None,
            subscribed_markets: Vec::new(),
            stats: WssStats::default(),
            disconnect_history: VecDeque::with_capacity(5),
            connect_url,
            pending_events: VecDeque::new(),
            auth,
        }
    }

    /// Get connection statistics
    #[must_use]
    pub fn stats(&self) -> WssStats {
        self.stats.clone()
    }

    /// Check if connected
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    fn format_subscription(&self) -> Option<Value> {
        if self.subscribed_markets.is_empty() {
            return None;
        }

        // Polymarket CLOB WebSocket expects uppercase "USER" for the type field
        Some(json!({
            "type": "USER",
            "auth": {
                "apiKey": self.auth.api_key,
                "secret": self.auth.secret,
                "passphrase": self.auth.passphrase,
            },
            "markets": self.subscribed_markets,
        }))
    }

    async fn send_subscription(&mut self) -> Result<()> {
        if let Some(message) = self.format_subscription() {
            self.send_raw_message(message).await
        } else {
            Ok(())
        }
    }

    async fn send_raw_message(&mut self, message: Value) -> Result<()> {
        if let Some(connection) = self.connection.as_mut() {
            let text = serde_json::to_string(&message).map_err(|e| PolymarketError::Parse {
                message: format!("Failed to serialize subscription message: {}", e),
                source: Some(Box::new(e)),
            })?;
            connection
                .send(Message::Text(text.into()))
                .await
                .map_err(|e| {
                    PolymarketError::stream(
                        format!("Failed to send message: {}", e),
                        StreamErrorKind::MessageCorrupted,
                    )
                })?;
            return Ok(());
        }
        Err(PolymarketError::stream(
            "WebSocket connection not established",
            StreamErrorKind::ConnectionFailed,
        ))
    }

    async fn connect(&mut self) -> Result<()> {
        let mut attempts = 0;
        loop {
            match connect_async(&self.connect_url).await {
                Ok((socket, _)) => {
                    self.connection = Some(socket);
                    if attempts > 0 {
                        self.stats.reconnect_count += 1;
                    }
                    return Ok(());
                }
                Err(err) => {
                    attempts += 1;
                    let delay = self.reconnect_delay(attempts);
                    self.stats.errors += 1;
                    if attempts >= MAX_RECONNECT_ATTEMPTS {
                        return Err(PolymarketError::stream(
                            format!("Failed to connect after {} attempts: {}", attempts, err),
                            StreamErrorKind::ConnectionFailed,
                        ));
                    }
                    sleep(delay).await;
                }
            }
        }
    }

    fn reconnect_delay(&self, attempts: u32) -> Duration {
        let millis = BASE_RECONNECT_DELAY.as_millis() * attempts as u128;
        Duration::from_millis(millis.min(MAX_RECONNECT_DELAY.as_millis()) as u64)
    }

    async fn ensure_connection(&mut self) -> Result<()> {
        if self.connection.is_none() {
            self.connect().await?;
            self.send_subscription().await?;
        }
        Ok(())
    }

    /// Subscribe to user channel for specified market IDs
    ///
    /// # Arguments
    ///
    /// * `market_ids` - List of market/condition IDs to subscribe to
    pub async fn subscribe(&mut self, market_ids: Vec<String>) -> Result<()> {
        self.subscribed_markets = market_ids;
        self.ensure_connection().await?;
        self.send_subscription().await
    }

    /// Get next user event, reconnecting transparently on connection drop
    pub async fn next_event(&mut self) -> Result<WssUserEvent> {
        loop {
            if let Some(evt) = self.pending_events.pop_front() {
                return Ok(evt);
            }
            self.ensure_connection().await?;

            match timeout(KEEPALIVE_INTERVAL, self.connection.as_mut().unwrap().next()).await {
                Ok(Some(Ok(Message::Text(text)))) => {
                    let trimmed = text.trim();
                    if trimmed.eq_ignore_ascii_case("ping") || trimmed.eq_ignore_ascii_case("pong")
                    {
                        continue;
                    }
                    let first_char = trimmed.chars().next();
                    if first_char != Some('{') && first_char != Some('[') {
                        warn!("ignoring unexpected text frame: {}", trimmed);
                        continue;
                    }
                    let events = parse_user_events(&text)?;
                    self.stats.messages_received += events.len() as u64;
                    self.stats.last_message_time = Some(Utc::now());
                    for evt in events {
                        self.pending_events.push_back(evt);
                    }
                    if let Some(evt) = self.pending_events.pop_front() {
                        return Ok(evt);
                    }
                    continue;
                }
                Ok(Some(Ok(Message::Ping(payload)))) => {
                    if let Some(connection) = self.connection.as_mut() {
                        let _ = connection.send(Message::Pong(payload)).await;
                    }
                }
                Ok(Some(Ok(Message::Pong(_)))) => {}
                Ok(Some(Ok(Message::Close(_)))) => {
                    self.disconnect_history.push_back(Utc::now());
                    if self.disconnect_history.len() > 5 {
                        self.disconnect_history.pop_front();
                    }
                    self.connection = None;
                }
                Ok(Some(Ok(_))) => {}
                Ok(Some(Err(err))) => {
                    warn!("WebSocket error: {}", err);
                    self.connection = None;
                    self.stats.errors += 1;
                    continue;
                }
                Ok(None) => {
                    self.connection = None;
                }
                Err(_) => {
                    // Keepalive timeout - send proper WebSocket ping frame
                    if let Some(connection) = self.connection.as_mut() {
                        let _ = connection.send(Message::Ping(vec![].into())).await;
                    }
                }
            }
        }
    }

    /// Close the connection
    pub async fn close(&mut self) {
        if let Some(mut conn) = self.connection.take() {
            let _ = conn.close(None).await;
        }
    }
}

// ============================================================================
// Parsing Functions
// ============================================================================

fn parse_market_events(text: &str) -> Result<Vec<WssMarketEvent>> {
    let value: Value = serde_json::from_str(text).map_err(|err| PolymarketError::Parse {
        message: format!("Invalid JSON: {}", err),
        source: Some(Box::new(err)),
    })?;

    if let Some(array) = value.as_array() {
        array
            .iter()
            .map(parse_market_event_value)
            .collect::<Result<Vec<_>>>()
    } else {
        Ok(vec![parse_market_event_value(&value)?])
    }
}

fn parse_market_event_value(value: &Value) -> Result<WssMarketEvent> {
    let event_type = value
        .get("event_type")
        .and_then(|v| v.as_str())
        .or_else(|| value.get("type").and_then(|v| v.as_str()))
        .ok_or_else(|| PolymarketError::parse("Missing event_type/type in market message"))?;

    match event_type {
        "book" => {
            let parsed: MarketBook =
                serde_json::from_value(value.clone()).map_err(|err| PolymarketError::Parse {
                    message: format!("Failed to parse book message: {}", err),
                    source: Some(Box::new(err)),
                })?;
            Ok(WssMarketEvent::Book(parsed))
        }
        "price_change" => {
            let parsed =
                serde_json::from_value::<PriceChangeMessage>(value.clone()).map_err(|err| {
                    PolymarketError::Parse {
                        message: format!("Failed to parse price_change: {}", err),
                        source: Some(Box::new(err)),
                    }
                })?;
            Ok(WssMarketEvent::PriceChange(parsed))
        }
        "tick_size_change" => {
            let parsed =
                serde_json::from_value::<TickSizeChangeMessage>(value.clone()).map_err(|err| {
                    PolymarketError::Parse {
                        message: format!("Failed to parse tick_size_change: {}", err),
                        source: Some(Box::new(err)),
                    }
                })?;
            Ok(WssMarketEvent::TickSizeChange(parsed))
        }
        "last_trade_price" => {
            let parsed =
                serde_json::from_value::<LastTradeMessage>(value.clone()).map_err(|err| {
                    PolymarketError::Parse {
                        message: format!("Failed to parse last_trade_price: {}", err),
                        source: Some(Box::new(err)),
                    }
                })?;
            Ok(WssMarketEvent::LastTrade(parsed))
        }
        other => Err(PolymarketError::parse(format!(
            "Unknown market event_type: {}",
            other
        ))),
    }
}

fn parse_user_events(text: &str) -> Result<Vec<WssUserEvent>> {
    let value: Value = serde_json::from_str(text).map_err(|err| PolymarketError::Parse {
        message: format!("Invalid JSON: {}", err),
        source: Some(Box::new(err)),
    })?;

    if let Some(array) = value.as_array() {
        array
            .iter()
            .map(parse_user_event_value)
            .collect::<Result<Vec<_>>>()
    } else {
        Ok(vec![parse_user_event_value(&value)?])
    }
}

fn parse_user_event_value(value: &Value) -> Result<WssUserEvent> {
    let event_type = value
        .get("event_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| PolymarketError::parse("Missing event_type in user message"))?;

    match event_type {
        "trade" => {
            let parsed =
                serde_json::from_value::<WssUserTradeMessage>(value.clone()).map_err(|err| {
                    PolymarketError::Parse {
                        message: format!("Failed to parse user trade message: {}", err),
                        source: Some(Box::new(err)),
                    }
                })?;
            Ok(WssUserEvent::Trade(parsed))
        }
        "order" => {
            let parsed =
                serde_json::from_value::<WssUserOrderMessage>(value.clone()).map_err(|err| {
                    PolymarketError::Parse {
                        message: format!("Failed to parse user order message: {}", err),
                        source: Some(Box::new(err)),
                    }
                })?;
            Ok(WssUserEvent::Order(parsed))
        }
        other => Err(PolymarketError::parse(format!(
            "Unknown user event_type: {}",
            other
        ))),
    }
}
