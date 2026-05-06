//! σ-convergence reduction layer for prism-btc.
//!
//! - [`BlockCertificate`] — wraps the un-fabricatable
//!   `Grounded<ConstrainedTypeInput, BlockHashTag>` together with the triadic
//!   coordinates and the wire-format header/nonce. Phantom-typed by the
//!   σ-projection morphism kind (`Sigma: ProjectionMapKind + Total`).
//! - [`run_convergence`] — runs the σ-convergence loop over a (header, target)
//!   pair. The σ-projection (SHA256d) is passed as a closure from `prism-btc`.
//!   Shape certification runs once before the loop, not per candidate.
//! - [`certify_wire_bytes`] — certifies an existing 80-byte wire header by
//!   re-running the σ-projection and shape certification.
//! - [`block_hash_shape_certificate`] — the one shape-certificate used by
//!   genesis and every mining round.
//! - [`Fnv1aHasher16`] — substrate hasher for CompileUnit content fingerprints.
//! - [`serialize_header`] — the canonical 80-byte wire layout.
//! - [`sha256d`] — the σ-projection over wire bytes.
//! - [`ConvergenceFailure`] / [`InvalidLength`] — failure modes.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod certificate;
pub mod compile_unit;
pub mod convergence;
pub mod error;
pub mod hasher;
pub(crate) mod nonce_iter;
pub mod serialize;
pub mod sha256d;

pub use certificate::BlockCertificate;
pub use compile_unit::block_hash_shape_certificate;
pub use convergence::{certify_wire_bytes, run_convergence};
pub use error::{ConvergenceFailure, InvalidLength};
pub use hasher::Fnv1aHasher16;
pub use serialize::serialize_header;
pub use sha256d::sha256d;
