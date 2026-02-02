use clap::{Parser, Subcommand};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

const PROGRAM_ID: &str = "KovA5cAnNeR7xQwK8rY2NjmEv3bUnDL4sHfT9pRs1Wz";
const DEFAULT_RPC: &str = "https://api.mainnet-beta.solana.com";

#[derive(Parser)]
#[command(
    name = "kova",
    about = "Kova -- Solana token survival probability scanner",
    version,
    propagate_version = true
)]
struct Cli {
    /// Solana RPC endpoint URL
    #[arg(long, default_value = DEFAULT_RPC, global = true)]
    rpc_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {