//! prism-mine — drive prism-btc's σ-convergence loop against a real bitcoind.
//!
//! Example (regtest, against the local node we set up):
//!   prism-mine \
//!     --rpc-url http://127.0.0.1:18443 \
//!     --rpc-user prism --rpc-pass demo \
//!     --network regtest \
//!     --payout bcrt1qz8lf3ukw2atr2vssncqdum5lk6mvgnw5qv6au9 \
//!     --blocks 1

use anyhow::{Context, Result};
use bitcoin::Network;
use bitcoincore_rpc::Auth;
use clap::Parser;

use prism_btc_node::PrismMiner;

#[derive(Parser, Debug)]
#[command(
    name = "prism-mine",
    about = "Mine Bitcoin blocks via prism-btc σ-convergence"
)]
struct Args {
    #[arg(long)]
    rpc_url: String,
    #[arg(long)]
    rpc_user: String,
    #[arg(long)]
    rpc_pass: String,
    #[arg(long, value_parser = parse_network)]
    network: Network,
    #[arg(long)]
    payout: String,
    #[arg(long, default_value_t = 1)]
    blocks: u32,
}

fn parse_network(s: &str) -> Result<Network, String> {
    match s.to_ascii_lowercase().as_str() {
        "mainnet" | "bitcoin" => Ok(Network::Bitcoin),
        "testnet" | "testnet3" => Ok(Network::Testnet),
        "testnet4" => Ok(Network::Testnet4),
        "signet" => Ok(Network::Signet),
        "regtest" => Ok(Network::Regtest),
        other => Err(format!("unknown network: {other}")),
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let auth = Auth::UserPass(args.rpc_user.clone(), args.rpc_pass.clone());
    let miner = PrismMiner::connect(&args.rpc_url, auth, &args.payout, args.network)
        .context("PrismMiner::connect")?;

    println!(
        "prism-mine: connected to {} on {:?}; payout {}",
        args.rpc_url, args.network, args.payout
    );

    for i in 1..=args.blocks {
        let started = std::time::Instant::now();
        let mined = miner.mine_one_block().context("mine_one_block")?;
        let dt = started.elapsed();
        println!(
            "[{i}/{}] mined block #{} hash={} nonce={} txs={} ({:?})",
            args.blocks, mined.height, mined.hash, mined.nonce, mined.tx_count, dt
        );
    }

    Ok(())
}
