use prism_btc::{genesis_block_hash, mine, BlockHeader, MerkleRoot, Target};
use prism_btc_primitives::{Bits, Timestamp, Version};

fn genesis_header() -> BlockHeader {
    // Merkle root in Bitcoin internal byte order (reversed from display).
    // Display: 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b
    // Internal: 3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a
    let merkle_bytes: [u8; 32] = [
        0x3b, 0xa3, 0xed, 0xfd, 0x7a, 0x7b, 0x12, 0xb2, 0x7a, 0xc7, 0x2c, 0x3e, 0x67, 0x76, 0x8f,
        0x61, 0x7f, 0xc8, 0x1b, 0xc3, 0x88, 0x8a, 0x51, 0x32, 0x3a, 0x9f, 0xb8, 0xaa, 0x4b, 0x1e,
        0x5e, 0x4a,
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
fn mine_genesis_block_integration() {
    let header = genesis_header();
    let target = Target::new(Target::GENESIS_NBITS);
    let cert = mine(&header, &target).expect("genesis must be found");

    assert_eq!(cert.nonce(), 2083236893, "wrong nonce");

    // Expected hash in display (big-endian) format — sha256d returns reversed bytes
    let expected: [u8; 32] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0xd6, 0x68, 0x9c, 0x08, 0x5a, 0xe1, 0x65, 0x83, 0x1e,
        0x93, 0x4f, 0xf7, 0x63, 0xae, 0x46, 0xa2, 0xa6, 0xc1, 0x72, 0xb3, 0xf1, 0xb6, 0x0a, 0x8c,
        0xe2, 0x6f,
    ];
    assert_eq!(cert.hash_bytes(), &expected, "wrong hash");

    // Triadic coordinates of the genesis hash:
    // First 5 bytes are 0x00 — bits 0..4 of spectrum must be clear.
    assert_eq!(
        cert.spectrum() & 0x1f,
        0,
        "spectrum: first 5 bytes must be zero"
    );
    // Genesis hash has many set bits after byte 5 — stratum must be non-zero.
    assert!(cert.stratum() > 0, "stratum must be non-zero");
}

#[test]
fn genesis_grounded_constant_is_certified() {
    // The genesis_grounded() function runs uor_ground! at call time.
    // Verify the Grounded<BlockHash> carries non-trivial certification metadata.
    let grounded_const = genesis_block_hash();
    // unit_address is a content-addressed FNV-1a hash of BlockHash's type IRI +
    // constraint list — non-zero for any non-trivial constraint system.
    assert_ne!(
        grounded_const.unit_address(),
        0,
        "grounded unit_address must be non-zero"
    );
    // W32 Witt level was requested in the uor_ground! macro and propagated.
    assert_eq!(
        grounded_const.witt_level_bits(),
        32,
        "W32 level must propagate"
    );
}
