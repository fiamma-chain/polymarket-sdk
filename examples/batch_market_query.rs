//! Batch Market Query Example
//!
//! This example demonstrates how to query multiple markets by condition IDs
//! in a single batch request, which is more efficient than querying them individually.
//!
//! Run with: `cargo run --example batch_market_query --features client`

use polymarket_rs_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the Gamma API client (no auth required for public data)
    let client = GammaClient::new(GammaConfig::default())?;

    println!("=== Polymarket Batch Market Query Example ===\n");

    // Example 1: Query multiple markets by condition IDs
    println!("1. Fetching markets by condition IDs...");
    
    // These are example condition IDs - replace with actual condition IDs from Polymarket
    let condition_ids = vec![
        // You can get these from the Gamma API or market URLs
        // Format: "0x..." (hex string)
    ];

    if condition_ids.is_empty() {
        println!("   No condition IDs provided - demonstrating with empty list");
        let markets = client.get_markets_by_condition_ids(&[]).await?;
        println!("   Result: {} markets (expected 0 for empty input)", markets.len());
    } else {
        let markets = client.get_markets_by_condition_ids(&condition_ids).await?;
        println!("   Found {} markets:", markets.len());
        
        for market in &markets {
            println!("   - Condition ID: {}", market.condition_id);
            println!("     Question: {}", market.question.as_deref().unwrap_or("Unknown"));
            println!("     Active: {}, Closed: {}", market.active, market.closed);
            
            // Show outcome prices
            let (yes_price, no_price) = market.parse_outcome_prices();
            if let Some(yes) = yes_price {
                println!(
                    "     Prices: Yes @ ${:.2}, No @ ${:.2}",
                    yes,
                    no_price.unwrap_or(1.0 - yes)
                );
            }
            println!();
        }
    }

    // Example 2: Batch query with large number of condition IDs
    println!("\n2. Demonstrating batched query for large lists...");
    println!("   (Using batching to avoid URL length limits)");
    
    // For demonstration - in real usage, you'd have actual condition IDs
    let large_condition_list: Vec<String> = vec![];
    
    if large_condition_list.is_empty() {
        println!("   No condition IDs provided - skipping batched query demo");
    } else {
        // Use batched version for large lists (default batch size: 50)
        let markets = client
            .get_markets_by_condition_ids_batched(&large_condition_list, Some(50))
            .await?;
        println!("   Retrieved {} markets in batches", markets.len());
    }

    // Example 3: Compare with individual queries
    println!("\n3. Comparing batch vs individual queries...");
    println!("   Batch query: Single API call for multiple condition IDs");
    println!("   Individual queries: One API call per condition ID");
    println!("   Recommendation: Use batch queries for better performance!");

    // Example 4: Get a single market first, then batch query related markets
    println!("\n4. Practical workflow example...");
    
    // First, get some active markets
    let params = ListParams::new().with_limit(3).with_active(true);
    let sample_markets = client.get_markets(Some(params)).await?;
    
    if !sample_markets.is_empty() {
        println!("   Found {} sample markets", sample_markets.len());
        
        // Extract their condition IDs
        let condition_ids: Vec<String> = sample_markets
            .iter()
            .map(|m| m.condition_id.clone())
            .collect();
        
        println!("   Condition IDs: {:?}", condition_ids);
        
        // Now batch query them (in real usage, you might have these IDs from another source)
        let queried_markets = client.get_markets_by_condition_ids(&condition_ids).await?;
        println!("   Successfully batch queried {} markets", queried_markets.len());
        
        for market in &queried_markets {
            println!(
                "   - {} ({})",
                market.question.as_deref().unwrap_or("Unknown"),
                market.condition_id
            );
        }
    } else {
        println!("   No active markets found for demonstration");
    }

    println!("\n=== Done! ===");
    println!("\nUsage tips:");
    println!("- Use get_markets_by_condition_ids() for up to ~50 condition IDs");
    println!("- Use get_markets_by_condition_ids_batched() for larger lists");
    println!("- Batch queries are more efficient than individual get_market() calls");
    
    Ok(())
}

