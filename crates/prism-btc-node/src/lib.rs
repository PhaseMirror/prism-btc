//! Bitcoin Core RPC integration for prism-btc.
//!
//! End-to-end mining: fetch a block template from a running bitcoind, build the
//! coinbase + merkle tree using rust-bitcoin, hand the header to
//! [`prism_btc::MiningRound`] for σ-convergence, then reassemble and submit
//! the full block.
//!
//! prism-btc itself owns the σ-convergence loop and the typed certificate
//! ([`prism_btc::BlockCertificate`]); rust-bitcoin owns the
//! transaction/script/merkle machinery; this crate is the wiring.

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

use prism_btc::{
    Bits, BlockCertificate, BlockHeader, ConvergenceFailure, MerkleRoot, MiningRound, Target,
    Timestamp, Version,
};

const COINBASE_WITNESS_RESERVED: [u8; 32] = [0u8; 32];

/// One end-to-end mining attempt against a connected bitcoind.
pub struct PrismMiner {
    client: Client,
    payout_address: Address,
    network: Network,
}

impl PrismMiner {
    pub fn connect(
        rpc_url: &str,
        auth: Auth,
        payout_address: &str,
        network: Network,
    ) -> Result<Self> {
        let client = Client::new(rpc_url, auth).context("RPC client connect")?;
        let address = payout_address
            .parse::<Address<_>>()
            .context("parse payout address")?
            .require_network(network)
            .context("payout address network mismatch")?;
        Ok(Self {
            client,
            payout_address: address,
            network,
        })
    }

    /// Fetch a block template, mine it via prism-btc, submit, return summary.
    pub fn mine_one_block(&self) -> Result<MinedBlock> {
        // Pick rules that match the network. Signet needs the Signet rule;
        // others reject it.
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

        let template = self
            .client
            .get_block_template(
                GetBlockTemplateModes::Template,
                rules,
                &[] as &[GetBlockTemplateCapabilities],
            )
            .context("getblocktemplate RPC")?;

        let job = MiningJob::from_template(&template, &self.payout_address)?;
        let header = job.header.clone();
        let target = job.target;

        let cert = MiningRound::new(header, target)
            .converge()
            .map_err(|e| match e {
                ConvergenceFailure::FiberExhausted => anyhow::anyhow!(
                    "nonce fiber exhausted (2^32 candidates) without satisfying target"
                ),
            })?;

        // The mined wire bytes carry the winning nonce at [76..80].
        let wire80 = cert.encode_wire();
        let nonce = u32::from_le_bytes([wire80[76], wire80[77], wire80[78], wire80[79]]);

        let block = job.assemble(nonce);

        // Returns Ok(()) on accept; Err on reject (with bitcoind's reason string).
        self.client
            .submit_block(&block)
            .context("submitblock rejected")?;

        Ok(MinedBlock {
            hash: block.block_hash(),
            height: template.height,
            nonce,
            cert,
            tx_count: block.txdata.len(),
        })
    }

    pub fn network(&self) -> Network {
        self.network
    }
}

pub struct MinedBlock {
    pub hash: BtcBlockHash,
    pub height: u64,
    pub nonce: u32,
    pub cert: BlockCertificate,
    pub tx_count: usize,
}

/// All the per-template state a mining attempt needs.
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
    fn from_template(tpl: &GetBlockTemplateResult, payout: &Address) -> Result<Self> {
        let other_txs: Vec<Transaction> = tpl
            .transactions
            .iter()
            .map(|t| t.transaction().context("decode template tx"))
            .collect::<Result<_>>()?;

        let coinbase = build_coinbase_tx(tpl, payout)?;

        // Block merkle root: leaves are the legacy (non-witness) txids.
        let mut leaves = Vec::with_capacity(other_txs.len() + 1);
        leaves.push(coinbase.compute_txid().to_raw_hash());
        for tx in &other_txs {
            leaves.push(tx.compute_txid().to_raw_hash());
        }
        let merkle_root_raw = calculate_root(leaves.into_iter())
            .context("merkle root computation (empty leaf set unexpected)")?;
        let btc_merkle = bitcoin::TxMerkleNode::from_raw_hash(merkle_root_raw);

        // Decode `bits`: 4-byte big-endian compact target as returned by the RPC.
        if tpl.bits.len() != 4 {
            bail!(
                "getblocktemplate.bits has unexpected length {}",
                tpl.bits.len()
            );
        }
        let bits_u32 = u32::from_be_bytes([tpl.bits[0], tpl.bits[1], tpl.bits[2], tpl.bits[3]]);
        let btc_bits = CompactTarget::from_consensus(bits_u32);

        // Bitcoin's wire-format header timestamp is u32 (Y2106 protocol limit);
        // bitcoind's RPC returns u64 only because of JSON typing. A u64 that
        // doesn't fit in u32 is a protocol-violating template — bail loudly.
        let time_u32: u32 = tpl.current_time.try_into().with_context(|| {
            format!(
                "template current_time {} exceeds u32::MAX",
                tpl.current_time
            )
        })?;

        // Block version is i32 in the wire format; the RPC returns u32. Values
        // above i32::MAX would be a protocol-violating template.
        let version_i32: i32 = tpl
            .version
            .try_into()
            .with_context(|| format!("template version {} exceeds i32::MAX", tpl.version))?;

        // prev_hash and merkle: rust-bitcoin's BlockHash/TxMerkleNode are in
        // internal (little-endian-display) order — exactly what goes into the
        // wire-format header field and into our BlockHeader slots.
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

    /// Splice the winning nonce back into a full rust-bitcoin Block. Uses
    /// rust-bitcoin's serialisation for the wire payload so we inherit its
    /// segwit-aware tx encoding rules; prism-btc's role was to find the nonce.
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

/// Build a BIP141-compliant coinbase transaction.
///
/// - Single coinbase input with scriptSig containing BIP34 height push +
///   a "prism-btc" tag (acts as extranonce; bitcoind doesn't require it).
/// - Output 0: full block reward to the payout address.
/// - Output 1: OP_RETURN with the BIP141 witness commitment, when the
///   template provides one (segwit-active networks).
/// - Witness: the coinbase's single TxIn carries one item — 32 zero bytes,
///   the BIP141 "witness reserved value".
fn build_coinbase_tx(tpl: &GetBlockTemplateResult, payout: &Address) -> Result<Transaction> {
    let height_i64: i64 = tpl
        .height
        .try_into()
        .with_context(|| format!("template height {} exceeds i64::MAX", tpl.height))?;

    let script_sig = ScriptBuilder::new()
        .push_int(height_i64)
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

    // The template's `default_witness_commitment` is the *full* scriptPubKey
    // (OP_RETURN + 36-byte push of [0xaa, 0x21, 0xa9, 0xed, <32-byte commit>]),
    // not just the digest. Empty ScriptBuf means no segwit / no commitment.
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
