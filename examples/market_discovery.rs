//! Market Discovery Example
//!
//! This example demonstrates how to query markets and events from Polymarket
//! without any authentication. Perfect for building market explorers or dashboards.
//!
//! Run with: `cargo run --example market_discovery --features client`

use polymarket_rs_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the Gamma API client (no auth required for public data)
    let client = GammaClient::new(GammaConfig::default())?;

    println!("=== Polymarket Market Discovery ===\n");

    // 1. Fetch active markets
    println!("1. Fetching active markets...");
    let params = ListParams::new().with_limit(5);
    let markets = client.get_markets(Some(params)).await?;

    println!("Found {} markets:", markets.len());
    for market in &markets {
        println!(
            "  - {} (Volume: ${:.2})",
            market.question.as_deref().unwrap_or("Unknown"),
            market
                .volume_num
                .map(|v| v.to_string().parse::<f64>().unwrap_or(0.0))
                .unwrap_or(0.0)
        );
        // Show parsed outcome prices using helper method
        let (yes_price, no_price) = market.parse_outcome_prices();
        if let Some(yes) = yes_price {
            println!(
                "    Yes @ ${:.2}, No @ ${:.2}",
                yes,
                no_price.unwrap_or(1.0 - yes)
            );
        }
    }

    // 2. Fetch events
    println!("\n2. Fetching events...");
    let params = ListParams::new().with_limit(3);
    let events = client.get_events(Some(params)).await?;

    println!("Found {} events:", events.len());
    for event in &events {
        println!(
            "  - {} ({} markets)",
            event.name.as_deref().unwrap_or("Unknown"),
            event.markets.len()
        );
    }

    // 3. Search for specific topics
    println!("\n3. Searching for 'election' markets...");
    let search_results = client.search_all("election", 10).await?;

    println!(
        "Found {} events matching 'election'",
        search_results.events.len()
    );
    for event in search_results.events.iter().take(3) {
        println!("  - {}", event.question.as_deref().unwrap_or("Unknown"));
    }

    // 4. Fetch tags/categories
    println!("\n4. Fetching available tags...");
    let tags = client.get_tags().await?;
    println!("Available categories:");
    for tag in tags.iter().take(10) {
        println!(
            "  - {} ({})",
            tag.name.as_deref().unwrap_or("?"),
            tag.slug.as_deref().unwrap_or("?")
        );
    }

    println!("\n=== Done! ===");
    Ok(())
}
