//! prism-btc — the prism implementor for Bitcoin proof-of-work.
//!
//! Real-time structural inference: types declare the categorical
//! routing between input parameters (template prefix, target) and
//! output parameters (admitting nonce, digest, foundation-sealed
//! shape Grounded); the runtime in this crate walks that routing.
//!
//! See [`ARCHITECTURE.md`](https://github.com/afflom/prism-btc/blob/main/ARCHITECTURE.md)
//! for the normative specification.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod domain;
pub mod ops;
pub mod pipeline;
pub mod shapes;

// Public façade.
pub use domain::{
    Bits, BlockHash, BlockHashGrounded, BlockHashTag, BlockHeader, MerkleRoot, MiningTag,
    MiningWitness, Target, Timestamp, TriadicCoords, Version,
};
pub use pipeline::{block_hash_grounded, mine, MiningFailure, MiningOutcome};
pub use shapes::{PrismBtcBounds, Sha256dHasher, TargetSubBundle, TemplatePrefixShape};

#[cfg(feature = "std")]
pub use pipeline::mine_parallel;

// Cancel hooks for tip-watcher-driven aborts (used by the bitcoind
// boundary in prism-btc-node).
pub use ops::traversal::{Cancel, Cancelled, FiberOutcome, NeverCancel};

#[cfg(feature = "std")]
pub use ops::traversal::traverse_parallel;

// Wire-format helpers — used by the bitcoind boundary to assemble the
// final 80-byte block bytes from a (prefix, winning nonce) pair.
pub use ops::header::{serialize_header, serialize_prefix, splice_nonce};
pub use ops::merkle::merkle_root_internal;
pub use ops::sha256::{sha256, sha256d_display, sha256d_internal};

/// Successor of the previous `genesis()` API. Re-derives the prism-btc
/// shape attestation without running a fiber traversal. Equivalent to
/// `block_hash_grounded()`; kept as `genesis` for any callers still
/// using the old name during reconciliation.
#[doc(hidden)]
pub fn genesis() -> BlockHashGrounded {
    block_hash_grounded()
}
