//! WebSocket Streaming Example
//!
//! This example demonstrates real-time data streaming using
//! the RTDS (Real-Time Data Stream) and CLOB WebSocket connections.
//!
//! Run with: `cargo run --example websocket_stream --features stream`

use polymarket_sdk::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for better debugging
    tracing_subscriber::fmt::init();

    println!("=== Polymarket WebSocket Streaming ===\n");

    // Example market asset IDs (replace with real ones from Gamma API)
    // You can get these from: market.parse_token_ids()
    let asset_ids = vec![
        "21742633143463906290569050155826241533067272736897614950488156847949938836455".to_string(),
    ];

    // Demo 1: RTDS Stream (Real-Time Data Stream for live trades)
    println!("1. RTDS Stream Demo (trades and activity)...");
    demo_rtds_stream().await?;

    // Demo 2: CLOB WebSocket (for order book and price changes)
    println!("\n2. CLOB WebSocket Demo (price updates)...");
    demo_clob_stream(&asset_ids).await?;

    println!("\n=== Done! ===");
    Ok(())
}

async fn demo_rtds_stream() -> Result<()> {
    println!("Creating RTDS client...");

    // Create RTDS client with default configuration
    let mut client = RtdsClient::with_defaults();

    // Connect returns a receiver channel for events
    let mut event_rx = client.connect().await?;

    println!("Connected to RTDS, listening for events...");
    println!("(Will timeout after 10 seconds)\n");

    // Listen for events with a timeout
    let timeout = Duration::from_secs(10);
    let start = tokio::time::Instant::now();

    loop {
        tokio::select! {
            Some(event) = event_rx.recv() => {
                match event {
                    RtdsEvent::Connected => {
                        println!("[RTDS] Connected to server");
                    }
                    RtdsEvent::Trade(trade) => {
                        // Print trade details
                        if let Some(asset) = &trade.asset {
                            println!(
                                "[TRADE] {} | Outcome: {} | Price: ${:.2}",
                                &asset[..20.min(asset.len())],
                                trade.outcome.as_deref().unwrap_or("?"),
                                trade.price.unwrap_or(0.0)
                            );
                        }
                    }
                    RtdsEvent::Disconnected => {
                        println!("[RTDS] Disconnected");
                        break;
                    }
                    RtdsEvent::Error(e) => {
                        println!("[RTDS] Error: {}", e);
                    }
                    RtdsEvent::Reconnecting { attempt, backoff_secs } => {
                        println!("[RTDS] Reconnecting (attempt {}, backoff {}s)", attempt, backoff_secs);
                    }
                    RtdsEvent::RawMessage(msg) => {
                        println!("[RTDS] Message type: {}", msg.msg_type);
                    }
                }
            }
            _ = tokio::time::sleep_until(start + timeout) => {
                println!("[RTDS] Timeout reached");
                break;
            }
        }
    }

    // Clean shutdown
    client.disconnect().await;
    println!("RTDS demo complete");
    Ok(())
}

async fn demo_clob_stream(asset_ids: &[String]) -> Result<()> {
    println!("Creating CLOB WebSocket client...");

    // Create and configure market client
    let mut client = WssMarketClient::new();

    // Subscribe to specific assets
    client.subscribe(asset_ids.to_vec()).await?;

    println!(
        "Subscribed to {} asset(s), listening for price updates...",
        asset_ids.len()
    );
    println!("(Will timeout after 10 seconds)\n");

    // Listen for events with a timeout
    let timeout = Duration::from_secs(10);
    let start = tokio::time::Instant::now();

    loop {
        // Check timeout
        if start.elapsed() > timeout {
            println!("[WSS] Timeout reached");
            break;
        }

        // Try to get next event with a small timeout
        tokio::select! {
            result = client.next_event() => {
                match result {
                    Ok(WssMarketEvent::Book(book)) => {
                        println!(
                            "[BOOK] Market: {} | Bids: {} | Asks: {}",
                            &book.market[..20.min(book.market.len())],
                            book.bids.len(),
                            book.asks.len()
                        );
                    }
                    Ok(WssMarketEvent::PriceChange(msg)) => {
                        println!(
                            "[PRICE] Market: {} | {} change(s)",
                            &msg.market[..20.min(msg.market.len())],
                            msg.price_changes.len()
                        );
                        for change in msg.price_changes.iter().take(2) {
                            println!(
                                "  {:?} @ {} (size: {})",
                                change.side,
                                change.price,
                                change.size
                            );
                        }
                    }
                    Ok(WssMarketEvent::LastTrade(trade)) => {
                        println!(
                            "[TRADE] Asset: {}... | Price: {}",
                            &trade.asset_id[..20.min(trade.asset_id.len())],
                            trade.price
                        );
                    }
                    Ok(WssMarketEvent::TickSizeChange(msg)) => {
                        println!(
                            "[TICK] Asset: {}... | {} -> {}",
                            &msg.asset_id[..20.min(msg.asset_id.len())],
                            msg.old_tick_size,
                            msg.new_tick_size
                        );
                    }
                    Err(e) => {
                        println!("[WSS] Error: {:?}", e);
                        break;
                    }
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                // Brief sleep to prevent tight loop
            }
        }
    }

    // Clean shutdown
    client.close().await;
    println!("CLOB WebSocket demo complete");
    Ok(())
}
