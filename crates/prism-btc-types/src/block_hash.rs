use uor_foundation_macros::ConstrainedType;

/// A Bitcoin block hash — a 32-byte SHA256d digest.
///
/// The ring is W8 (Z/(2^8)Z per byte site), with a residue constraint of 256 (mod 256 = 0).
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
