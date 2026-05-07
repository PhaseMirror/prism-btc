//! Foundation substitution-axis selections (ADR-007/010/018).
//!
//! - [`bounds::PrismBtcBounds`] — the `HostBounds` profile per architecture §3.2.
//! - [`hasher::Sha256dHasher`] — the `Hasher` body (pure-Rust SHA-256d) per §3.3.
//!
//! `HostTypes` is bound to `uor_foundation::DefaultHostTypes` directly at
//! the model declaration site; no prism-btc-specific selection is required
//! (architecture §3.1).

pub mod bounds;
pub mod hasher;

pub use bounds::PrismBtcBounds;
pub use hasher::Sha256dHasher;
