//! Bitcoin mining as σ-convergence over the nonce fiber of the 32×W8 Datum space.
//!
//! The block header (80 bytes) is the source space. The σ-projection (SHA256d)
//! maps each (header, nonce) pair to a candidate 32-byte Datum. The target
//! shape constraint (leading zero bytes) defines a sub-bundle of the Datum
//! space. Mining is a search for a nonce whose image under σ lands in that
//! sub-bundle and passes the UOR `run_pipeline` certification.
//!
//! ## Entry points
//!
//! - [`MiningRound`] — σ-convergence context; call `.converge()` to mine
//! - [`genesis`] — compile-time-certified genesis block hash constant
//! - [`Boundary`] — trait for crossing the raw-bytes / certified-types boundary
//! - [`prelude`] — re-exports all client-facing types
//!
//! Import `prism_btc::prelude::*` for all client-facing types.
//! No raw byte functions, no nonce values, and no convergence mechanics
//! appear in the public surface.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

// Internal implementation modules — not part of the public surface
mod genesis;
mod mine;
mod serialize;
mod sha256d;

// Public API modules
pub(crate) mod boundary_impls;
pub(crate) mod mining_round;
pub mod prelude;
pub mod traits;

pub use mining_round::MiningRound;
pub use prism_btc_reduction::{BlockCertificate, ConvergenceFailure};
pub use prism_btc_types::{BlockHash, BlockHeader, MerkleRoot, Target, TriadicCoords};
pub use traits::{Boundary, BoundaryDecodeError, Triadic};

/// Return the formally-grounded genesis block hash certificate.
///
/// Produced by `uor_ground!` at call time — cannot be fabricated.
/// The `Grounded<BlockHash>` carries non-zero `unit_address` and W32 `witt_level_bits`.
pub fn genesis() -> uor_foundation::enforcement::Grounded<BlockHash> {
    genesis::genesis_block_hash_internal()
}
