use prism_btc_reduction::BlockCertificate;
use uor_foundation::enforcement::{BinaryGroundingMap, BinaryProjectionMap, DigestProjectionMap};

use crate::traits::{Boundary, BoundaryDecodeError};

/// Wire-boundary isomorphism for the canonical Bitcoin certificate.
///
/// `Ingest = BinaryGroundingMap`, `Emit = BinaryProjectionMap` — both
/// `Total + Invertible`. The σ-projection kind on the certificate itself
/// (`Sigma = DigestProjectionMap`, only `Total`) is the *internal* morphism;
/// the wire boundary is a separate isomorphism over the surrounding 80 bytes.
impl Boundary for BlockCertificate<DigestProjectionMap> {
    type Error = BoundaryDecodeError;
    type Ingest = BinaryGroundingMap;
    type Emit = BinaryProjectionMap;

    /// Decode an 80-byte wire-format block header into a certified `BlockCertificate`.
    /// Always re-runs the σ-projection (SHA256d) and the (infallible) UOR shape pipeline.
    /// The only failure mode is a wrong byte-slice length.
    fn decode(bytes: &[u8]) -> Result<Self, BoundaryDecodeError> {
        prism_btc_reduction::certify_wire_bytes(bytes)
            .map_err(|e| BoundaryDecodeError { got: e.got })
    }

    /// Encode the certificate back to the canonical 80-byte Bitcoin wire format.
    fn encode(&self) -> Vec<u8> {
        self.encode_wire().to_vec()
    }
}
