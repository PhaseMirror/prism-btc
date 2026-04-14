use crate::merkle_root::MerkleRoot;
use prism_btc_primitives::{Bits, Timestamp, Version};

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

/// UOR CompileUnit marker for a block header.
///
/// Carries the `UOR_COMPILE_UNIT_IRI` constant (emitted by `#[derive(CompileUnit)]`)
/// for compile-time drift detection. Field names document the `CompileUnitBuilder`
/// protocol.
///
/// The primary `mine()` path calls `run_pipeline(&block_hash, BLOCK_HASH_SITE_WITT_BITS)`
/// directly on the hash output and does not need `BlockHeaderUnit`.
/// `BlockHeaderUnit` exists to enforce that the crate remains anchored to the
/// `CompileUnit` protocol at the type level.
use uor_foundation::enforcement::Term;
use uor_foundation::{VerificationDomain, WittLevel};
use uor_foundation_macros::CompileUnit;

// This struct is intentionally never constructed — it exists solely for the
// `#[derive(CompileUnit)]` compile-time drift-detection effect. The generated
// `UOR_COMPILE_UNIT_IRI` constant is referenced below to make that contract explicit.
#[allow(dead_code)]
#[derive(CompileUnit)]
pub(crate) struct BlockHeaderUnit<'a> {
    pub(crate) builder_root_term: &'a [Term],
    pub(crate) builder_witt_level_ceiling: WittLevel,
    pub(crate) builder_thermodynamic_budget: u64,
    pub(crate) builder_target_domains: &'a [VerificationDomain],
}

/// Compile-time anchor: asserts the `CompileUnit` IRI is reachable, keeping
/// the `BlockHeaderUnit` derive in scope for drift detection.
pub(crate) const _COMPILE_UNIT_ANCHOR: &str = BlockHeaderUnit::UOR_COMPILE_UNIT_IRI;
