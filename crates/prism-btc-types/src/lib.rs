//! Bitcoin domain types for prism-btc.
//!
//! ## Key types
//!
//! - [`BlockHash`] — 32-byte SHA256d output; W8 per-byte ring
//! - [`Target`] — compact nBits encoding; decodes to a 32-byte big-endian threshold
//! - [`MerkleRoot`] — 32-byte Merkle tree root; same ring as `BlockHash`
//! - [`BlockHeader`] — five fixed header fields (nonce is the free dimension,
//!   injected by the miner)
//! - [`Version`], [`Timestamp`], [`Bits`] — typed u32 newtypes for the three
//!   integer fields of `BlockHeader`
//! - [`TriadicCoords`] — PRISM triadic decomposition of a hash
//! - [`BlockHashTag`] — phantom tag for `Grounded<ConstrainedTypeInput, _>`
//! - [`BlockHashGrounded`] — alias for the formally-grounded block-hash certificate

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod block_hash;
pub mod block_header;
pub mod merkle_root;
pub mod scalars;
pub mod tag;
pub mod target;
pub mod triadic;

pub use block_hash::BlockHash;
pub use block_header::BlockHeader;
pub use merkle_root::MerkleRoot;
pub use scalars::{Bits, Timestamp, Version};
pub use tag::BlockHashTag;
pub use target::Target;
pub use triadic::TriadicCoords;

/// Ergonomic alias for the grounded block-hash certificate.
///
/// `Grounded<ConstrainedTypeInput, BlockHashTag>` is produced by
/// `prism-btc-reduction::block_hash_shape_certificate` (used by both
/// `prism-btc::genesis` and the mining loop).
pub type BlockHashGrounded = uor_foundation::enforcement::Grounded<
    uor_foundation::enforcement::ConstrainedTypeInput,
    BlockHashTag,
>;
