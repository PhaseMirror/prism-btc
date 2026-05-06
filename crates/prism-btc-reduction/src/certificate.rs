use core::marker::PhantomData;

use prism_btc_types::{BlockHashGrounded, BlockHeader, TriadicCoords};
use uor_foundation::enforcement::{DigestProjectionMap, ProjectionMapKind, Total};

use crate::serialize::serialize_header;

/// A formally-grounded mining result.
///
/// ## Type-level structure
///
/// `Sigma` is the morphism kind of the σ-projection that produced this
/// certificate's hash from the wire bytes. It is bounded by
/// `ProjectionMapKind + Total`: every wire-byte input maps to some 32-byte
/// digest, but the σ-projection is not required to be invertible or
/// structure-preserving. For Bitcoin the canonical instantiation is
/// `Sigma = DigestProjectionMap` — `Total`, neither `Invertible` nor
/// `PreservesStructure` (as one would expect of SHA256d).
///
/// ## Un-fabricability
///
/// The [`BlockHashGrounded`] field is produced solely by
/// [`crate::block_hash_shape_certificate`] (which calls `pipeline::run_const`);
/// it cannot be fabricated by user code. The sealed constructor is the UOR
/// enforcement of `freeRank = 0`: the shape has been formally certified.
///
/// All fields are private. Public accessors are: [`Self::digest`] for the
/// 32-byte hash, [`Self::grounded`] for the formal grounding, [`Self::coords`]
/// for the triadic decomposition, [`Self::encode_wire`] for the canonical
/// 80-byte payload (which contains the nonce by protocol). There is no `u32`
/// nonce accessor — the nonce is wire-format internal.
pub struct BlockCertificate<Sigma: ProjectionMapKind + Total = DigestProjectionMap> {
    grounded: BlockHashGrounded,
    nonce: u32,
    coords: TriadicCoords,
    header: BlockHeader,
    _sigma: PhantomData<Sigma>,
}

impl<Sigma: ProjectionMapKind + Total> BlockCertificate<Sigma> {
    pub(crate) fn new(
        grounded: BlockHashGrounded,
        nonce: u32,
        digest: [u8; 32],
        header: BlockHeader,
    ) -> Self {
        Self {
            grounded,
            nonce,
            coords: TriadicCoords::from_hash(&digest),
            header,
            _sigma: PhantomData,
        }
    }

    /// The formally-certified block-hash grounding certificate.
    pub fn grounded(&self) -> &BlockHashGrounded {
        &self.grounded
    }

    /// The 32-byte block hash (Bitcoin display order, big-endian most-significant first).
    ///
    /// This is the σ-projection output — equivalent to `self.coords().datum`,
    /// surfaced directly for callers that don't need the triadic decomposition.
    pub fn digest(&self) -> &[u8; 32] {
        &self.coords.datum
    }

    /// The PRISM triadic coordinates of the block hash (datum, stratum, spectrum).
    pub fn coords(&self) -> &TriadicCoords {
        &self.coords
    }

    /// Re-encode the certificate to its canonical 80-byte Bitcoin wire format.
    ///
    /// Together with [`crate::certify_wire_bytes`] this realises the
    /// `BinaryGroundingMap` ↔ `BinaryProjectionMap` isomorphism the `Boundary`
    /// trait carries at the type level: encode → decode → encode is the
    /// identity on wire bytes, and decode → encode → decode is the identity
    /// on certificates.
    pub fn encode_wire(&self) -> [u8; 80] {
        serialize_header(&self.header, self.nonce)
    }
}
