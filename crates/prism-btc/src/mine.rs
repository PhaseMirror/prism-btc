use crate::serialize::serialize_header;
use crate::sha256d::sha256d;
use prism_btc_reduction::{BlockCertificate, ConvergenceFailure};
use prism_btc_types::{BlockHeader, Target};

/// Mine a block header by running the σ-convergence loop.
///
/// **σ-convergence loop:**
/// 1. Serialize the 80-byte header with the candidate nonce
/// 2. σ-projection (SHA256d): 80-byte header → 32-byte candidate Datum
/// 3. Target pre-filter: lexicographic target check rejects most candidates
/// 4. `run_pipeline`: formal shape certification — runs pipeline reduction
///    over BlockHash's ConstrainedTypeShape constraints (W8 per-byte, 32 sites)
/// 5. On success, wrap the un-fabricatable `Grounded<BlockHash>` in a `BlockCertificate`
///
/// SHA256d is the σ-projection (ingestion hash), NOT the UOR ψ-map. Foundation reserves
/// ψ for the categorical functor chain ψ_1..ψ_9.
///
/// This function is `pub(crate)` — callers use `MiningRound::converge()`, which carries
/// the `#[uor_grounded(level = "W32")]` Witt assertion via `converge_at_w32()`.
pub(crate) fn mine(
    header: &BlockHeader,
    target: &Target,
) -> Result<BlockCertificate, ConvergenceFailure> {
    let target_bytes = target.to_bytes();
    let header_clone = header.clone();
    prism_btc_reduction::run_convergence(header.clone(), target_bytes, move |nonce| {
        let raw = serialize_header(&header_clone, nonce);
        sha256d(&raw)
    })
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
    fn mine_easy_target() {
        use crate::MiningRound;
        let header = genesis_header();
        // 0x207fffff: very easy target; converges in < 1ms in debug mode.
        // Convergence termination is formally proven in prism-btc-lean/PrismBtc/ConvergenceProtocol.lean.
        let target = Target::new(0x207fffff);
        let cert = MiningRound::new(header, target)
            .converge()
            .expect("easy target must converge");
        // Verify the returned hash satisfies the target.
        assert!(target.is_satisfied_by_bytes(&cert.coords().datum));
    }
}
