use crate::merkle_root::MerkleRoot;
use crate::scalars::{Bits, Timestamp, Version};

/// Pure Bitcoin block header fields (without nonce).
///
/// The nonce is NOT stored here — it is the free dimension injected by the miner
/// during the σ-convergence loop in `prism-btc-reduction`.
#[derive(Clone)]
pub struct BlockHeader {
    pub version: Version,
    pub prev_hash: [u8; 32],
    pub merkle_root: MerkleRoot,
    pub timestamp: Timestamp,
    pub bits: Bits,
}
