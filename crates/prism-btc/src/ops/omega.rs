//! `OmegaBtc` — Specialized Bitcoin Ω Operator.
//!
//! Formalizes Bitcoin PoW as a lawful restriction of the Core Multiplicity Operator.
//! The W32 fiber traversal is modeled as a spectral decomposition of the 
//! specialized operator Ω_btc.

use crate::ops::sha256::sha256d_display;
use crate::ops::header::splice_nonce;

/// Universal Multiplicity Constant Λm (standard anchor).
pub const LAMBDA_M_BTC: f64 = 1.738317;

/// Specialized Ω operator for the Bitcoin sector.
/// Built as a restriction functor R_btc: Ω -> Ω_btc.
pub struct OmegaBtc {
    /// System dimension (e.g., number of primes in the base).
    pub dim: usize,
    /// Lawfulness epsilon bound.
    pub epsilon: f64,
    /// Stability anchor Λm.
    pub anchor: f64,
}

impl Default for OmegaBtc {
    fn default() -> Self {
        Self::new(12, 1e-4, LAMBDA_M_BTC)
    }
}

impl OmegaBtc {
    /// Instantiate a new Bitcoin-specialized Ω operator.
    pub fn new(dim: usize, epsilon: f64, anchor: f64) -> Self {
        Self { dim, epsilon, anchor }
    }

    /// σ-projection (SHA-256d) as a spectral evaluation over a fiber point.
    /// Under the Multiplicity lens, this is the mapping from the W32 
    /// coordinate ring to the 256-bit energy state.
    pub fn project(&self, prefix: &[u8; 76], nonce: u32) -> [u8; 32] {
        let header = splice_nonce(prefix, nonce);
        sha256d_display(&header)
    }

    /// Admissibility gate: does this spectral state satisfy the protocol target?
    pub fn is_admissible(&self, digest: &[u8; 32], target: &[u8; 32]) -> bool {
        // Lexicographic comparison in display order
        digest <= target
    }

    /// Lawfulness score: computes the resonance between the found digest
    /// and the stability anchor Λm.
    pub fn lawfulness_score(&self, digest: &[u8; 32]) -> f64 {
        // In a real realization, this would compute the spectral residual.
        // For now, we use a proxy based on the digest's entropy relative to Λm.
        let entropy = digest.iter().map(|&b| b as f64).sum::<f64>() / 256.0;
        let residual = (entropy - self.anchor).abs();
        (-residual).exp()
    }
}
