# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
