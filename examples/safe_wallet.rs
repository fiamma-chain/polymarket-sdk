//! Safe Wallet (Proxy Wallet) Example
//!
//! This example demonstrates how to work with Polymarket's Gnosis Safe proxy wallets.
//! These are the wallets that hold your funds on Polymarket.
//!
//! Environment variables (optional):
//! - `POLY_BUILDER_API_KEY`: Builder API key for relayer authentication
//! - `POLY_BUILDER_SECRET`: Builder API secret (base64 encoded)
//! - `POLY_BUILDER_PASSPHRASE`: Builder API passphrase
//!
//! Run with: `cargo run --example safe_wallet --features safe`

use polymarket_rs_sdk::prelude::*;
use polymarket_rs_sdk::relayer::BuilderApiCredentials;
use polymarket_rs_sdk::safe::{
    build_safe_create_typed_data, derive_safe_address, CONDITIONAL_TOKENS_ADDRESS,
    CTF_EXCHANGE_ADDRESS, EXCHANGE_ADDRESS, NATIVE_USDC_CONTRACT_ADDRESS,
    NEG_RISK_CTF_EXCHANGE_ADDRESS, SAFE_FACTORY, SAFE_INIT_CODE_HASH, USDC_CONTRACT_ADDRESS,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    println!("=== Polymarket Safe Wallet Example ===\n");

    // 1. Initialize the Relayer client
    println!("1. Initializing Relayer client...");
    let mut _relayer = RelayerClient::with_defaults()?;

    // Load Builder API credentials for authenticated operations
    match BuilderApiCredentials::from_env() {
        Ok(creds) => {
            _relayer = _relayer.with_builder_credentials(creds);
            println!("   Builder API credentials loaded");
        }
        Err(_) => {
            println!("   Builder API credentials not set (optional)");
        }
    }
    println!("   Relayer client initialized");

    // 2. Compute the deterministic Safe address for a signer
    println!("\n2. Computing Safe wallet address...");
    let example_signer = "0x1234567890123456789012345678901234567890";

    // The Safe address is deterministically derived using CREATE2
    match derive_safe_address(example_signer) {
        Ok(safe_address) => {
            println!("   Signer (EOA): {}", example_signer);
            println!("   Derived Safe: {}", safe_address);
        }
        Err(e) => {
            println!("   Could not derive Safe address: {}", e);
        }
    }
    println!();
    println!("   Note: The Safe address is computed using CREATE2 with:");
    println!("   - Factory: {}", SAFE_FACTORY);
    println!("   - Init code hash: {}...", &SAFE_INIT_CODE_HASH[..20]);

    // 3. Check if the Safe is deployed
    println!("\n3. Checking Safe deployment status...");
    println!("   (Requires RPC connection - skipped in example)");
    println!();
    println!("   To check deployment status:");
    println!("   ```");
    println!("   let rpc_url = \"https://polygon-rpc.com\";");
    println!("   relayer = relayer.with_default_rpc(rpc_url);");
    println!("   let deployed = relayer.check_proxy_deployed(&signer_address).await?;");
    println!("   ```");

    // 4. Deploy a new Safe (if not already deployed)
    println!("\n4. Safe deployment flow...");
    println!("   To deploy a new Safe wallet:");
    println!();

    // Build the typed data for signing
    let typed_data = build_safe_create_typed_data(example_signer, None)?;
    println!("   Typed data domain: {:?}", typed_data.domain.name);

    println!();
    println!("   1. Build the deployment typed data:");
    println!("   ```");
    println!("   let typed_data = build_safe_create_typed_data(&signer_address);");
    println!("   ```");
    println!();
    println!("   2. Sign with your signer wallet (e.g., via Privy):");
    println!("   ```");
    println!("   let signature = signer.sign_typed_data(&typed_data).await?;");
    println!("   ```");
    println!();
    println!("   3. Submit deployment request:");
    println!("   ```");
    println!("   let result = relayer.deploy_safe(&signer_address, &signature).await?;");
    println!("   println!(\"Safe deployed at: {{}}\", result.proxy_wallet);");
    println!("   ```");

    // 5. Token approvals for trading
    println!("\n5. Setting up token approvals...");
    println!("   Before trading, you need to approve the exchange contracts:");
    println!();
    println!("   USDC approval (for placing orders):");
    println!("   ```");
    println!("   let approve_data = build_token_approve_typed_data(");
    println!("       &safe_address,");
    println!("       USDC_CONTRACT_ADDRESS,");
    println!("       EXCHANGE_ADDRESS,");
    println!("       u128::MAX, // Unlimited approval");
    println!("   );");
    println!("   ```");
    println!();
    println!("   CTF approval (for trading outcome tokens):");
    println!("   ```");
    println!("   let ctf_approve_data = build_ctf_approve_typed_data(");
    println!("       &safe_address,");
    println!("       CTF_EXCHANGE_ADDRESS,");
    println!("       true, // Set approval");
    println!("   );");
    println!("   ```");

    // 6. Transferring funds
    println!("\n6. USDC transfer example...");
    println!("   To transfer USDC from Safe to another address:");
    println!("   ```");
    println!("   let transfer_data = build_usdc_transfer_typed_data(");
    println!("       &safe_address,");
    println!("       \"0xRecipientAddress...\",");
    println!("       1_000_000, // 1 USDC (6 decimals)");
    println!("   );");
    println!("   let signature = signer.sign_typed_data(&transfer_data).await?;");
    println!("   let tx = relayer.submit_safe_transaction(");
    println!("       &safe_address,");
    println!("       &transfer_data,");
    println!("       &signature,");
    println!("   ).await?;");
    println!("   ```");

    // 7. Contract addresses reference
    println!("\n7. Contract addresses reference:");
    println!("   USDC (Bridged):     {}", USDC_CONTRACT_ADDRESS);
    println!("   USDC (Native):      {}", NATIVE_USDC_CONTRACT_ADDRESS);
    println!("   Exchange:           {}", EXCHANGE_ADDRESS);
    println!("   CTF Exchange:       {}", CTF_EXCHANGE_ADDRESS);
    println!("   Neg Risk Exchange:  {}", NEG_RISK_CTF_EXCHANGE_ADDRESS);
    println!("   Conditional Tokens: {}", CONDITIONAL_TOKENS_ADDRESS);
    println!("   Safe Factory:       {}", SAFE_FACTORY);

    println!("\n=== Safe Wallet Example Complete ===");
    println!("\nThis example demonstrates the Safe wallet concepts.");
    println!("For production use, integrate with a signing service like Privy.");

    Ok(())
}
