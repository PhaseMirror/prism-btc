//! Bitcoin scalar primitives bound to the UOR ring vocabulary.
//!
//! This crate defines newtype wrappers for every Bitcoin scalar type and maps each
//! to an XSD built-in via the [`PrimitiveBinding`] trait.  It sits at the base of
//! the dependency chain — no macros from `uor-foundation-macros`, no business logic.
//!
//! ## Scalar bindings
//!
//! | Newtype | Inner | XSD type |
//! |---|---|---|
//! | [`Satoshi`] / [`BlockHeight`] | `u64` | `NonNegativeInteger` |
//! | [`Nonce`] / [`Version`] / [`Timestamp`] / [`Bits`] | `u32` | `PositiveInteger` |
//! | [`FeeRate`] | `u64` (sat/kvbyte) | `Decimal` |
//! | [`Address`] | `String` | `String` |
//!
//! ## Ring identity
//!
//! [`ops`] verifies the core Z/(2^n)Z identity `neg(bnot(x)) = x + 1` at both
//! W8 (per-byte hash ring) and W32 (nonce ring) Witt levels.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod binding;
pub mod ops;
pub mod scalars;

pub use binding::PrimitiveBinding;
pub use scalars::{Address, Bits, BlockHeight, FeeRate, Nonce, Satoshi, Timestamp, Version};
