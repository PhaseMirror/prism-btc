/// PRISM triadic coordinates for a 32-byte hash value.
///
/// The three independent dimensions of a content-addressed object in UOR space:
/// - `datum`   — identity: the hash bytes themselves
/// - `stratum` — global Hamming weight: count of active (set) bits across all 32 bytes
/// - `spectrum` — non-zero byte mask: bit i is set iff byte i of the hash is non-zero
pub struct TriadicCoords {
    pub datum: [u8; 32],
    pub stratum: u32,
    pub spectrum: u32,
}

impl TriadicCoords {
    /// Compute triadic coordinates from a 32-byte hash.
    pub fn from_hash(hash: &[u8; 32]) -> Self {
        let mut stratum: u32 = 0;
        let mut spectrum: u32 = 0;

        for (i, &byte) in hash.iter().enumerate() {
            stratum += byte.count_ones();
            if byte != 0 {
                spectrum |= 1u32 << i;
            }
        }

        Self {
            datum: *hash,
            stratum,
            spectrum,
        }
    }

    /// Returns true iff these coordinates satisfy the given number of required leading zero bytes.
    ///
    /// `required_zero_bytes` is the minimum number of bytes at datum[0..] that must be 0x00.
    pub fn satisfies_target(&self, required_zero_bytes: u32) -> bool {
        let n = required_zero_bytes as usize;
        if n > 32 {
            return false;
        }
        // The spectrum mask: bits 0..n must all be zero (those bytes must be zero)
        let leading_mask = if n >= 32 {
            u32::MAX
        } else {
            (1u32 << n).wrapping_sub(1)
        };
        (self.spectrum & leading_mask) == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genesis_triadic_coords() {
        // Bitcoin genesis hash: 000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f
        let hash: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0xd6, 0x68, 0x9c, 0x08, 0x5a, 0xe1, 0x65, 0x83,
            0x1e, 0x93, 0x4f, 0xf7, 0x63, 0xae, 0x46, 0xa2, 0xa6, 0xc1, 0x72, 0xb3, 0xf1, 0xb6,
            0x0a, 0x8c, 0xe2, 0x6f,
        ];
        let coords = TriadicCoords::from_hash(&hash);

        // First 5 bytes are zero — bits 0..4 of spectrum must be 0
        assert_eq!(coords.spectrum & 0x1f, 0);

        // 3 required zero bytes (genesis target) — must pass
        assert!(coords.satisfies_target(3));

        // 6 required zero bytes — first 5 are zero, byte[5]=0x19 != 0, so should fail
        assert!(!coords.satisfies_target(6));
    }

    #[test]
    fn stratum_all_ones() {
        let hash = [0xffu8; 32];
        let coords = TriadicCoords::from_hash(&hash);
        assert_eq!(coords.stratum, 256); // 32 bytes × 8 bits
        assert_eq!(coords.spectrum, u32::MAX);
    }

    #[test]
    fn stratum_all_zeros() {
        let hash = [0x00u8; 32];
        let coords = TriadicCoords::from_hash(&hash);
        assert_eq!(coords.stratum, 0);
        assert_eq!(coords.spectrum, 0);
        assert!(coords.satisfies_target(32));
    }
}
