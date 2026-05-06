/// A Bitcoin Merkle root — a 32-byte SHA256d digest of the transaction tree.
///
/// Same ring shape as `BlockHash`: 32 independent W8 sites.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MerkleRoot(pub [u8; 32]);

impl MerkleRoot {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}
