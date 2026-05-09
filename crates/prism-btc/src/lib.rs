//! prism-btc — the prism implementor for Bitcoin proof-of-work.
//!
//! Real-time structural inference, expressed as a foundation 0.3.2
//! `PrismModel<H, B, A>`: the input shape is the 80-byte canonical
//! Bitcoin block header ([`MiningInput`]); the output shape is
//! foundation's identity `ConstrainedTypeInput`; the route is the
//! identity term tree; the application `Hasher` is [`Sha256dHasher`]
//! (pure-Rust SHA-256d).
//!
//! prism-btc owns the W32 nonce-fiber traversal that finds the
//! admitting fiber point ([`mine`], [`mine_parallel`]) and the runtime
//! evaluator for SHA-256d in display order
//! ([`crate::ops::sha256::sha256d_display`]) — that 32-byte digest is
//! the Bitcoin block hash carried on [`MiningOutcome::digest`].
//! Foundation owns the shape attestation: invoking
//! `BitcoinMiningModel::forward` on the admitted 80-byte header mints a
//! `Grounded<ConstrainedTypeInput, MiningTag>` ([`MiningWitness`])
//! whose `content_fingerprint` is the application-Hasher digest of the
//! CompileUnit metadata, certifying the typed-iso path under
//! `(DefaultHostTypes, PrismBtcBounds, Sha256dHasher)`.
//!
//! See [`ARCHITECTURE.md`](https://github.com/afflom/prism-btc/blob/main/ARCHITECTURE.md)
//! for the normative specification.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod domain;
pub mod model;
pub mod ops;
pub mod pipeline;
pub mod shapes;

// Public façade.
pub use domain::{
    Bits, BlockHash, BlockHeader, MerkleRoot, MiningTag, MiningWitness, Target, Timestamp,
    TriadicCoords, Version,
};
pub use model::{BitcoinMiningModel, BitcoinMiningRoute, MiningInput};
pub use ops::audit::FractalTrace;
pub use pipeline::{block_hash_grounded, mine, MiningFailure, MiningOutcome};
pub use shapes::{PrismBtcBounds, Sha256dHasher};

#[cfg(feature = "std")]
pub use pipeline::mine_parallel;

// Cancel hooks for tip-watcher-driven aborts.
pub use ops::traversal::{Cancel, Cancelled, FiberOutcome, NeverCancel};

#[cfg(feature = "std")]
pub use ops::traversal::traverse_parallel;

// Wire-format helpers — used by the bitcoind boundary in prism-btc-node
// to assemble the final 80-byte block bytes.
pub use ops::header::{serialize_header, serialize_prefix, splice_nonce};
pub use ops::merkle::merkle_root_internal;
pub use ops::sha256::{sha256, sha256d_display, sha256d_internal};
