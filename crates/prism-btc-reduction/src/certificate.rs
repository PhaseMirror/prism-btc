use prism_btc_types::{BlockHash, BlockHeader, TriadicCoords};
use uor_foundation::enforcement::Grounded;

/// A formally-grounded mining result.
///
/// The `Grounded<BlockHash>` field is produced solely by `uor_foundation::pipeline::run_pipeline`
/// — it cannot be fabricated by user code. This structural guarantee is the UOR enforcement
/// of `freeRank = 0`: the shape has been formally certified by the pipeline.
///
/// All fields are private. The nonce is never observable by callers — it exists only as
/// internal wire-format state accessible via `nonce_wire()` for `Boundary::encode()`.
pub struct BlockCertificate {
    grounded: Grounded<BlockHash>,
    nonce: u32,
    coords: TriadicCoords,
    header: BlockHeader,
}

impl BlockCertificate {
    pub(crate) fn new(
        grounded: Grounded<BlockHash>,
        nonce: u32,
        hash: [u8; 32],
        header: BlockHeader,
    ) -> Self {
        Self {
            grounded,
            nonce,
            coords: TriadicCoords::from_hash(&hash),
            header,
        }
    }

    /// The formally-certified block hash (`Grounded<BlockHash>`).
    pub fn hash(&self) -> &Grounded<BlockHash> {
        &self.grounded
    }

    /// The PRISM triadic coordinates of the block hash (datum, stratum, spectrum).
    pub fn coords(&self) -> &TriadicCoords {
        &self.coords
    }

    /// Wire-format nonce — used only by `Boundary::encode()` in `prism-btc`.
    ///
    /// This accessor is `pub` (not `pub(crate)`) because `prism-btc` is a separate crate.
    /// It is intentionally not re-exported in the prelude and its name communicates
    /// its restricted purpose: reconstruction of the 80-byte wire format only.
    pub fn nonce_wire(&self) -> u32 {
        self.nonce
    }

    /// Wire-format header — used only by `Boundary::encode()` in `prism-btc`.
    ///
    /// Same rationale as `nonce_wire()`: accessible cross-crate for wire encoding only,
    /// not for general use, and not re-exported in the prelude.
    pub fn header_wire(&self) -> &BlockHeader {
        &self.header
    }
}
