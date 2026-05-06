use uor_foundation::enforcement::{GroundingMapKind, Invertible, ProjectionMapKind, Total};

/// A type that crosses the raw-bytes / certified-types boundary via decode/encode.
///
/// The `Ingest` and `Emit` associated types record, at the type level, the
/// foundation morphism kinds that realise the round-trip:
///
/// - `Ingest: GroundingMapKind + Total + Invertible` — bytes flowing into a
///   typed certificate. `Total + Invertible` means every well-formed wire
///   payload decodes to exactly one certificate.
/// - `Emit: ProjectionMapKind + Total + Invertible` — typed certificate
///   flowing back to bytes. `Total + Invertible` closes the round-trip.
///
/// Together they form a **zero-cost isomorphism** between the wire-byte space
/// and the certified-type space: encode → decode → encode is the identity on
/// wire bytes; decode → encode → decode is the identity on certificates.
///
/// `decode` always re-certifies by re-running the full UOR pipeline — it
/// cannot be bypassed.
pub trait Boundary: Sized {
    type Error;
    type Ingest: GroundingMapKind + Total + Invertible;
    type Emit: ProjectionMapKind + Total + Invertible;
    fn decode(bytes: &[u8]) -> Result<Self, Self::Error>;
    fn encode(&self) -> Vec<u8>;
}

/// Decode error returned by `Boundary::decode` for `BlockCertificate`.
///
/// The shape pipeline is infallible (const-validated CompileUnit), so the
/// only failure mode is a wrong byte-slice length.
#[derive(Debug)]
pub struct BoundaryDecodeError {
    pub got: usize,
}
