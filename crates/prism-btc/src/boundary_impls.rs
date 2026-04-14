use prism_btc_reduction::{BlockCertificate, CertifyError};
use prism_btc_types::TriadicCoords;

use crate::serialize::serialize_header;
use crate::traits::{Boundary, BoundaryDecodeError, Triadic};

impl Triadic for BlockCertificate {
    fn coords(&self) -> &TriadicCoords {
        self.coords()
    }
}

impl Boundary for BlockCertificate {
    type Error = BoundaryDecodeError;

    /// Decode an 80-byte wire-format block header into a certified `BlockCertificate`.
    ///
    /// Always re-runs the σ-projection (SHA256d) and `run_pipeline` — cannot be bypassed.
    fn decode(bytes: &[u8]) -> Result<Self, BoundaryDecodeError> {
        prism_btc_reduction::certify_wire_bytes(bytes).map_err(|e| match e {
            CertifyError::InvalidLength { got } => BoundaryDecodeError::InvalidLength { got },
            CertifyError::PipelineRejected(r) => BoundaryDecodeError::PipelineRejected(r),
        })
    }

    /// Encode the certificate back to the canonical 80-byte Bitcoin wire format.
    fn encode(&self) -> Vec<u8> {
        serialize_header(self.header_wire(), self.nonce_wire()).to_vec()
    }
}
