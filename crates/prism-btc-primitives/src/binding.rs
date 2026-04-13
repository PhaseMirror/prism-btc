/// Binds a Rust primitive type to its UOR type vocabulary entry.
///
/// Implementors declare which ring they inhabit (via the ring width in bits)
/// and provide canonical u64 value extraction for ring arithmetic.
pub trait PrimitiveBinding: Sized + Copy {
    /// The ring width in bits (e.g. 8 for u8, 32 for u32, 64 for u64).
    fn witt_bits() -> u16;

    /// Lift this value to its canonical u64 representation for ring ops.
    fn to_u64(self) -> u64;

    /// Recover from a u64 ring value. Returns None if out of range.
    fn from_u64(v: u64) -> Option<Self>;
}

use crate::scalars::{Bits, BlockHeight, FeeRate, Nonce, Satoshi, Timestamp, Version};

impl PrimitiveBinding for Nonce {
    fn witt_bits() -> u16 {
        32
    }
    fn to_u64(self) -> u64 {
        self.0 as u64
    }
    fn from_u64(v: u64) -> Option<Self> {
        u32::try_from(v).ok().map(Nonce)
    }
}

impl PrimitiveBinding for Version {
    fn witt_bits() -> u16 {
        32
    }
    fn to_u64(self) -> u64 {
        self.0 as u64
    }
    fn from_u64(v: u64) -> Option<Self> {
        u32::try_from(v).ok().map(Version)
    }
}

impl PrimitiveBinding for Timestamp {
    fn witt_bits() -> u16 {
        32
    }
    fn to_u64(self) -> u64 {
        self.0 as u64
    }
    fn from_u64(v: u64) -> Option<Self> {
        u32::try_from(v).ok().map(Timestamp)
    }
}

impl PrimitiveBinding for Bits {
    fn witt_bits() -> u16 {
        32
    }
    fn to_u64(self) -> u64 {
        self.0 as u64
    }
    fn from_u64(v: u64) -> Option<Self> {
        u32::try_from(v).ok().map(Bits)
    }
}

impl PrimitiveBinding for Satoshi {
    fn witt_bits() -> u16 {
        64
    }
    fn to_u64(self) -> u64 {
        self.0
    }
    fn from_u64(v: u64) -> Option<Self> {
        Some(Satoshi(v))
    }
}

impl PrimitiveBinding for BlockHeight {
    fn witt_bits() -> u16 {
        64
    }
    fn to_u64(self) -> u64 {
        self.0
    }
    fn from_u64(v: u64) -> Option<Self> {
        Some(BlockHeight(v))
    }
}

impl PrimitiveBinding for FeeRate {
    fn witt_bits() -> u16 {
        64
    }
    fn to_u64(self) -> u64 {
        self.0
    }
    fn from_u64(v: u64) -> Option<Self> {
        Some(FeeRate(v))
    }
}
