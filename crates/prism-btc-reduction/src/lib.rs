//! Nonce iteration and block certification for prism-btc.
//!
//! This crate provides the stateful components of the σ-convergence loop:
//!
//! - [`BlockCertificate`] — wraps the un-fabricatable `Grounded<BlockHash>` produced
//!   by `uor_foundation::pipeline::run_pipeline` together with the triadic coordinates.
//!   All fields private; nonce is never observable.
//! - [`run_convergence`] — runs the σ-convergence loop over a (header, target) pair.
//!   The σ-projection (SHA256d) is passed as a closure from `prism-btc`.
//! - [`certify_wire_bytes`] — certifies an existing 80-byte wire header by re-running
//!   the σ-projection and `run_pipeline`. Used by `Boundary::decode`.
//! - [`ConvergenceFailure`] — failure modes from the σ-convergence loop.
//! - [`CertifyError`] — failure modes from `certify_wire_bytes`.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod certificate;
pub mod convergence;
pub mod error;
pub(crate) mod nonce_iter;

pub use certificate::BlockCertificate;
pub use convergence::{certify_wire_bytes, run_convergence};
pub use error::{CertifyError, ConvergenceFailure};
