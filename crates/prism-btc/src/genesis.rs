use prism_btc_types::BlockHash;
use uor_foundation::enforcement::Grounded;
use uor_foundation_macros::{uor_ground, uor_grounded};

/// Return a formally-grounded `BlockHash` shape certificate.
///
/// Runs the UOR pipeline over `BlockHash`'s `ConstrainedTypeShape` constraints
/// (W8 per-byte residue constraint, 32 sites). Used as the genesis regression anchor.
///
/// Bitcoin genesis parameters (for reference):
///   version:     1
///   prev_hash:   [0u8; 32]
///   merkle_root: 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b
///   timestamp:   1231006505  (2009-01-03 18:15:05 UTC)
///   bits:        0x1d00ffff
///   nonce:       2083236893  (0x7c2bac1d)
///   hash:        000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f
///
/// The `uor_ground!` macro calls `run_pipeline::<BlockHash>` on a default
/// `BlockHash` value. If the pipeline fails, it panics with
/// `"uor_ground! pipeline failure: reduction:ConvergenceStall"`.
///
/// `#[uor_grounded(level = "W32")]` emits a static Witt level assertion ensuring
/// this function operates at Z/(2^32)Z, consistent with `converge_at_w32()`.
#[uor_grounded(level = "W32")]
pub(crate) fn genesis_block_hash_internal() -> Grounded<BlockHash> {
    uor_ground! {
        compile_unit genesis_block_hash {
            root_term: { 0 };
            witt_level_ceiling: W32;
            thermodynamic_budget: 4294967295.0;
            target_domains: { ComposedAlgebraic };
        } as Grounded<BlockHash>
    }
}
