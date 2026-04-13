//! Bitcoin domain types as UOR-constrained ring elements.
//!
//! Every type in this crate carries a `#[derive(ConstrainedType)]` annotation that
//! generates a [`ConstrainedTypeShape`] impl baking the type's ring constraints into
//! the binary at compile time.  Drift from the UOR ontology is a compile error.
//!
//! ## Key types
//!
//! - [`BlockHash`] — 32-byte SHA256d output; W8 per-byte ring (Z/(2^8)Z × 32)
//! - [`Target`] — compact nBits encoding; decodes to a 32-byte big-endian threshold
//! - [`MerkleRoot`] — 32-byte Merkle tree root; same ring as `BlockHash`
//! - [`BlockHeader`] — the five fixed header fields (nonce excluded — it is the free
//!   dimension injected by the miner)
//! - [`TriadicCoords`] — PRISM triadic decomposition of a hash: datum, stratum,
//!   spectrum
//!
//! Compile-time ring identity assertions live in [`assertions`] and guard against
//! regression in the UOR ring operations.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod assertions;
pub mod block_hash;
pub mod block_header;
pub mod merkle_root;
pub mod target;
pub mod triadic;

pub use block_hash::BlockHash;
pub use block_header::{BlockHeader, BlockHeaderUnit};
pub use merkle_root::MerkleRoot;
pub use target::Target;
pub use triadic::TriadicCoords;
