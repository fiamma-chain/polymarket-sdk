//! WebSocket streaming clients for real-time data.
//!
//! This module provides WebSocket clients for:
//!
//! - [`RtdsClient`] - Real-Time Data Stream for live trades
//! - [`WssMarketClient`] - CLOB WebSocket for order book updates
//! - [`WssUserClient`] - CLOB WebSocket for user events

pub mod clob;
pub mod rtds;

// RTDS exports
pub use rtds::{
    MarketStream, MockStream, RtdsClient, RtdsConfig, RtdsEvent, RtdsMessage, RtdsSubscription,
    RtdsSubscriptionMessage, StreamManager, StreamMessage, StreamStats, Subscription,
    TradePayload, WebSocketStream, WssAuth, WssSubscription,
};

// CLOB WebSocket exports
pub use clob::{
    LastTradeMessage, MarketBook, OrderSummary, PriceChangeEntry, PriceChangeMessage,
    TickSizeChangeMessage, WssMarketClient, WssMarketEvent, WssStats, WssUserClient,
    WssUserEvent, WssUserOrderMessage, WssUserTradeMessage,
};
