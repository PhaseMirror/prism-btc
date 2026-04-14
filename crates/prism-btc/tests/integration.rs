use prism_btc::{
    genesis, BlockCertificate, BlockHeader, Boundary, BoundaryDecodeError, MerkleRoot, MiningRound,
    Target,
};
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
fn mine_converges_and_satisfies_target() {
    let header = genesis_header();
    // 0x207fffff: very easy target; converges in < 1ms in debug mode.
    // Convergence termination is formally proven in prism-btc-lean/PrismBtc/ConvergenceProtocol.lean.
    let target = Target::new(0x207fffff);
    let cert = MiningRound::new(header, target)
        .converge()
        .expect("easy target must converge");

    // Returned hash must satisfy the target constraint.
    assert!(target.is_satisfied_by_bytes(&cert.coords().datum));
    // Triadic coordinates must be populated.
    assert_ne!(cert.coords().datum, [0u8; 32], "datum must be non-zero");
    // Easy target (0x207fffff) requires at least one leading zero byte →
    // bit 0 of spectrum (which counts leading-zero bytes) must be set.
    assert_eq!(
        cert.coords().spectrum & 1,
        1,
        "easy target produces at least one leading zero byte"
    );
}

#[test]
fn genesis_grounded_constant_is_certified() {
    // genesis() runs uor_ground! at call time.
    // Verify the Grounded<BlockHash> carries non-trivial certification metadata.
    let grounded_const = genesis();
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

#[test]
fn boundary_decode_rejects_wrong_length() {
    let short = [0u8; 79];
    let result = BlockCertificate::decode(&short);
    assert!(
        matches!(result, Err(BoundaryDecodeError::InvalidLength { got: 79 })),
        "decode of 79 bytes must return InvalidLength {{ got: 79 }}"
    );

    let long = [0u8; 81];
    let result = BlockCertificate::decode(&long);
    assert!(
        matches!(result, Err(BoundaryDecodeError::InvalidLength { got: 81 })),
        "decode of 81 bytes must return InvalidLength {{ got: 81 }}"
    );
}
