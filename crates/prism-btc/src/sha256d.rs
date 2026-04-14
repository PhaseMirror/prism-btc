use sha2::{Digest, Sha256};

/// σ-projection: maps the 80-byte block header to a 32-byte Datum candidate.
///
/// This is NOT the UOR ψ-map. Foundation reserves ψ for the categorical functor
/// chain ψ_1..ψ_9 (Constraints → Nerve → Chain → Homology → … → KInvariants).
/// SHA256d is a non-structure-preserving avalanche hash with no algebraic
/// obligations to satisfy. It is called the σ-projection (ingestion function)
/// to distinguish it from the ψ family.
///
/// Bitcoin stores 256-bit hash integers in little-endian byte order internally
/// but displays them with bytes reversed (most-significant byte first). This
/// function reverses the sha2 output to match display convention so the
/// returned array is lexicographically comparable with the target byte array
/// from `Target::to_bytes()`.
#[inline]
pub(crate) fn sha256d(data: &[u8]) -> [u8; 32] {
    let first = Sha256::digest(data);
    let second = Sha256::digest(first);
    let mut result: [u8; 32] = second.into();
    result.reverse();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256d_empty() {
        // SHA256d(""), result reversed (display format):
        // SHA256(SHA256("")) raw = 5df6e0e2761359d30a8275058e299fcc0381534545f55cf43e41983f5d4c9456
        // Reversed (display): 56944c5d3f9841...
        let result = sha256d(b"");
        // The reversed bytes of 5df6e0e2761359d30a8275058e299fcc0381534545f55cf43e41983f5d4c9456:
        let expected: [u8; 32] = [
            0x56, 0x94, 0x4c, 0x5d, 0x3f, 0x98, 0x41, 0x3e, 0xf4, 0x5c, 0xf5, 0x45, 0x45, 0x53,
            0x81, 0x03, 0xcc, 0x9f, 0x29, 0x8e, 0x05, 0x75, 0x82, 0x0a, 0xd3, 0x59, 0x13, 0x76,
            0xe2, 0xe0, 0xf6, 0x5d,
        ];
        assert_eq!(result, expected);
    }
}
