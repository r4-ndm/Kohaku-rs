//! Rust CLI wallet — port target: <https://github.com/kassandraoftroy/kohaku-cli>

use clap::{Parser, Subcommand};
use kohaku_stealth::generate_meta_address;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "kohaku", about = "Community Kohaku Rust CLI (work in progress)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate an ERC-5564 stealth meta-address (Phase 1).
    StealthMeta,
    /// List implemented commands vs upstream TypeScript CLI.
    Status,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    match cli.command {
        Commands::StealthMeta => {
            let meta = generate_meta_address()?;
            println!("stealth meta-address: {}", meta.stealth_meta_address_hex);
            println!("(store spending/viewing keys securely — not printed by default)");
        }
        Commands::Status => {
            println!("kohaku-rs CLI — upstream blueprint: kassandraoftroy/kohaku-cli");
            println!();
            println!("Implemented:");
            println!("  stealth-meta");
            println!();
            println!("Planned (match upstream commands):");
            println!("  create-wallet, list-wallets, balances, next-fresh-address");
            println!("  shield, unshield, see-decrypted-storage");
        }
    }
    Ok(())
}
