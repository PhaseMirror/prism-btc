use prism_btc_types::BlockHeader;

/// Serialize a block header + nonce to the canonical 80-byte Bitcoin wire format.
///
/// Layout (all multi-byte fields in little-endian):
/// ```text
/// [0..4]   version     (LE u32)
/// [4..36]  prev_hash   (32 bytes, as-is)
/// [36..68] merkle_root (32 bytes, as-is)
/// [68..72] timestamp   (LE u32)
/// [72..76] bits        (LE u32)
/// [76..80] nonce       (LE u32)
/// ```
///
/// This is the wire-byte ingest layout; combined with [`crate::certify_wire_bytes`]
/// it forms the `BinaryGroundingMap` ↔ `BinaryProjectionMap` isomorphism the
/// `Boundary` trait carries at the type level.
pub fn serialize_header(header: &BlockHeader, nonce: u32) -> [u8; 80] {
    let mut buf = [0u8; 80];

    buf[0..4].copy_from_slice(&header.version.0.to_le_bytes());
    buf[4..36].copy_from_slice(&header.prev_hash);
    buf[36..68].copy_from_slice(header.merkle_root.as_bytes());
    buf[68..72].copy_from_slice(&header.timestamp.0.to_le_bytes());
    buf[72..76].copy_from_slice(&header.bits.0.to_le_bytes());
    buf[76..80].copy_from_slice(&nonce.to_le_bytes());

    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism_btc_types::{Bits, MerkleRoot, Timestamp, Version};

    // Merkle root in Bitcoin internal byte order (reversed from display).
    // Display: 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b
    // Internal: 3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a
    const GENESIS_MERKLE: [u8; 32] = [
        0x3b, 0xa3, 0xed, 0xfd, 0x7a, 0x7b, 0x12, 0xb2, 0x7a, 0xc7, 0x2c, 0x3e, 0x67, 0x76, 0x8f,
        0x61, 0x7f, 0xc8, 0x1b, 0xc3, 0x88, 0x8a, 0x51, 0x32, 0x3a, 0x9f, 0xb8, 0xaa, 0x4b, 0x1e,
        0x5e, 0x4a,
    ];

    fn genesis_header() -> BlockHeader {
        BlockHeader {
            version: Version(1),
            prev_hash: [0u8; 32],
            merkle_root: MerkleRoot::from_bytes(GENESIS_MERKLE),
            timestamp: Timestamp(1231006505),
            bits: Bits(0x1d00ffff),
        }
    }

    #[test]
    fn serialize_header_length_80() {
        let buf = serialize_header(&genesis_header(), 0);
        assert_eq!(buf.len(), 80);
    }

    #[test]
    fn serialize_header_genesis_bytes() {
        // Genesis nonce: 2083236893 = 0x7c2bac1d
        let buf = serialize_header(&genesis_header(), 2083236893);

        assert_eq!(&buf[0..4], &[0x01, 0x00, 0x00, 0x00]);
        assert_eq!(&buf[4..36], &[0u8; 32]);
        assert_eq!(&buf[36..68], &GENESIS_MERKLE);
        assert_eq!(&buf[68..72], &1231006505u32.to_le_bytes());
        assert_eq!(&buf[72..76], &0x1d00ffffu32.to_le_bytes());
        assert_eq!(&buf[76..80], &2083236893u32.to_le_bytes());
    }
}
