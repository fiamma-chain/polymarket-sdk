# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3] - 2025-12-25

### Fixed

- **rustls CryptoProvider** - Fix runtime panic when using realtime WebSocket streams
  - Explicitly specify `ring` as the crypto provider for rustls
  - Resolves "Could not automatically determine the process-level CryptoProvider" error
  - Affects downstream projects using `reqwest` or `tokio-tungstenite` with rustls 0.23+

## [0.1.2] - 2025-12-25

### Added

- **GammaClient** - `get_markets_by_condition_ids()` method for batch querying markets by condition IDs
  - Efficiently query multiple markets in a single API request
  - Supports up to ~50 condition IDs per request
  - 10-50x performance improvement over individual queries
- **GammaClient** - `get_markets_by_condition_ids_batched()` method for large-scale batch queries
  - Automatically batches requests to avoid URL length limits
  - Default batch size of 50 condition IDs per request
  - Ideal for querying 50+ markets
- **GammaClient** - `get_market_by_condition_id()` convenience method
  - Query a single market by its condition ID
  - Returns the first matching market if multiple exist

### Fixed

- **GammaClient** - `get_market()` method now correctly uses market ID (integer) instead of condition_id
  - Breaking change: Parameter type changed from `&str` to `u64` to match [Polymarket API specification](https://docs.polymarket.com/api-reference/markets/get-market-by-id)
  - Previous usage with condition_id was incorrect per official API docs
  - Use `get_market_by_condition_id()` for condition ID queries

### Performance

- Batch market queries: 100 markets now require 2 API calls instead of 100 (50x improvement)

## [0.1.1] - 2025-12-22

### Fixed

- **ClobClient** - Fix L2 authentication address handling in `get_order`, `get_open_orders`, `cancel_orders`, `cancel_all_orders` methods
  - Add `get_auth_address()` helper method to centralize auth address logic
  - Now correctly uses `auth_address` (set via `with_auth_address()`) instead of signer address
  - Fixes Builder API authentication failures in these methods

## [0.1.0] - 2025-12-20

### Changed

- Bump version to 0.1.0 for stable release

## [0.0.5] - 2025-12-10

### Added

- **ClobClient** - `derive_or_create_api_key()` method for automatic wallet registration handling
  - Automatically detects "Could not derive api key" error for unregistered wallets
  - Registers wallet via `create_api_key_with_signature()` and retries derive
  - Simplifies first-time Privy ServerWallet integration
- **PolymarketError** - `is_wallet_not_registered()` helper method to detect CLOB registration errors

## [0.0.4] - 2025-12-09

### Fixed

- **OpenOrder struct** - Correct field mapping to match actual CLOB `/data/orders` API response
- **Orders endpoint** - Use `/data/orders` endpoint for GET requests (not `/orders`)

### Added

- `PaginatedResponse<T>` - Generic wrapper type for paginated CLOB endpoints
- Backward compatible accessor methods on `OpenOrder`: `token_id()`, `maker()`, `signer()`

## [0.0.3] - 2025-12-08

### Changed

- **Config defaults** - Use URL helper functions in Default implementations for cleaner configuration

## [0.0.2] - 2025-12-06

### Added

- Publish script for crates.io releases
- Improved RTDS message parsing robustness with debug logging

### Fixed

- Clippy lints for cleaner code
- Exclude Cargo.lock from published crate (library best practice)

## [0.0.1] - 2025-12-05

### Added

- Initial release
- **Core modules**
  - `core::error` - Unified error handling with retry support
  - `core::endpoints` - API endpoint management
- **Type definitions**
  - `types::common` - Side, ApiCredentials, BookLevel
  - `types::market` - Market, Event, Token
  - `types::order` - OrderOptions, SignedOrderRequest
  - `types::trader` - TraderProfile, Position, Trade
- **Authentication**
  - `auth::eip712` - EIP-712 typed data signing
  - `auth::hmac` - HMAC-SHA256 API key signing
  - `auth::headers` - L1/L2 HTTP header generation
  - `auth::builder` - Builder API authentication (derived from polymarket-rs-sdk)
- **API Clients**
  - `client::gamma` - Market discovery via Gamma API
  - `client::data` - Trader data via Data API
  - `client::profiles` - User profiles API
  - `client::clob` - CLOB REST API for trading
- **Order Management**
  - `order::builder` - Order creation and signing
  - `order::contracts` - Contract configuration
- **WebSocket Streams**
  - `stream::rtds` - Real-time trade data stream
  - `stream::clob` - Order book WebSocket
- **Safe Wallet**
  - `safe::address` - Safe address derivation
  - `safe::eip712` - Safe transaction signing
  - `safe::encoding` - ERC20/ERC1155 data encoding
  - `safe::relayer` - Relayer client for Safe operations

### Attribution

- Builder API signing code derived from [polymarket-rs-sdk](https://github.com/TechieBoy/polymarket-rs-client)
