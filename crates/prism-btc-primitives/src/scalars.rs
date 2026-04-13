#[cfg(not(feature = "std"))]
use alloc::string::String;

/// Satoshi amounts — XSD NonNegativeInteger → u64
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Satoshi(pub u64);

/// Block heights — XSD NonNegativeInteger → u64
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockHeight(pub u64);

/// Fee rate in sat/kvbyte (scaled integer) — XSD Decimal → u64
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FeeRate(pub u64);

/// Nonce — XSD PositiveInteger → u32
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Nonce(pub u32);

/// Block version — XSD PositiveInteger → u32
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Version(pub u32);

/// Block timestamp (Unix epoch) — XSD PositiveInteger → u32
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Timestamp(pub u32);

/// Compact nBits target encoding — XSD PositiveInteger → u32
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Bits(pub u32);

/// Bitcoin address (bech32 / base58check) — XSD String
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Address(pub String);
