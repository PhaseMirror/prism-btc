use prism_btc_types::BlockHash;
use uor_foundation::enforcement::Grounded;
use uor_foundation_macros::uor_grounded;

/// Return a formally-grounded `BlockHash` shape certificate.
///
/// Runs the UOR ψ-reduction pipeline over `BlockHash`'s constraint shape,
/// producing a `Grounded<BlockHash>` that formally certifies the type inhabits
/// the W32 ring. Used as the genesis regression anchor in tests.
///
/// `#[uor_grounded(level = "W32")]` emits a static Witt level assertion ensuring
/// this function operates at Z/(2^32)Z, consistent with `mine()`.
#[uor_grounded(level = "W32")]
pub fn genesis_block_hash() -> Grounded<BlockHash> {
    prism_btc_reduction::genesis_grounded()
}
