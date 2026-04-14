use prism_btc_reduction::{BlockCertificate, ConvergenceFailure};
use prism_btc_types::{BlockHeader, Target};
use uor_foundation_macros::uor_grounded;

use crate::mine::mine;

/// A single σ-convergence attempt over one (header, target) pair.
///
/// The nonce dimension is fully internal. Callers see only `BlockCertificate`
/// on success or `ConvergenceFailure` on failure — never individual nonce values.
pub struct MiningRound {
    header: BlockHeader,
    target: Target,
}

impl MiningRound {
    /// Construct a new convergence context for the given header and target.
    pub fn new(header: BlockHeader, target: Target) -> Self {
        Self { header, target }
    }

    /// Run the σ-convergence loop until a certified `BlockCertificate` is found
    /// or the nonce fiber is exhausted.
    ///
    /// Delegates to `converge_at_w32` — a free function carrying the
    /// `#[uor_grounded(level = "W32")]` Witt assertion. The `#[uor_grounded]`
    /// attribute is on the free function (not this impl method) because
    /// the macro targets free functions.
    pub fn converge(self) -> Result<BlockCertificate, ConvergenceFailure> {
        converge_at_w32(self.header, self.target)
    }
}

/// Convergence entry point for the W32 nonce ring.
///
/// `#[uor_grounded(level = "W32")]` asserts this function operates at Z/(2^32)Z —
/// the nonce ring — consistent with the W32 grounding level for nonce iteration.
///
/// SHA256d is the σ-projection (ingestion hash), NOT the UOR ψ-map. Foundation
/// reserves ψ for the categorical functor chain ψ_1..ψ_9. SHA256d is a
/// non-structure-preserving avalanche function satisfying none of those obligations.
#[uor_grounded(level = "W32")]
pub(crate) fn converge_at_w32(
    header: BlockHeader,
    target: Target,
) -> Result<BlockCertificate, ConvergenceFailure> {
    mine(&header, &target)
}
