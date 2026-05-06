use crate::block_hash::BlockHash;

/// Compact nBits encoding of the Bitcoin proof-of-work target.
///
/// The ring is W32 (Z/(2^32)Z). The Bitcoin target check (hash ≤ target) is
/// enforced by `is_satisfied_by_bytes` directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Target {
    pub nbits: u32,
}

impl Target {
    /// Genesis block nBits: `0x1d00ffff`
    pub const GENESIS_NBITS: u32 = 0x1d00ffff;

    pub const fn new(nbits: u32) -> Self {
        Self { nbits }
    }

    /// Decode nBits to a 32-byte big-endian target value.
    ///
    /// The nBits encoding: top byte is the exponent (byte length of the mantissa),
    /// lower 3 bytes are the mantissa (signed, but Bitcoin treats as unsigned here).
    pub fn to_bytes(&self) -> [u8; 32] {
        let nbits = self.nbits;
        let exp = (nbits >> 24) as usize;
        let mantissa = nbits & 0x00ff_ffff;

        let mut target = [0u8; 32];
        if exp == 0 || exp > 32 {
            return target;
        }

        // Place the 3-byte mantissa at byte offset (32 - exp) through (32 - exp + 2)
        // The most significant byte of the mantissa goes at (32 - exp)
        let start = 32usize.saturating_sub(exp);
        let m2 = ((mantissa >> 16) & 0xff) as u8;
        let m1 = ((mantissa >> 8) & 0xff) as u8;
        let m0 = (mantissa & 0xff) as u8;

        if start < 32 {
            target[start] = m2;
        }
        if start + 1 < 32 {
            target[start + 1] = m1;
        }
        if start + 2 < 32 {
            target[start + 2] = m0;
        }

        target
    }

    /// Returns the minimum number of leading zero bytes required in a valid block hash.
    ///
    /// This is a conservative lower bound derived from the nBits exponent: the number of
    /// fully-zero leading bytes guaranteed by the target constraint.
    pub fn leading_zero_bytes(&self) -> u32 {
        let exp = (self.nbits >> 24) as usize;
        // exp = byte-length of the value; so (32 - exp) bytes are definitely zero
        if exp >= 32 {
            0
        } else {
            (32 - exp) as u32
        }
    }

    /// Returns true iff `hash` satisfies this target (hash ≤ target, big-endian).
    pub fn is_satisfied_by(&self, hash: &BlockHash) -> bool {
        self.is_satisfied_by_bytes(hash.as_bytes())
    }

    /// Returns true iff `hash` satisfies this target (hash ≤ target, big-endian).
    ///
    /// This is the hot-path check called from the mining loop.
    #[inline]
    pub fn is_satisfied_by_bytes(&self, hash: &[u8; 32]) -> bool {
        hash <= &self.to_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genesis_nbits_leading_zeros() {
        let t = Target::new(Target::GENESIS_NBITS);
        // 0x1d00ffff: exp = 0x1d = 29, so 32 - 29 = 3 leading zero bytes
        assert_eq!(t.leading_zero_bytes(), 3);
    }

    #[test]
    fn genesis_target_bytes() {
        let t = Target::new(Target::GENESIS_NBITS);
        let bytes = t.to_bytes();
        // First 3 bytes must be zero
        assert_eq!(bytes[0], 0);
        assert_eq!(bytes[1], 0);
        assert_eq!(bytes[2], 0);
        // Mantissa 0x00ffff at bytes[3..6]: 0x00, 0xff, 0xff
        assert_eq!(bytes[3], 0x00);
        assert_eq!(bytes[4], 0xff);
        assert_eq!(bytes[5], 0xff);
    }

    #[test]
    fn target_satisfaction() {
        let t = Target::new(Target::GENESIS_NBITS);
        // Genesis hash: 000000000019d668...
        let genesis_hash: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0xd6, 0x68, 0x9c, 0x08, 0x5a, 0xe1, 0x65, 0x83,
            0x1e, 0x93, 0x4f, 0xf7, 0x63, 0xae, 0x46, 0xa2, 0xa6, 0xc1, 0x72, 0xb3, 0xf1, 0xb6,
            0x0a, 0x8c, 0xe2, 0x6f,
        ];
        assert!(t.is_satisfied_by_bytes(&genesis_hash));
    }

    #[test]
    fn target_satisfaction_boundary() {
        let t = Target::new(Target::GENESIS_NBITS);
        let target_bytes = t.to_bytes();

        // Hash equal to target must satisfy (≤)
        assert!(t.is_satisfied_by_bytes(&target_bytes));

        // Hash one above target must not satisfy.
        // Increment byte 6 (which is 0x00 in genesis target → 0x01) to produce a
        // strictly-greater hash without triggering byte-wrap carry propagation.
        let mut above = target_bytes;
        above[6] = 0x01;
        assert!(!t.is_satisfied_by_bytes(&above));
    }

    #[test]
    fn to_bytes_exp_zero_returns_all_zeros() {
        // exp = 0 → exponent field is 0x00; to_bytes must return [0u8; 32]
        let t = Target::new(0x00_ffffff);
        assert_eq!(t.to_bytes(), [0u8; 32]);
    }

    #[test]
    fn to_bytes_exp_overflow_returns_all_zeros() {
        // exp = 33 > 32 → overflow guard; to_bytes must return [0u8; 32]
        let t = Target::new(0x21_000000);
        assert_eq!(t.to_bytes(), [0u8; 32]);
    }

    #[test]
    fn to_bytes_zero_mantissa() {
        // 0x1d_000000: exp=29, mantissa=0 → all bytes zero
        let t = Target::new(0x1d_000000);
        assert_eq!(t.to_bytes(), [0u8; 32]);
    }
}
