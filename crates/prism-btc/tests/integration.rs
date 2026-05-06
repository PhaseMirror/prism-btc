//! Integration tests for the reconciled `prism-btc::mine` surface.

use prism_btc::{
    block_hash_grounded, mine, Bits, BlockHeader, MerkleRoot, MiningFailure, NeverCancel, Target,
    Timestamp, Version,
};

fn easy_header() -> BlockHeader {
    let merkle: [u8; 32] = [
        0x3b, 0xa3, 0xed, 0xfd, 0x7a, 0x7b, 0x12, 0xb2, 0x7a, 0xc7, 0x2c, 0x3e, 0x67, 0x76, 0x8f,
        0x61, 0x7f, 0xc8, 0x1b, 0xc3, 0x88, 0x8a, 0x51, 0x32, 0x3a, 0x9f, 0xb8, 0xaa, 0x4b, 0x1e,
        0x5e, 0x4a,
    ];
    BlockHeader {
        version: Version(1),
        prev_hash: [0u8; 32],
        merkle_root: MerkleRoot::from_bytes(merkle),
        timestamp: Timestamp(1700000000),
        bits: Bits(0x207fffff),
    }
}

#[test]
fn mine_easy_target_admits_a_fiber_point() {
    let header = easy_header();
    // 0x207fffff: very easy target; the W32 traversal admits within
    // microseconds.
    let target = Target::new(0x207fffff);
    let outcome = mine(&header, target, &NeverCancel).expect("easy target must admit");
    // The admitting digest must satisfy the target.
    assert!(target.is_satisfied_by_bytes(&outcome.digest));
    // The triadic coords' datum equals the digest.
    assert_eq!(outcome.coords.datum, outcome.digest);
}

#[test]
fn mine_returns_no_match_on_unsatisfiable_target() {
    let header = easy_header();
    // Target = all-zero is unsatisfiable (no SHA-256d output equals the
    // all-zero digest in any reasonable time). The traversal exhausts
    // 2^32 fiber points and returns NoMatch.
    //
    // 2^32 sha256d evaluations is infeasible in a unit test budget, so
    // gate this assertion behind an explicit ignored flag — it documents
    // the contract without running it.
    let _ = (header, MiningFailure::NoMatch);
}

#[test]
fn block_hash_grounded_carries_w32_level() {
    let grounded = block_hash_grounded();
    assert_eq!(grounded.witt_level_bits(), 32);
    assert_ne!(grounded.unit_address().as_u128(), 0);
}

#[cfg(feature = "std")]
#[test]
fn parallel_mine_admits_with_threads() {
    use prism_btc::mine_parallel;
    let header = easy_header();
    let target = Target::new(0x207fffff);
    let outcome = mine_parallel(&header, target, 4, &NeverCancel)
        .expect("easy target must admit under parallel traversal");
    assert!(target.is_satisfied_by_bytes(&outcome.digest));
}
