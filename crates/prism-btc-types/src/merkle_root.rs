use uor_foundation_macros::ConstrainedType;

/// A Bitcoin Merkle root — a 32-byte SHA256d digest of the transaction tree.
///
/// Same ring shape as BlockHash (W8, residue=256) — unconstrained at the type level.
#[derive(Debug, Clone, Default, PartialEq, Eq, ConstrainedType)]
#[uor(residue = 256, hamming = 0)]
pub struct MerkleRoot(pub [u8; 32]);

impl MerkleRoot {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}
