use crate::merkle_root::MerkleRoot;
use prism_btc_primitives::{Bits, Timestamp, Version};

/// Pure Bitcoin block header fields (without nonce).
///
/// The nonce is NOT stored here — it is the free dimension injected by the miner
/// during the nonce iteration loop in `prism-btc-reduction`.
pub struct BlockHeader {
    pub version: Version,
    pub prev_hash: [u8; 32],
    pub merkle_root: MerkleRoot,
    pub timestamp: Timestamp,
    pub bits: Bits,
}

/// UOR CompileUnit marker for a block header.
///
/// Carries the `UOR_COMPILE_UNIT_IRI` constant (emitted by `#[derive(CompileUnit)]`)
/// for compile-time drift detection. Field names document the `CompileUnitBuilder`
/// protocol.
///
/// The primary `mine()` path calls `run_pipeline(&block_hash, 8u16)` directly on the
/// hash output and does not need `BlockHeaderUnit`. `BlockHeaderUnit` exists to enforce
/// that the crate remains anchored to the `CompileUnit` protocol at the type level.
use uor_foundation::enforcement::Term;
use uor_foundation::{VerificationDomain, WittLevel};
use uor_foundation_macros::CompileUnit;

#[derive(CompileUnit)]
pub struct BlockHeaderUnit<'a> {
    pub builder_root_term: &'a [Term],
    pub builder_witt_level_ceiling: WittLevel,
    pub builder_thermodynamic_budget: u64,
    pub builder_target_domains: &'a [VerificationDomain],
}
