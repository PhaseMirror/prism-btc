//! Minimal 128-bit FNV-1a hasher used for content-addressing UOR CompileUnits.
//!
//! This is NOT the Bitcoin σ-projection (that remains SHA256d in `prism-btc::sha256d`).
//! `Fnv1aHasher16` is the substrate hasher consumed by `uor_foundation::pipeline::run`
//! and `pipeline::run_const` to fold the CompileUnit's canonical byte layout into a
//! deterministic 16-byte content fingerprint.

use uor_foundation::enforcement::Hasher;
use uor_foundation::{DefaultHostBounds, HostBounds};

const FP_MAX: usize = <DefaultHostBounds as HostBounds>::FINGERPRINT_MAX_BYTES;

/// Two-state FNV-1a producing a 16-byte content fingerprint.
///
/// Layout: the first 8 bytes of `finalize()`'s output hold the `a` state
/// (big-endian); the next 8 bytes hold the `b` state; the remaining
/// `FP_MAX - 16` bytes are zero, where `FP_MAX` is the
/// `DefaultHostBounds::FINGERPRINT_MAX_BYTES` capacity (32).
#[derive(Debug, Clone, Copy)]
pub struct Fnv1aHasher16 {
    a: u64,
    b: u64,
}

const FNV_OFFSET_BASIS_A: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_OFFSET_BASIS_B: u64 = 0x8422_2325_cbf2_9ce4;
const FNV_PRIME: u64 = 0x100_0000_01b3;

impl Hasher for Fnv1aHasher16 {
    const OUTPUT_BYTES: usize = 16;

    fn initial() -> Self {
        Self {
            a: FNV_OFFSET_BASIS_A,
            b: FNV_OFFSET_BASIS_B,
        }
    }

    fn fold_byte(mut self, x: u8) -> Self {
        self.a ^= x as u64;
        self.a = self.a.wrapping_mul(FNV_PRIME);
        self.b ^= (x as u64).rotate_left(8);
        self.b = self.b.wrapping_mul(FNV_PRIME);
        self
    }

    fn finalize(self) -> [u8; FP_MAX] {
        let mut buf = [0u8; FP_MAX];
        buf[..8].copy_from_slice(&self.a.to_be_bytes());
        buf[8..16].copy_from_slice(&self.b.to_be_bytes());
        buf
    }
}
