use uor_foundation::pipeline::run_pipeline;

use crate::{serialize_header, sha256d};
use prism_btc_reduction::{MineError, MiningCertificate, NonceIter};
use prism_btc_types::{BlockHash, BlockHeader, Target};
use uor_foundation_macros::uor_grounded;

/// Mine a block header by iterating the nonce space until `run_pipeline` succeeds.
///
/// **ψ-loop algorithm:**
/// 1. Serialize the 80-byte header with the candidate nonce
/// 2. SHA256d (ψ-map): 80-byte CompileUnit → 32-byte candidate Datum
/// 3. Target pre-filter: ~(2^(8N) - 1) / 2^(8N) of candidates fail here
/// 4. `run_pipeline`: formal shape certification — runs 7-stage SAT reduction
///    over BlockHash's ConstrainedTypeShape constraints (W8 per-byte, 32 sites)
/// 5. On success, wrap the un-fabricatable `Grounded<BlockHash>` in a certificate
///
/// `#[uor_grounded(level = "W32")]` emits a static Witt level assertion ensuring
/// this function operates at Z/(2^32)Z — the nonce's ring. Mismatched level = compile error.
#[uor_grounded(level = "W32")]
pub fn mine(header: &BlockHeader, target: &Target) -> Result<MiningCertificate, MineError> {
    let mut iter = NonceIter::new();

    while let Some(nonce) = iter.next_nonce() {
        // ψ-map (SHA256d): 80-byte CompileUnit → 32-byte candidate Datum
        let raw = serialize_header(header, nonce);
        let hash = sha256d(&raw);

        // Fast pre-filter: Bitcoin target check (lexicographic comparison).
        // The overwhelming majority of candidates fail here — skip pipeline for those.
        if !target.is_satisfied_by_bytes(&hash) {
            continue;
        }

        // Formal shape certification via the UOR ψ-reduction pipeline.
        // witt_bits = 8: W8 level (Z/(2^8)Z) — one site per byte of the 32-byte hash.
        let block_hash = BlockHash::from_bytes(hash);
        match run_pipeline(&block_hash, 8u16) {
            Ok(grounded) => {
                // Grounded<BlockHash>: formally certified, freeRank = 0.
                // Cannot be fabricated — produced only by run_pipeline.
                return Ok(MiningCertificate::new(grounded, nonce, hash));
            }
            Err(reason) => return Err(MineError::PipelineFailed { nonce, reason }),
        }
    }

    Err(MineError::NonceSpaceExhausted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism_btc_primitives::{Bits, Timestamp, Version};
    use prism_btc_types::MerkleRoot;

    fn genesis_header() -> BlockHeader {
        // Merkle root in Bitcoin internal byte order (reversed from display).
        // Display: 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b
        // Internal: 3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a
        let merkle_bytes: [u8; 32] = [
            0x3b, 0xa3, 0xed, 0xfd, 0x7a, 0x7b, 0x12, 0xb2, 0x7a, 0xc7, 0x2c, 0x3e, 0x67, 0x76,
            0x8f, 0x61, 0x7f, 0xc8, 0x1b, 0xc3, 0x88, 0x8a, 0x51, 0x32, 0x3a, 0x9f, 0xb8, 0xaa,
            0x4b, 0x1e, 0x5e, 0x4a,
        ];
        BlockHeader {
            version: Version(1),
            prev_hash: [0u8; 32],
            merkle_root: MerkleRoot::from_bytes(merkle_bytes),
            timestamp: Timestamp(1231006505),
            bits: Bits(0x1d00ffff),
        }
    }

    #[test]
    #[ignore = "mines full genesis block (~2B nonces) — run in release with: cargo test --release -- --ignored"]
    fn mine_genesis_regression() {
        let header = genesis_header();
        let target = Target::new(Target::GENESIS_NBITS);
        let cert = mine(&header, &target).expect("genesis must be found");

        assert_eq!(cert.nonce(), 2083236893);

        let expected_hash: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0xd6, 0x68, 0x9c, 0x08, 0x5a, 0xe1, 0x65, 0x83,
            0x1e, 0x93, 0x4f, 0xf7, 0x63, 0xae, 0x46, 0xa2, 0xa6, 0xc1, 0x72, 0xb3, 0xf1, 0xb6,
            0x0a, 0x8c, 0xe2, 0x6f,
        ];
        assert_eq!(cert.hash_bytes(), &expected_hash);
    }
}
