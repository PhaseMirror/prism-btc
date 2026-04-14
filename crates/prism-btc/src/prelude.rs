//! Re-exports all client-facing types.
//!
//! Import `prism_btc::prelude::*` for the complete public API surface.

pub use crate::traits::{Boundary, BoundaryDecodeError, Triadic};
pub use crate::{genesis, MiningRound};
pub use prism_btc_primitives::{Address, Bits, BlockHeight, FeeRate, Satoshi, Timestamp, Version};
pub use prism_btc_reduction::{BlockCertificate, ConvergenceFailure};
pub use prism_btc_types::{BlockHash, BlockHeader, MerkleRoot, Target, TriadicCoords};
pub use uor_foundation::enforcement::{Grounded, Validated};
