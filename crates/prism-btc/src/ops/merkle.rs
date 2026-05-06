//! `MerkleRootDerivation` — pairwise SHA-256d up the transaction tree.
//!
//! Given a coinbase txid and a list of other transaction txids
//! (in template order), derives the 32-byte merkle root by recursive
//! pairwise SHA-256d, duplicating the last element when a level has
//! odd parity.
//!
//! Runtime: prism-btc's pure-Rust SHA-256d (no dependency on the
//! `bitcoin` crate's hashing). Bitcoin txids are stored in *internal*
//! byte order (least-significant byte first); the merkle pairing
//! concatenates internal-order bytes and applies SHA-256d, also in
//! internal byte order.

use crate::ops::sha256::sha256d_internal;

/// Compute the merkle root of `[coinbase_txid, *other_txids]` in
/// internal byte order. The root is what gets placed in
/// `BlockHeader.merkle_root`.
pub fn merkle_root_internal(coinbase_txid: &[u8; 32], other_txids: &[[u8; 32]]) -> [u8; 32] {
    let mut layer: Vec<[u8; 32]> = Vec::with_capacity(other_txids.len() + 1);
    layer.push(*coinbase_txid);
    layer.extend_from_slice(other_txids);

    while layer.len() > 1 {
        if layer.len() % 2 == 1 {
            // Bitcoin merkle: duplicate the last element if odd parity.
            let last = *layer.last().expect("non-empty by loop guard");
            layer.push(last);
        }
        let mut next: Vec<[u8; 32]> = Vec::with_capacity(layer.len() / 2);
        let mut i = 0;
        while i < layer.len() {
            let mut pair = [0u8; 64];
            pair[..32].copy_from_slice(&layer[i]);
            pair[32..].copy_from_slice(&layer[i + 1]);
            next.push(sha256d_internal(&pair));
            i += 2;
        }
        layer = next;
    }

    layer[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merkle_of_single_coinbase_is_the_coinbase_txid() {
        let coinbase: [u8; 32] = [0x42; 32];
        let root = merkle_root_internal(&coinbase, &[]);
        assert_eq!(root, coinbase);
    }

    #[test]
    fn merkle_pair_matches_manual_sha256d() {
        let cb: [u8; 32] = [0xaa; 32];
        let other: [u8; 32] = [0xbb; 32];
        let mut concat = [0u8; 64];
        concat[..32].copy_from_slice(&cb);
        concat[32..].copy_from_slice(&other);
        let expected = sha256d_internal(&concat);
        let root = merkle_root_internal(&cb, &[other]);
        assert_eq!(root, expected);
    }

    #[test]
    fn merkle_three_leaves_duplicates_last() {
        // 3 txids → at level 0 [a, b, c] → odd, duplicate c → [a, b, c, c]
        // → level 1 [d2(a||b), d2(c||c)]
        // → level 2 [d2(level1[0] || level1[1])]
        let a: [u8; 32] = [0x01; 32];
        let b: [u8; 32] = [0x02; 32];
        let c: [u8; 32] = [0x03; 32];
        let mut ab = [0u8; 64];
        ab[..32].copy_from_slice(&a);
        ab[32..].copy_from_slice(&b);
        let l1_0 = sha256d_internal(&ab);
        let mut cc = [0u8; 64];
        cc[..32].copy_from_slice(&c);
        cc[32..].copy_from_slice(&c);
        let l1_1 = sha256d_internal(&cc);
        let mut top = [0u8; 64];
        top[..32].copy_from_slice(&l1_0);
        top[32..].copy_from_slice(&l1_1);
        let expected = sha256d_internal(&top);

        let root = merkle_root_internal(&a, &[b, c]);
        assert_eq!(root, expected);
    }
}
