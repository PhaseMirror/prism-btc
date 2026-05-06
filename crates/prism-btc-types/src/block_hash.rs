/// A Bitcoin block hash — a 32-byte SHA256d digest.
///
/// Ring: 32 independent W8 sites (Z/(2^8)Z per byte). Total Datum space: 256^32 = 2^256 values.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BlockHash(pub [u8; 32]);

impl BlockHash {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}
