use prism_btc_types::TriadicCoords;
use uor_foundation::enforcement::PipelineFailure;

/// Any certified hash type carries PRISM triadic coordinates (datum, stratum, spectrum).
pub trait Triadic {
    fn coords(&self) -> &TriadicCoords;
}

/// A type that crosses the raw-bytes / certified-types boundary via decode/encode.
///
/// `decode` always re-certifies by re-running the full UOR pipeline — it cannot be bypassed.
/// `encode` reconstructs the canonical wire representation from private internal state.
pub trait Boundary: Sized {
    type Error;
    fn decode(bytes: &[u8]) -> Result<Self, Self::Error>;
    fn encode(&self) -> Vec<u8>;
}

/// Decode error returned by `Boundary::decode` for `BlockCertificate`.
#[derive(Debug)]
pub enum BoundaryDecodeError {
    /// The byte slice was not exactly 80 bytes.
    InvalidLength { got: usize },
    /// The σ-projection output did not satisfy the UOR pipeline constraints.
    PipelineRejected(PipelineFailure),
}
