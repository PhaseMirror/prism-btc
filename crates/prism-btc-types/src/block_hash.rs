use uor_foundation_macros::ConstrainedType;

/// The Witt reference level for each byte site of a `BlockHash`.
///
/// A `BlockHash` is a **32-tuple of W8 elements** — 32 independent sites of Z/(2^8)Z.
/// This is NOT W256 (a single element of Z/(2^256)Z); it is the per-site W8 ring
/// matching the `residue = 256` annotation (2^8 = 256 elements per site).
///
/// Passed to `run_pipeline` to name the per-site Witt level explicitly — it is the W8
/// reference level for certification proofs, not a stage count.
pub const BLOCK_HASH_SITE_WITT_BITS: u16 = 8; // W8

/// A Bitcoin block hash — a 32-byte SHA256d digest.
///
/// Ring: 32 independent W8 sites (Z/(2^8)Z per byte). Total Datum space: 256^32 = 2^256 values.
///
/// The `residue = 256` annotation encodes Z/(2^8)Z per site (2^8 = 256 elements per site).
/// The `hamming = 0` annotation marks this type as unconstrained at the type level —
/// the actual Hamming constraint (leading zero bytes) is checked per-candidate by Target.
#[derive(Debug, Clone, Default, PartialEq, Eq, ConstrainedType)]
#[uor(residue = 256, hamming = 0)]
pub struct BlockHash(pub [u8; 32]);

impl BlockHash {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}
