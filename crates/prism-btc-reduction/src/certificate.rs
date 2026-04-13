use prism_btc_types::{BlockHash, TriadicCoords};
use uor_foundation::enforcement::Grounded;

/// A formally-grounded mining result.
///
/// The `Grounded<BlockHash>` field is produced solely by `uor_foundation::pipeline::run_pipeline`
/// — it cannot be fabricated by user code. This structural guarantee is the UOR enforcement
/// of `freeRank = 0`: the shape has been formally certified by the 7-stage SAT pipeline.
pub struct MiningCertificate {
    pub grounded: Grounded<BlockHash>,
    pub nonce: u32,
    pub coords: TriadicCoords,
}

impl MiningCertificate {
    pub fn new(grounded: Grounded<BlockHash>, nonce: u32, hash: [u8; 32]) -> Self {
        Self {
            grounded,
            nonce,
            coords: TriadicCoords::from_hash(&hash),
        }
    }

    /// The certified block hash bytes.
    pub fn hash_bytes(&self) -> &[u8; 32] {
        &self.coords.datum
    }

    /// The winning nonce value.
    pub fn nonce(&self) -> u32 {
        self.nonce
    }

    /// Global Hamming weight (active bits) of the block hash.
    pub fn stratum(&self) -> u32 {
        self.coords.stratum
    }

    /// Non-zero byte mask of the block hash (bit i set iff byte i != 0).
    pub fn spectrum(&self) -> u32 {
        self.coords.spectrum
    }
}
