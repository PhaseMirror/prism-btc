//! Long-running mining orchestrator: extranonce rolling, tip-staleness
//! cancellation, hash-rate progress reporting.
//!
//! Sits one layer above [`crate::PrismMiner::mine_one_block`]. Where
//! `mine_one_block` is "fetch a template, run prism_btc::mine once",
//! a [`MiningSession`] runs until it either mines a block or the user
//! stops it — refreshing the template on every tip change, bumping
//! the coinbase extranonce when [`prism_btc::mine`] returns
//! [`prism_btc::MiningFailure::NoMatch`], and emitting progress events.
//!
//! **The mining algorithm is `prism_btc::mine_parallel`, not anything
//! in this module.** This module is bitcoind RPC plumbing.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use bitcoin::{Address, BlockHash as BtcBlockHash, Network};
use bitcoincore_rpc::json::{
    GetBlockTemplateCapabilities, GetBlockTemplateModes, GetBlockTemplateResult,
    GetBlockTemplateRules,
};
use bitcoincore_rpc::{Auth, Client, RpcApi};

use prism_btc::{mine_parallel, Cancel, MiningFailure, MiningWitness};

use crate::MiningJob;

/// Configuration knobs for a mining session.
pub struct SessionConfig {
    /// Coset count for `mine_parallel`'s W32 partition. `None` defaults
    /// to `std::thread::available_parallelism()`.
    pub threads: Option<usize>,
    /// How often the tip watcher polls bitcoind for `getbestblockhash`.
    pub tip_poll: Duration,
    /// How often a progress event is emitted while a search is in flight.
    pub progress_every: Duration,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            threads: None,
            tip_poll: Duration::from_millis(500),
            progress_every: Duration::from_secs(5),
        }
    }
}

/// A long-running mining orchestrator.
pub struct MiningSession {
    rpc_url: String,
    auth: Auth,
    payout_address: Address,
    network: Network,
    cfg: SessionConfig,
}

impl MiningSession {
    pub fn new(
        rpc_url: &str,
        auth: Auth,
        payout_address: &str,
        network: Network,
        cfg: SessionConfig,
    ) -> Result<Self> {
        let client = Client::new(rpc_url, auth.clone()).context("RPC client connect")?;
        let info = client
            .get_blockchain_info()
            .context("getblockchaininfo for chain-mismatch guard")?;
        if info.chain != network {
            bail!(
                "chain-mismatch guard: node is on {:?} but --network is {network:?}",
                info.chain
            );
        }
        let address = payout_address
            .parse::<Address<_>>()
            .context("parse payout address")?
            .require_network(network)
            .with_context(|| format!("payout address is not for network {network:?}"))?;
        Ok(Self {
            rpc_url: rpc_url.to_string(),
            auth,
            payout_address: address,
            network,
            cfg,
        })
    }

    /// Run the session until a block is mined and accepted, or until
    /// `user_cancel` is set.
    pub fn mine_until_block(&self, user_cancel: Arc<AtomicBool>) -> Result<MinedBlock> {
        let main_client = Client::new(&self.rpc_url, self.auth.clone())?;

        loop {
            if user_cancel.load(Ordering::Relaxed) {
                bail!("user cancelled");
            }

            let template = self.fetch_template(&main_client)?;
            let initial_tip = template.previous_block_hash;
            eprintln!(
                "session: new template at height {} (prev {}, target_bits {:08x})",
                template.height,
                initial_tip,
                u32::from_be_bytes([
                    template.bits[0],
                    template.bits[1],
                    template.bits[2],
                    template.bits[3]
                ])
            );

            // Spawn a tip-watcher; it sets `tip_changed` on chain advance.
            let tip_changed = Arc::new(AtomicBool::new(false));
            let watcher =
                self.spawn_tip_watcher(initial_tip, tip_changed.clone(), user_cancel.clone());

            let mut extranonce: u64 = 0;
            let outcome = loop {
                if tip_changed.load(Ordering::Relaxed) || user_cancel.load(Ordering::Relaxed) {
                    break InnerOutcome::CancelledOrStale;
                }

                let job = MiningJob::from_template_with_extranonce(
                    &template,
                    &self.payout_address,
                    extranonce,
                )?;

                let attempt_started = Instant::now();
                let header = job.header.clone();
                let target = job.target;

                // The cancel adapter bridges (tip_changed | user_cancel)
                // into prism-btc's Cancel trait.
                let cancel = SessionCancel {
                    tip_changed: tip_changed.clone(),
                    user_cancel: user_cancel.clone(),
                };

                let threads = self.cfg.threads.unwrap_or_else(|| {
                    std::thread::available_parallelism()
                        .map(|n| n.get())
                        .unwrap_or(1)
                });

                let result = mine_parallel(&header, target, threads, &cancel);

                let elapsed = attempt_started.elapsed();
                eprintln!(
                    "session: extranonce={} traversal completed in {:?}",
                    extranonce, elapsed
                );

                match result {
                    Ok(outcome) => break InnerOutcome::Found(Box::new((job, outcome))),
                    Err(MiningFailure::NoMatch) => {
                        extranonce += 1;
                        eprintln!(
                            "session: extranonce {} exhausted, bumping to {}",
                            extranonce - 1,
                            extranonce
                        );
                    }
                    Err(MiningFailure::Cancelled) => {
                        break InnerOutcome::CancelledOrStale;
                    }
                }
            };

            watcher.stop();

            match outcome {
                InnerOutcome::Found(boxed) => {
                    let (job, mining_outcome) = *boxed;
                    let block = job.assemble(mining_outcome.nonce);
                    main_client
                        .submit_block(&block)
                        .context("submitblock rejected")?;
                    return Ok(MinedBlock {
                        hash: block.block_hash(),
                        height: template.height,
                        nonce: mining_outcome.nonce,
                        witness: mining_outcome.witness,
                        tx_count: block.txdata.len(),
                    });
                }
                InnerOutcome::CancelledOrStale => {
                    if user_cancel.load(Ordering::Relaxed) {
                        bail!("user cancelled mid-search");
                    }
                    eprintln!("session: tip changed mid-search, refetching template");
                }
            }
        }
    }

    fn fetch_template(&self, client: &Client) -> Result<GetBlockTemplateResult> {
        let rules: &[GetBlockTemplateRules] = match self.network {
            Network::Signet => &[
                GetBlockTemplateRules::SegWit,
                GetBlockTemplateRules::Signet,
                GetBlockTemplateRules::Csv,
                GetBlockTemplateRules::Taproot,
            ],
            _ => &[
                GetBlockTemplateRules::SegWit,
                GetBlockTemplateRules::Csv,
                GetBlockTemplateRules::Taproot,
            ],
        };
        client
            .get_block_template(
                GetBlockTemplateModes::Template,
                rules,
                &[] as &[GetBlockTemplateCapabilities],
            )
            .context("getblocktemplate RPC")
    }

    fn spawn_tip_watcher(
        &self,
        initial_tip: BtcBlockHash,
        tip_changed: Arc<AtomicBool>,
        user_cancel: Arc<AtomicBool>,
    ) -> TipWatcher {
        let url = self.rpc_url.clone();
        let auth = self.auth.clone();
        let poll = self.cfg.tip_poll;
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = stop.clone();
        let join = thread::spawn(move || {
            let watcher_client = match Client::new(&url, auth) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("watcher: failed to open RPC: {e:?}");
                    return;
                }
            };
            while !stop_clone.load(Ordering::Relaxed) && !user_cancel.load(Ordering::Relaxed) {
                match watcher_client.get_best_block_hash() {
                    Ok(tip) => {
                        if tip != initial_tip {
                            tip_changed.store(true, Ordering::Release);
                            return;
                        }
                    }
                    Err(e) => {
                        eprintln!("watcher: getbestblockhash error: {e:?}");
                    }
                }
                thread::sleep(poll);
            }
        });
        TipWatcher {
            stop,
            join: Some(join),
        }
    }

    pub fn network(&self) -> Network {
        self.network
    }
}

struct TipWatcher {
    stop: Arc<AtomicBool>,
    join: Option<thread::JoinHandle<()>>,
}
impl TipWatcher {
    fn stop(mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(h) = self.join.take() {
            let _ = h.join();
        }
    }
}

struct SessionCancel {
    tip_changed: Arc<AtomicBool>,
    user_cancel: Arc<AtomicBool>,
}
impl Cancel for SessionCancel {
    fn is_cancelled(&self) -> bool {
        self.tip_changed.load(Ordering::Relaxed) || self.user_cancel.load(Ordering::Relaxed)
    }
}

enum InnerOutcome {
    Found(Box<(MiningJob, prism_btc::MiningOutcome)>),
    CancelledOrStale,
}

/// Block summary returned to the caller of [`MiningSession::mine_until_block`].
pub struct MinedBlock {
    pub hash: BtcBlockHash,
    pub height: u64,
    pub nonce: u32,
    pub witness: MiningWitness,
    pub tx_count: usize,
}
