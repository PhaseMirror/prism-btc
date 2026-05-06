use prism_btc_types::{Bits, BlockHeader, MerkleRoot, Timestamp, Version};
use uor_foundation::enforcement::{DigestProjectionMap, ProjectionMapKind, Total};

use crate::certificate::BlockCertificate;
use crate::compile_unit::block_hash_shape_certificate;
use crate::error::{ConvergenceFailure, InvalidLength};
use crate::nonce_iter::NonceIter;
use crate::sha256d::sha256d;

/// Run the σ-convergence loop.
///
/// `Sigma` is the morphism kind of the σ-projection encoded in `hash_fn`.
/// It is bounded by `ProjectionMapKind + Total`: every nonce produces some
/// digest, but the σ-projection need not be invertible or
/// structure-preserving. Bitcoin's σ-projection (SHA256d) is the canonical
/// instance with `Sigma = DigestProjectionMap`.
///
/// The UOR shape certification runs exactly once (before the nonce loop) via
/// the infallible `block_hash_shape_certificate`. The certificate is
/// identical for every candidate in the round; it is cloned into the winning
/// `BlockCertificate`.
pub fn run_convergence<Sigma, F>(
    header: BlockHeader,
    target_bytes: [u8; 32],
    hash_fn: F,
) -> Result<BlockCertificate<Sigma>, ConvergenceFailure>
where
    Sigma: ProjectionMapKind + Total,
    F: Fn(u32) -> [u8; 32],
{
    let grounded = block_hash_shape_certificate();

    let mut iter = NonceIter::new();
    while let Some(nonce) = iter.next_nonce() {
        let hash = hash_fn(nonce);
        // Fast pre-filter: lexicographic target check.
        // The vast majority of candidates (≈ 2^(256 - difficulty_bits)) are rejected here.
        if hash > target_bytes {
            continue;
        }
        return Ok(BlockCertificate::new(grounded.clone(), nonce, hash, header));
    }
    Err(ConvergenceFailure::FiberExhausted)
}

/// Certify an existing 80-byte wire-format block header by re-running the
/// σ-projection (SHA256d) and the shape pipeline.
///
/// Used by `Boundary::decode` in `prism-btc`. The shape pipeline is infallible
/// (const-validated), so the only failure mode is a wrong byte-slice length.
/// Returns `BlockCertificate<DigestProjectionMap>` because Bitcoin's
/// σ-projection is fixed at the wire boundary.
pub fn certify_wire_bytes(
    bytes: &[u8],
) -> Result<BlockCertificate<DigestProjectionMap>, InvalidLength> {
    if bytes.len() != 80 {
        return Err(InvalidLength { got: bytes.len() });
    }

    let grounded = block_hash_shape_certificate();

    // All slice operations below are guaranteed by the `bytes.len() != 80` guard above.
    let version = u32::from_le_bytes(
        bytes[0..4]
            .try_into()
            .expect("validated 80-byte buffer at entry; slice bounds guaranteed"),
    );
    let mut prev_hash = [0u8; 32];
    prev_hash.copy_from_slice(&bytes[4..36]);
    let mut merkle_bytes = [0u8; 32];
    merkle_bytes.copy_from_slice(&bytes[36..68]);
    let timestamp = u32::from_le_bytes(
        bytes[68..72]
            .try_into()
            .expect("validated 80-byte buffer at entry; slice bounds guaranteed"),
    );
    let bits_val = u32::from_le_bytes(
        bytes[72..76]
            .try_into()
            .expect("validated 80-byte buffer at entry; slice bounds guaranteed"),
    );
    let nonce = u32::from_le_bytes(
        bytes[76..80]
            .try_into()
            .expect("validated 80-byte buffer at entry; slice bounds guaranteed"),
    );

    let header = BlockHeader {
        version: Version(version),
        prev_hash,
        merkle_root: MerkleRoot::from_bytes(merkle_bytes),
        timestamp: Timestamp(timestamp),
        bits: Bits(bits_val),
    };

    Ok(BlockCertificate::new(
        grounded,
        nonce,
        sha256d(bytes),
        header,
    ))
}
