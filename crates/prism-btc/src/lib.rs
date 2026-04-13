//! Bitcoin mining as UOR shape-preserving morphism search.
//!
//! `prism-btc` reframes Bitcoin proof-of-work as finding a shape-preserving
//! morphism from a block header (with a free nonce dimension) to a 32-byte
//! [`BlockHash`] whose triadic coordinates satisfy the target shape constraint.
//!
//! ## ψ-loop algorithm
//!
//! ```text
//! for nonce in 0..=u32::MAX:
//!     raw  = serialize_header(header, nonce)   // 80-byte CompileUnit
//!     hash = sha256d(raw)                       // ψ-map: CompileUnit → Datum
//!     if hash > target: continue                // fast pre-filter (~2^(8N) rejects)
//!     grounded = run_pipeline(&BlockHash(hash)) // formal SAT certification
//!     return MiningCertificate { grounded, nonce, coords }
//! ```
//!
//! The `Grounded<BlockHash>` returned by `run_pipeline` is the un-fabricatable
//! certificate: its sealed constructor is only reachable through the pipeline,
//! structurally enforcing `freeRank = 0`.
//!
//! ## Entry points
//!
//! - [`mine`] — run the ψ-loop; returns a [`MiningCertificate`] on success
//! - [`genesis_block_hash`] — compile-time-verified genesis constant
//! - [`serialize_header`] — 80-byte Bitcoin wire format serialization
//! - [`sha256d`] — double-SHA256 (returns bytes in display/big-endian order)

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod genesis;
pub mod mine;
pub mod serialize;
pub mod sha256d;

pub use genesis::genesis_block_hash;
pub use mine::mine;
pub use serialize::serialize_header;
pub use sha256d::sha256d;

// Re-export primary public types for convenience
pub use prism_btc_reduction::{MineError, MiningCertificate};
pub use prism_btc_types::{BlockHash, BlockHeader, MerkleRoot, Target, TriadicCoords};
