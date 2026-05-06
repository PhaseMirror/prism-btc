//! Re-exports all client-facing types.
//!
//! Import `prism_btc::prelude::*` for the complete public API surface.

pub use crate::traits::{Boundary, BoundaryDecodeError};
pub use crate::{genesis, MiningRound};
pub use prism_btc_reduction::{BlockCertificate, ConvergenceFailure};
pub use prism_btc_types::{
    Bits, BlockHash, BlockHashGrounded, BlockHashTag, BlockHeader, MerkleRoot, Target, Timestamp,
    TriadicCoords, Version,
};
pub use uor_foundation::enforcement::{
    BinaryGroundingMap, BinaryProjectionMap, ConstrainedTypeInput, DigestProjectionMap, Grounded,
    GroundingMapKind, Invertible, ProjectionMapKind, Total, Validated,
};
