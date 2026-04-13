/// Ring identity verification at the primitive level.
///
/// The core algebraic identity of Z/(2^n)Z:
///   neg(bnot(x)) = succ(x)  i.e.  -(~x) = x + 1
///
/// These tests verify the identity holds for the two Witt levels used
/// by Bitcoin types: W8 (per-byte hash ring) and W32 (nonce ring).
/// The uor! compile-time assertions in prism-btc-types cover the same
/// identity at compile time; these unit tests cover it at runtime.
/// neg(bnot(x)) = succ(x) in Z/(2^8)Z
#[inline(always)]
pub fn ring_identity_u8(x: u8) -> bool {
    let bnot = !x;
    let neg_bnot = bnot.wrapping_neg();
    let succ = x.wrapping_add(1);
    neg_bnot == succ
}

/// neg(bnot(x)) = succ(x) in Z/(2^32)Z
#[inline(always)]
pub fn ring_identity_u32(x: u32) -> bool {
    let bnot = !x;
    let neg_bnot = bnot.wrapping_neg();
    let succ = x.wrapping_add(1);
    neg_bnot == succ
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_identity_u8_exhaustive() {
        for x in 0u8..=255 {
            assert!(ring_identity_u8(x), "identity failed for u8 x={x}");
        }
    }

    #[test]
    fn ring_identity_u32_spot_checks() {
        for x in [0u32, 1, 42, 0xFF, 0x7c2bac1d, u32::MAX - 1, u32::MAX] {
            assert!(ring_identity_u32(x), "identity failed for u32 x={x}");
        }
    }
}
