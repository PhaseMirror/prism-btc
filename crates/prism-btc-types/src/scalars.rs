//! Block-header scalar newtypes.
//!
//! Three u32 fields appear in a Bitcoin block header alongside the two 32-byte
//! hashes; the newtypes prevent accidental swaps in struct-literal construction.

/// Block version — XSD PositiveInteger → u32.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Version(pub u32);

/// Block timestamp (Unix epoch) — XSD PositiveInteger → u32.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Timestamp(pub u32);

/// Compact nBits target encoding — XSD PositiveInteger → u32.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Bits(pub u32);
