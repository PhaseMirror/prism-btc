//! Long-running mining orchestrator: extranonce rolling, tip-staleness
//! cancellation, parallel σ-convergence, and hash-rate reporting.
//!
//! This sits one layer above [`crate::PrismMiner::mine_one_block`]. Where
//! `mine_one_block` is "fetch a template, search 2^32 nonces once", a
//! [`MiningSession`] runs until it either mines a block or the user stops
//! it — refreshing the template on every tip change, bumping the coinbase
//! extranonce when the inner search exhausts, and emitting periodic
//! progress events.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use bitcoin::absolute::LockTime;
use bitcoin::block::{Header as BtcHeader, Version as BtcBlockVersion};
use bitcoin::blockdata::transaction::{OutPoint, TxIn, TxOut};
use bitcoin::hashes::Hash;
use bitcoin::merkle_tree::calculate_root;
use bitcoin::pow::CompactTarget;
use bitcoin::script::{Builder as ScriptBuilder, ScriptBuf};
use bitcoin::{
    Address, Amount, Block, BlockHash as BtcBlockHash, Network, Sequence, Transaction, Witness,
};
use bitcoincore_rpc::json::{
    GetBlockTemplateCapabilities, GetBlockTemplateModes, GetBlockTemplateResult,
    GetBlockTemplateRules,
};
use bitcoincore_rpc::{Auth, Client, RpcApi};

use prism_btc_reduction::{
    run_convergence_parallel, serialize_header, sha256d, BlockCertificate, NoMatch,
};
use prism_btc_types::{Bits, BlockHeader, MerkleRoot, Target, Timestamp, Version};
use uor_foundation::enforcement::DigestProjectionMap;

const COINBASE_WITNESS_RESERVED: [u8; 32] = [0u8; 32];

/// Configuration knobs for a mining session.
pub struct SessionConfig {
    /// Number of cosets to partition the W32 nonce ring across.
    /// `None` → use `rayon::current_num_threads()`.
    pub threads: Option<usize>,
    /// How often the tip watcher polls the node for `getbestblockhash`.
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
        // Chain-mismatch guard: refuse to mine if the node says it's on a
        // different chain than the caller declared.
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

    /// Run the mining session until a block is mined and accepted, or until
    /// `user_cancel` is set. On success returns the accepted block summary.
    pub fn mine_until_block(&self, user_cancel: Arc<AtomicBool>) -> Result<MinedBlock> {
        let main_client = Client::new(&self.rpc_url, self.auth.clone())?;

        loop {
            if user_cancel.load(Ordering::Relaxed) {
                bail!("user cancelled");
            }

            // 1. Fetch current template.
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

            // 2. Spawn a tip-watcher that fires `tip_changed` on chain advance.
            let tip_changed = Arc::new(AtomicBool::new(false));
            let watcher =
                self.spawn_tip_watcher(initial_tip, tip_changed.clone(), user_cancel.clone());

            // 3. Outer extranonce loop: bump the coinbase extranonce, rebuild
            // merkle, run inner search, repeat until found / tip changed / cancelled.
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

                // Spawn a progress reporter that polls a shared hash counter.
                let hashes = Arc::new(AtomicU64::new(0));
                let progress_stop = Arc::new(AtomicBool::new(false));
                let progress_join =
                    self.spawn_progress(hashes.clone(), progress_stop.clone(), extranonce);

                // Inner cancel = OR(tip_changed, user_cancel)
                let inner_cancel = Arc::new(AtomicBool::new(false));
                let cancel_bridge = {
                    let inner = inner_cancel.clone();
                    let tip = tip_changed.clone();
                    let user = user_cancel.clone();
                    let stop = progress_stop.clone();
                    thread::spawn(move || loop {
                        if stop.load(Ordering::Relaxed) {
                            return;
                        }
                        if tip.load(Ordering::Relaxed) || user.load(Ordering::Relaxed) {
                            inner.store(true, Ordering::Release);
                            return;
                        }
                        thread::sleep(Duration::from_millis(50));
                    })
                };

                let header = job.header.clone();
                let target = job.target;
                let header_for_closure = job.header.clone();
                let hashes_for_closure = hashes.clone();
                let convergence_result = run_convergence_parallel::<DigestProjectionMap, _>(
                    header,
                    target.to_bytes(),
                    move |nonce| {
                        hashes_for_closure.fetch_add(1, Ordering::Relaxed);
                        sha256d(&serialize_header(&header_for_closure, nonce))
                    },
                    &inner_cancel,
                    self.cfg.threads,
                );

                // Stop progress + cancel bridge.
                progress_stop.store(true, Ordering::Relaxed);
                let _ = cancel_bridge.join();
                let _ = progress_join.join();

                match convergence_result {
                    Ok(cert) => {
                        break InnerOutcome::Found(Box::new((job, cert)));
                    }
                    Err(NoMatch::Exhausted) => {
                        // 2^32 nonces tried for this extranonce; bump and continue.
                        extranonce += 1;
                        eprintln!(
                            "session: extranonce {} exhausted, bumping to {}",
                            extranonce - 1,
                            extranonce
                        );
                    }
                    Err(NoMatch::Cancelled) => {
                        break InnerOutcome::CancelledOrStale;
                    }
                }
            };

            // 4. Stop the tip watcher.
            watcher.stop();

            match outcome {
                InnerOutcome::Found(boxed) => {
                    let (job, cert) = *boxed;
                    // Splice the winning nonce back in and submit.
                    let wire80 = cert.encode_wire();
                    let nonce =
                        u32::from_le_bytes([wire80[76], wire80[77], wire80[78], wire80[79]]);
                    let block = job.assemble(nonce);
                    main_client
                        .submit_block(&block)
                        .context("submitblock rejected")?;
                    return Ok(MinedBlock {
                        hash: block.block_hash(),
                        height: template.height,
                        nonce,
                        cert,
                        tx_count: block.txdata.len(),
                    });
                }
                InnerOutcome::CancelledOrStale => {
                    if user_cancel.load(Ordering::Relaxed) {
                        bail!("user cancelled mid-search");
                    }
                    // Otherwise: tip changed; loop and refetch template.
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
            // Independent client so we don't contend with the main thread's RPC.
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

    fn spawn_progress(
        &self,
        hashes: Arc<AtomicU64>,
        stop: Arc<AtomicBool>,
        extranonce: u64,
    ) -> thread::JoinHandle<()> {
        let every = self.cfg.progress_every;
        thread::spawn(move || {
            let started = Instant::now();
            let mut last_tick = started;
            let mut last_count: u64 = 0;
            while !stop.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(100));
                let now = Instant::now();
                if now.duration_since(last_tick) >= every {
                    let count = hashes.load(Ordering::Relaxed);
                    let dt = now.duration_since(last_tick).as_secs_f64();
                    let total_dt = now.duration_since(started).as_secs_f64();
                    let rate = (count - last_count) as f64 / dt;
                    let total_rate = count as f64 / total_dt;
                    eprintln!(
                        "session: extranonce={} hashes={} ({:.2} MH/s instant, {:.2} MH/s avg)",
                        extranonce,
                        count,
                        rate / 1.0e6,
                        total_rate / 1.0e6,
                    );
                    last_tick = now;
                    last_count = count;
                }
            }
        })
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

enum InnerOutcome {
    Found(Box<(MiningJob, BlockCertificate<DigestProjectionMap>)>),
    CancelledOrStale,
}

/// Block summary returned to the caller of [`MiningSession::mine_until_block`].
pub struct MinedBlock {
    pub hash: BtcBlockHash,
    pub height: u64,
    pub nonce: u32,
    pub cert: BlockCertificate<DigestProjectionMap>,
    pub tx_count: usize,
}

// ----- Internal: extranonce-aware MiningJob -----

struct MiningJob {
    header: BlockHeader,
    target: Target,
    coinbase: Transaction,
    other_txs: Vec<Transaction>,
    btc_version: BtcBlockVersion,
    btc_prev_hash: BtcBlockHash,
    btc_merkle: bitcoin::TxMerkleNode,
    btc_time: u32,
    btc_bits: CompactTarget,
}

impl MiningJob {
    fn from_template_with_extranonce(
        tpl: &GetBlockTemplateResult,
        payout: &Address,
        extranonce: u64,
    ) -> Result<Self> {
        let other_txs: Vec<Transaction> = tpl
            .transactions
            .iter()
            .map(|t| t.transaction().context("decode template tx"))
            .collect::<Result<_>>()?;

        let coinbase = build_coinbase_tx(tpl, payout, extranonce)?;

        let mut leaves = Vec::with_capacity(other_txs.len() + 1);
        leaves.push(coinbase.compute_txid().to_raw_hash());
        for tx in &other_txs {
            leaves.push(tx.compute_txid().to_raw_hash());
        }
        let merkle_root_raw = calculate_root(leaves.into_iter())
            .context("merkle root computation (empty leaf set unexpected)")?;
        let btc_merkle = bitcoin::TxMerkleNode::from_raw_hash(merkle_root_raw);

        if tpl.bits.len() != 4 {
            bail!(
                "getblocktemplate.bits has unexpected length {}",
                tpl.bits.len()
            );
        }
        let bits_u32 = u32::from_be_bytes([tpl.bits[0], tpl.bits[1], tpl.bits[2], tpl.bits[3]]);
        let btc_bits = CompactTarget::from_consensus(bits_u32);

        let time_u32: u32 = tpl.current_time.try_into().with_context(|| {
            format!(
                "template current_time {} exceeds u32::MAX",
                tpl.current_time
            )
        })?;
        let version_i32: i32 = tpl
            .version
            .try_into()
            .with_context(|| format!("template version {} exceeds i32::MAX", tpl.version))?;

        let prev_bytes: [u8; 32] = tpl.previous_block_hash.to_byte_array();
        let merkle_bytes: [u8; 32] = merkle_root_raw.to_byte_array();

        let header = BlockHeader {
            version: Version(tpl.version),
            prev_hash: prev_bytes,
            merkle_root: MerkleRoot::from_bytes(merkle_bytes),
            timestamp: Timestamp(time_u32),
            bits: Bits(bits_u32),
        };
        let target = Target::new(bits_u32);

        Ok(Self {
            header,
            target,
            coinbase,
            other_txs,
            btc_version: BtcBlockVersion::from_consensus(version_i32),
            btc_prev_hash: tpl.previous_block_hash,
            btc_merkle,
            btc_time: time_u32,
            btc_bits,
        })
    }

    fn assemble(self, nonce: u32) -> Block {
        let header = BtcHeader {
            version: self.btc_version,
            prev_blockhash: self.btc_prev_hash,
            merkle_root: self.btc_merkle,
            time: self.btc_time,
            bits: self.btc_bits,
            nonce,
        };
        let mut txdata = Vec::with_capacity(self.other_txs.len() + 1);
        txdata.push(self.coinbase);
        txdata.extend(self.other_txs);
        Block { header, txdata }
    }
}

/// BIP141-compliant coinbase transaction with an extranonce in scriptSig.
fn build_coinbase_tx(
    tpl: &GetBlockTemplateResult,
    payout: &Address,
    extranonce: u64,
) -> Result<Transaction> {
    let height_i64: i64 = tpl
        .height
        .try_into()
        .with_context(|| format!("template height {} exceeds i64::MAX", tpl.height))?;

    // scriptSig layout: <BIP34 height> <extranonce LE bytes> <"prism-btc">
    let script_sig = ScriptBuilder::new()
        .push_int(height_i64)
        .push_slice(extranonce.to_le_bytes())
        .push_slice(b"prism-btc")
        .into_script();

    let mut input = TxIn {
        previous_output: OutPoint::null(),
        script_sig,
        sequence: Sequence::MAX,
        witness: Witness::new(),
    };
    input.witness.push(COINBASE_WITNESS_RESERVED);

    let mut outputs = vec![TxOut {
        value: Amount::from_sat(tpl.coinbase_value.to_sat()),
        script_pubkey: payout.script_pubkey(),
    }];

    let commitment_script: &ScriptBuf = &tpl.default_witness_commitment;
    if !commitment_script.as_bytes().is_empty() {
        outputs.push(TxOut {
            value: Amount::ZERO,
            script_pubkey: commitment_script.clone(),
        });
    }

    Ok(Transaction {
        version: bitcoin::transaction::Version::TWO,
        lock_time: LockTime::ZERO,
        input: vec![input],
        output: outputs,
    })
}
