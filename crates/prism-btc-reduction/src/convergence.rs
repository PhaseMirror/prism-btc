use prism_btc_primitives::{Bits, Timestamp, Version};
use prism_btc_types::{BlockHash, BlockHeader, MerkleRoot, BLOCK_HASH_SITE_WITT_BITS};
use sha2::{Digest, Sha256};
use uor_foundation::pipeline::run_pipeline;

use crate::certificate::BlockCertificate;
use crate::error::{CertifyError, ConvergenceFailure};
use crate::nonce_iter::NonceIter;

/// Run the σ-convergence loop.
///
/// The `hash_fn` closure encapsulates the σ-projection (SHA256d) from the
/// 80-byte header space to the 32-byte Datum space. It is passed externally so
/// `NonceIter` stays `pub(crate)` and SHA256d stays `pub(crate)` in `prism-btc`.
///
/// **σ-projection is NOT the UOR ψ-map.** Foundation reserves ψ for the categorical
/// functor chain ψ_1..ψ_9 (Constraints → Nerve → Chain → Homology → … → KInvariants).
/// SHA256d is a deliberately non-structure-preserving avalanche function — it satisfies
/// none of the ψ obligations.
///
/// `BLOCK_HASH_SITE_WITT_BITS` (= 8, W8) is passed to `run_pipeline` to name the
/// per-site Witt level explicitly — the per-byte ring Z/(2^8)Z — not a stage count.
pub fn run_convergence<F>(
    header: BlockHeader,
    target_bytes: [u8; 32],
    hash_fn: F,
) -> Result<BlockCertificate, ConvergenceFailure>
where
    F: Fn(u32) -> [u8; 32],
{
    let mut iter = NonceIter::new();
    while let Some(nonce) = iter.next_nonce() {
        let hash = hash_fn(nonce);
        // Fast pre-filter: lexicographic target check.
        // The vast majority of candidates (≈ 2^(256 - difficulty_bits)) are rejected here.
        if hash > target_bytes {
            continue;
        }
        let block_hash = BlockHash::from_bytes(hash);
        // BLOCK_HASH_SITE_WITT_BITS = 8 = W8: the per-byte ring Z/(2^8)Z.
        // Names the Witt reference level for the 32×W8 BlockHash Datum, not a stage count.
        match run_pipeline(&block_hash, BLOCK_HASH_SITE_WITT_BITS) {
            Ok(grounded) => {
                return Ok(BlockCertificate::new(grounded, nonce, hash, header));
            }
            Err(reason) => {
                return Err(ConvergenceFailure::ReductionStall { reason });
            }
        }
    }
    Err(ConvergenceFailure::FiberExhausted)
}

/// Certify an existing 80-byte wire-format block header by re-running the full pipeline.
///
/// Used by `Boundary::decode` in `prism-btc`. Cannot be bypassed — always re-runs the
/// σ-projection (SHA256d via `sha2` directly) and `run_pipeline`. Does not accept a
/// pre-computed hash.
///
/// `sha256d` is `pub(crate)` in `prism-btc` and is unavailable here, so this function
/// calls `sha2` directly. This is intentional: `certify_wire_bytes` is a boundary
/// utility that must be self-contained in `prism-btc-reduction`.
pub fn certify_wire_bytes(bytes: &[u8]) -> Result<BlockCertificate, CertifyError> {
    if bytes.len() != 80 {
        return Err(CertifyError::InvalidLength { got: bytes.len() });
    }

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

    // σ-projection: SHA256d over the full 80 bytes (nonce included).
    // Returns bytes in Bitcoin display order (most-significant byte first).
    let first = Sha256::digest(bytes);
    let second = Sha256::digest(first);
    let mut hash: [u8; 32] = second.into();
    hash.reverse(); // Bitcoin display byte order (most-significant byte first)

    let block_hash = BlockHash::from_bytes(hash);
    match run_pipeline(&block_hash, BLOCK_HASH_SITE_WITT_BITS) {
        Ok(grounded) => Ok(BlockCertificate::new(grounded, nonce, hash, header)),
        Err(reason) => Err(CertifyError::PipelineRejected(reason)),
    }
}
