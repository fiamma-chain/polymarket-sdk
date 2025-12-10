# polymarket-sdk

[![Crates.io](https://img.shields.io/crates/v/polymarket-sdk.svg)](https://crates.io/crates/polymarket-sdk)
[![Documentation](https://docs.rs/polymarket-sdk/badge.svg)](https://docs.rs/polymarket-sdk)
[![License](https://img.shields.io/crates/l/polymarket-sdk.svg)](LICENSE-MIT)

A comprehensive Rust SDK for [Polymarket](https://polymarket.com) prediction markets.

## Features

- **Market Discovery** - Query markets, events, and metadata via Gamma API
- **Real-time Data** - WebSocket streams for trades and order book updates
- **Order Management** - Create, sign, and submit orders to CLOB
- **Authentication** - EIP-712 signing and HMAC-based API authentication
- **Safe Wallet** - Deploy and manage Gnosis Safe proxy wallets

## Installation

```toml
[dependencies]
polymarket-sdk = "0.0.5"
```

### Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `full` | All features enabled | Yes |
| `client` | REST API clients (Gamma, Data, CLOB) | Yes |
| `stream` | WebSocket streaming (RTDS, Order Book) | Yes |
| `order` | Order creation & signing | Yes |
| `safe` | Safe wallet integration | Yes |

## Quick Start

```rust
use polymarket_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Market discovery (no auth required)
    let client = GammaClient::new(Default::default())?;
    let markets = client.get_markets(None).await?;

    println!("Found {} markets", markets.len());

    for market in markets.iter().take(5) {
        println!("  - {}: {}", market.condition_id, market.question);
    }

    Ok(())
}
```

## Authentication

Polymarket uses two levels of authentication:

### L1 Authentication (Wallet Signature)

Used to create API credentials. Requires EIP-712 signing with your wallet.

```rust
use polymarket_sdk::auth::create_l1_headers;
use alloy_signer_local::PrivateKeySigner;

let signer: PrivateKeySigner = "0x...".parse()?;
let headers = create_l1_headers(&signer, None)?;
```

### L2 Authentication (API Key)

Used for daily API operations. Uses HMAC-SHA256 signing.

```rust
use polymarket_sdk::auth::create_l2_headers;

let headers = create_l2_headers(&signer, &credentials, "GET", "/orders", None)?;
```

## Trading Example

```rust
use polymarket_sdk::prelude::*;
use alloy_signer_local::PrivateKeySigner;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup signer
    let signer: PrivateKeySigner = std::env::var("PRIVATE_KEY")?.parse()?;

    // Create order builder
    let builder = OrderBuilder::new(signer.clone(), Some(SigType::PolyProxy), None);

    // Build order
    let order_args = OrderArgs {
        token_id: "0x...".to_string(),
        side: Side::Buy,
        price: "0.55".parse()?,
        size: "10.0".parse()?,
    };

    let options = OrderOptions {
        tick_size: "0.01".to_string(),
        neg_risk: false,
    };

    let signed_order = builder.create_order(order_args, &options).await?;

    // Submit to CLOB
    let clob = ClobClient::new(ClobConfig::default(), signer, credentials)?;
    let response = clob.submit_order(&signed_order).await?;

    println!("Order submitted: {}", response.order_id);
    Ok(())
}
```

## WebSocket Streaming

```rust
use polymarket_sdk::stream::{RtdsClient, RtdsConfig, RtdsEvent};

#[tokio::main]
async fn main() -> Result<()> {
    let config = RtdsConfig::default();
    let mut client = RtdsClient::new(config);
    let mut rx = client.connect().await?;

    // Subscribe to markets
    client.subscribe(vec!["market_id_1", "market_id_2"]).await?;

    while let Some(event) = rx.recv().await {
        match event {
            RtdsEvent::Trade(trade) => {
                println!("Trade: {} @ {}", trade.size, trade.price);
            }
            RtdsEvent::Error(e) => {
                eprintln!("Error: {}", e);
            }
            _ => {}
        }
    }

    Ok(())
}
```

## Safe Wallet

```rust
use polymarket_sdk::safe::{derive_safe_address, RelayerClient};

// Derive Safe address from EOA
let safe_address = derive_safe_address("0xYourEOAAddress")?;
println!("Safe address: {}", safe_address);

// Deploy Safe via Relayer
let relayer = RelayerClient::from_env()?;
let result = relayer.deploy_safe(&owner_address, &signature).await?;
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
