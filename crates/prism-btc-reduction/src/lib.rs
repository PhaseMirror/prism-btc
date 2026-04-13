//! Nonce iteration and mining certificate for prism-btc.
//!
//! This crate provides the stateful components of the ψ-loop:
//!
//! - [`NonceIter`] — iterates the full u32 nonce space `[0, 2^32)` in plain Rust.
//!   The nonce is the free dimension; no UOR machinery is needed to iterate it.
//! - [`MiningCertificate`] — wraps the un-fabricatable `Grounded<BlockHash>` produced
//!   by `uor_foundation::pipeline::run_pipeline` together with the winning nonce and
//!   the hash's [`prism_btc_types::TriadicCoords`].
//! - [`genesis_grounded`] — returns a compile-time-verified `Grounded<BlockHash>` for
//!   the Bitcoin genesis block via `uor_ground!`.
//! - [`MineError`] — failure modes from the mining loop.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod certificate;
pub mod error;
pub mod nonce_iter;
pub mod vectors;

pub use certificate::MiningCertificate;
pub use error::MineError;
pub use nonce_iter::NonceIter;
pub use vectors::genesis_grounded;
