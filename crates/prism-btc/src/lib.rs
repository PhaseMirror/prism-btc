//! Bitcoin mining as σ-convergence over the nonce fiber of the 32×W8 Datum space.
//!
//! The block header (80 bytes) is the source space. The σ-projection (SHA256d)
//! maps each (header, nonce) pair to a candidate 32-byte Datum. The target
//! shape constraint (leading zero bytes) defines a sub-bundle of the Datum
//! space. Mining is a search for a nonce whose image under σ lands in that
//! sub-bundle; the UOR shape certification runs exactly once per round and is
//! cloned into every winning candidate's [`BlockCertificate`].
//!
//! ## Type-level morphism kinds
//!
//! Two distinct morphisms appear in the prism-btc story; both are carried at
//! the type level via foundation `MorphismKind` markers:
//!
//! - **σ-projection** (header‖nonce → 32-byte digest):
//!   `Sigma = DigestProjectionMap` — `Total`, neither `Invertible` nor
//!   `PreservesStructure`. Carried as the phantom parameter of
//!   [`BlockCertificate`].
//! - **Wire round-trip** (`BlockCertificate` ↔ 80 bytes):
//!   `Ingest = BinaryGroundingMap`, `Emit = BinaryProjectionMap` —
//!   both `Total + Invertible`. Carried by the [`Boundary`] trait's
//!   associated types. This is the zero-cost isomorphism between the
//!   wire-byte space and the certified-type space.
//!
//! ## Entry points
//!
//! - [`MiningRound`] — σ-convergence context; call `.converge()` to mine
//! - [`genesis`] — formally-grounded block-hash shape certificate
//! - [`Boundary`] — trait for crossing the raw-bytes / certified-types boundary
//! - [`prelude`] — re-exports all client-facing types
//!
//! No `u32` nonce accessors and no convergence-loop mechanics appear in the
//! public surface; the nonce lives only inside `encode_wire()` bytes (because
//! Bitcoin protocol requires it there).

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub(crate) mod boundary_impls;
mod mine;
pub(crate) mod mining_round;
pub mod prelude;
pub mod traits;

pub use mining_round::MiningRound;
pub use prism_btc_reduction::{BlockCertificate, ConvergenceFailure};
pub use prism_btc_types::{
    Bits, BlockHash, BlockHashGrounded, BlockHashTag, BlockHeader, MerkleRoot, Target, Timestamp,
    TriadicCoords, Version,
};
pub use traits::{Boundary, BoundaryDecodeError};

/// Return the formally-grounded block-hash shape certificate.
///
/// Produced by `pipeline::run_const::<_, BinaryGroundingMap, Fnv1aHasher16>`
/// against the const-validated CompileUnit baked at compile time. The result
/// is tagged with `BlockHashTag` for type-level domain distinction. The
/// certificate is identical for every block; it certifies the shape, not a
/// specific hash value. **Infallible** — see
/// [`prism_btc_reduction::block_hash_shape_certificate`].
pub fn genesis() -> BlockHashGrounded {
    prism_btc_reduction::block_hash_shape_certificate()
}
