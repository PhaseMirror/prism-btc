use prism_btc_reduction::{
    run_convergence, serialize_header, sha256d, BlockCertificate, ConvergenceFailure,
};
use prism_btc_types::{BlockHeader, Target};
use uor_foundation::enforcement::DigestProjectionMap;

/// Mine a block header by running the σ-convergence loop.
///
/// Bitcoin's σ-projection is SHA256d-of-(header‖nonce); the matching morphism
/// kind is `DigestProjectionMap` (`Total`, neither `Invertible` nor
/// `PreservesStructure`). The returned certificate carries that kind at the
/// type level via its `Sigma` parameter.
pub(crate) fn mine(
    header: &BlockHeader,
    target: &Target,
) -> Result<BlockCertificate<DigestProjectionMap>, ConvergenceFailure> {
    let target_bytes = target.to_bytes();
    run_convergence::<DigestProjectionMap, _>(header.clone(), target_bytes, |nonce| {
        sha256d(&serialize_header(header, nonce))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism_btc_types::{Bits, MerkleRoot, Timestamp, Version};

    fn genesis_header() -> BlockHeader {
        // Merkle root in Bitcoin internal byte order (reversed from display).
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
        let target = Target::new(0x207fffff);
        let cert = MiningRound::new(header, target)
            .converge()
            .expect("easy target must converge");
        assert!(target.is_satisfied_by_bytes(cert.digest()));
    }
}
