//! Example: Fetching user positions with various filters
//!
//! This example demonstrates how to use the enhanced positions API to:
//! - Get all positions for a user
//! - Filter positions by various criteria
//! - Sort and paginate results
//! - Use convenience methods for common queries
//!
//! Run with:
//! ```bash
//! cargo run --example user_positions --features client
//! ```

use polymarket_rs_sdk::{
    DataClient, DataConfig, PositionSortBy, PositionsQuery, Result, SortDirection,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for better logging
    tracing_subscriber::fmt::init();

    // Create the data client
    let client = DataClient::new(DataConfig::default())?;

    // Example user address (replace with actual address)
    let user_address = "0x0000000000000000000000000000000000000000";

    println!("\n=== Example 1: Get all positions (simple) ===");
    match client.get_positions(user_address).await {
        Ok(positions) => {
            println!("Found {} positions", positions.len());
            for (i, pos) in positions.iter().take(3).enumerate() {
                println!("\nPosition {}:", i + 1);
                println!("  Market: {}", pos.title);
                println!("  Outcome: {}", pos.outcome);
                println!("  Size: {}", pos.size);
                println!("  Avg Price: {:.4}", pos.avg_price);
                println!("  Cash PnL: {:.2}", pos.cash_pnl);
                println!("  Percent PnL: {:.2}%", pos.percent_pnl * 100.0);
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n=== Example 2: Get redeemable positions only ===");
    match client.get_redeemable_positions(user_address).await {
        Ok(positions) => {
            println!("Found {} redeemable positions", positions.len());
            for pos in positions.iter().take(3) {
                println!("  - {} ({}) - Size: {}", pos.title, pos.outcome, pos.size);
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n=== Example 3: Get top profitable positions ===");
    match client.get_top_profitable_positions(user_address, Some(5)).await {
        Ok(positions) => {
            println!("Top 5 most profitable positions:");
            for (i, pos) in positions.iter().enumerate() {
                println!(
                    "  {}. {} ({}) - PnL: ${:.2} ({:.2}%)",
                    i + 1,
                    pos.title,
                    pos.outcome,
                    pos.cash_pnl,
                    pos.percent_pnl * 100.0
                );
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n=== Example 4: Get positions above size threshold ===");
    match client.get_positions_above_size(user_address, 100.0).await {
        Ok(positions) => {
            println!("Positions with size > 100:");
            for pos in positions.iter() {
                println!(
                    "  - {} ({}) - Size: {:.2}",
                    pos.title, pos.outcome, pos.size
                );
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n=== Example 5: Advanced query with custom filters ===");
    let query = PositionsQuery::new(user_address)
        .with_size_threshold(10.0) // Positions with at least 10 tokens
        .with_limit(20) // Return max 20 results
        .sort_by(PositionSortBy::PercentPnl) // Sort by percentage PnL
        .sort_direction(SortDirection::Desc); // Descending order

    match client.get_positions_with_query(&query).await {
        Ok(positions) => {
            println!("Filtered positions (size > 10, sorted by % PnL):");
            for pos in positions.iter().take(5) {
                println!(
                    "  - {} ({}) - Size: {:.2}, %PnL: {:.2}%",
                    pos.title,
                    pos.outcome,
                    pos.size,
                    pos.percent_pnl * 100.0
                );
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n=== Example 6: Get positions for specific markets ===");
    let market_ids = vec![
        "0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917".to_string(),
        // Add more market IDs as needed
    ];

    match client
        .get_positions_for_markets(user_address, market_ids)
        .await
    {
        Ok(positions) => {
            println!("Positions in specified markets:");
            for pos in positions.iter() {
                println!(
                    "  - {} ({}) - Current: ${:.2}",
                    pos.title, pos.outcome, pos.current_value
                );
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n=== Example 7: Pagination example ===");
    let page_size = 10;
    let page_number = 0;

    let query = PositionsQuery::new(user_address)
        .with_limit(page_size)
        .with_offset(page_number * page_size);

    match client.get_positions_with_query(&query).await {
        Ok(positions) => {
            println!("Page {} (showing {} positions):", page_number + 1, positions.len());
            for (i, pos) in positions.iter().enumerate() {
                println!(
                    "  {}. {} ({}) - Value: ${:.2}",
                    i + 1,
                    pos.title,
                    pos.outcome,
                    pos.current_value
                );
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n=== Example 8: Search positions by title ===");
    let query = PositionsQuery::new(user_address)
        .with_title("Trump") // Filter positions containing "Trump" in title
        .sort_by(PositionSortBy::Current)
        .sort_direction(SortDirection::Desc);

    match client.get_positions_with_query(&query).await {
        Ok(positions) => {
            println!("Positions matching title 'Trump':");
            for pos in positions.iter() {
                println!(
                    "  - {} ({}) - Current: ${:.2}",
                    pos.title, pos.outcome, pos.current_value
                );
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n=== Example 9: Display detailed position information ===");
    match client.get_positions(user_address).await {
        Ok(positions) => {
            if let Some(pos) = positions.first() {
                println!("\nDetailed position info:");
                println!("  Proxy Wallet: {}", pos.proxy_wallet);
                println!("  Market: {}", pos.title);
                println!("  Slug: {}", pos.slug);
                println!("  Event: {}", pos.event_slug);
                println!("  Condition ID: {}", pos.condition_id);
                println!("  Asset ID: {}", pos.asset);
                println!("  Outcome: {} (index: {})", pos.outcome, pos.outcome_index);
                println!("  Opposite: {}", pos.opposite_outcome);
                println!("  Size: {} tokens", pos.size);
                println!("  Average Price: ${:.4}", pos.avg_price);
                println!("  Current Price: ${:.4}", pos.cur_price);
                println!("  Initial Value: ${:.2}", pos.initial_value);
                println!("  Current Value: ${:.2}", pos.current_value);
                println!("  Total Bought: ${:.2}", pos.total_bought);
                println!("  Cash PnL: ${:.2}", pos.cash_pnl);
                println!("  Percent PnL: {:.2}%", pos.percent_pnl * 100.0);
                println!("  Realized PnL: ${:.2}", pos.realized_pnl);
                println!("  Percent Realized PnL: {:.2}%", pos.percent_realized_pnl * 100.0);
                println!("  Redeemable: {}", pos.redeemable);
                println!("  Mergeable: {}", pos.mergeable);
                println!("  Negative Risk: {}", pos.negative_risk);
                println!("  End Date: {}", pos.end_date);
                if let Some(icon) = &pos.icon {
                    println!("  Icon: {}", icon);
                }
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    Ok(())
}

