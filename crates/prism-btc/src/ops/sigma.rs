//! The σ-projection: the runtime that maps `(template_prefix, nonce)` to
//! a 32-byte block-hash digest in Bitcoin display order.
//!
//! Runtime: pure-Rust SHA-256d (see [`crate::ops::sha256`]) over the
//! 80-byte canonical header layout, with the result reversed to display
//! order so it can be lexicographically compared against a target.
//!
//! Structural identity: declared in [`crate::ops::term`] as a
//! `Term::Application` chain `Sha256dProjection ∘ HeaderSerialization`.
//! prism-btc, as the prism implementor for the Bitcoin use case,
//! provides this evaluator; foundation provides the type vocabulary.

use crate::domain::BlockHeader;
use crate::ops::header::serialize_header;
use crate::ops::sha256::sha256d_display;

/// σ-projection: full 80-byte header serialisation followed by SHA-256d
/// in Bitcoin display order. This is the function the W32 fiber
/// traversal evaluates at every fiber point.
#[inline]
pub fn sigma_project(header: &BlockHeader, nonce: u32) -> [u8; 32] {
    sha256d_display(&serialize_header(header, nonce))
}

/// Variant operating on a pre-serialised 76-byte prefix, splicing the
/// nonce into bytes [76..80) without rebuilding the full prefix on
/// every call. Hot-path use case: the W32 traversal computes the
/// prefix once per (template, extranonce) pair and reuses it across
/// fiber visits.
#[inline]
pub fn sigma_project_prefix(prefix: &[u8; 76], nonce: u32) -> [u8; 32] {
    let mut block = [0u8; 80];
    block[..76].copy_from_slice(prefix);
    block[76..80].copy_from_slice(&nonce.to_le_bytes());
    sha256d_display(&block)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Bits, BlockHeader, MerkleRoot, Timestamp, Version};

    fn genesis_header() -> BlockHeader {
        let merkle: [u8; 32] = [
            0x3b, 0xa3, 0xed, 0xfd, 0x7a, 0x7b, 0x12, 0xb2, 0x7a, 0xc7, 0x2c, 0x3e, 0x67, 0x76,
            0x8f, 0x61, 0x7f, 0xc8, 0x1b, 0xc3, 0x88, 0x8a, 0x51, 0x32, 0x3a, 0x9f, 0xb8, 0xaa,
            0x4b, 0x1e, 0x5e, 0x4a,
        ];
        BlockHeader {
            version: Version(1),
            prev_hash: [0u8; 32],
            merkle_root: MerkleRoot::from_bytes(merkle),
            timestamp: Timestamp(1231006505),
            bits: Bits(0x1d00ffff),
        }
    }

    #[test]
    fn sigma_at_genesis_nonce_yields_genesis_hash() {
        // Bitcoin's genesis block: nonce 2083236893 ⇒
        //   hash 000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f
        // (display order)
        let got = sigma_project(&genesis_header(), 2083236893);
        let expected: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0xd6, 0x68, 0x9c, 0x08, 0x5a, 0xe1, 0x65, 0x83,
            0x1e, 0x93, 0x4f, 0xf7, 0x63, 0xae, 0x46, 0xa2, 0xa6, 0xc1, 0x72, 0xb3, 0xf1, 0xb6,
            0x0a, 0x8c, 0xe2, 0x6f,
        ];
        assert_eq!(got, expected);
    }

    #[test]
    fn sigma_full_equals_prefix_variant() {
        use crate::ops::header::serialize_prefix;
        let h = genesis_header();
        let prefix = serialize_prefix(&h);
        for &n in &[0u32, 1, 0xdeadbeef, u32::MAX] {
            assert_eq!(sigma_project(&h, n), sigma_project_prefix(&prefix, n));
        }
    }
}
