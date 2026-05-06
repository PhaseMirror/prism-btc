//! prism-btc operations: the runtime that walks the foundation-typed
//! structure.
//!
//! Each operation has two surfaces:
//! - **Compile-time structural identity** declared (where applicable)
//!   in [`term`] as a `Term::Application` chain over foundation
//!   `PrimitiveOp` generators.
//! - **Runtime evaluator** in this module — pure-Rust functions the
//!   prism implementor (this crate) provides.
//!
//! Foundation 0.3.1's pipeline does not iterate over `Term` at runtime
//! (architecture §13); prism-btc's runtime is therefore the
//! load-bearing surface for actually computing Bitcoin proof-of-work.

pub mod header;
pub mod merkle;
pub mod sha256;
pub mod sigma;
pub mod term;
pub mod traversal;

pub use header::{serialize_header, serialize_prefix, splice_nonce};
pub use merkle::merkle_root_internal;
pub use sha256::{sha256, sha256d_display, sha256d_internal};
pub use sigma::{sigma_project, sigma_project_prefix};
pub use traversal::{traverse_sequential, Cancel, Cancelled, FiberOutcome, NeverCancel};

#[cfg(feature = "std")]
pub use traversal::traverse_parallel;
