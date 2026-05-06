use prism_btc_reduction::{BlockCertificate, ConvergenceFailure};
use prism_btc_types::{BlockHeader, Target};
use uor_foundation::enforcement::DigestProjectionMap;
use uor_foundation::WittLevel;

use crate::mine::mine;

// W32 anchor: the nonce iterates in Z/(2^32)Z.
const _: () = assert!(WittLevel::W32.witt_length() == 32);

/// A single σ-convergence attempt over one (header, target) pair.
///
/// The nonce dimension is fully internal. Callers see only the certified
/// `BlockCertificate<DigestProjectionMap>` on success or a
/// `ConvergenceFailure` on failure — never individual nonce values.
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
    /// The σ-projection (SHA256d) is a `DigestProjectionMap` — `Total`, not
    /// `Invertible`, not `PreservesStructure`. The returned certificate
    /// carries that morphism kind at the type level.
    pub fn converge(self) -> Result<BlockCertificate<DigestProjectionMap>, ConvergenceFailure> {
        mine(&self.header, &self.target)
    }
}
