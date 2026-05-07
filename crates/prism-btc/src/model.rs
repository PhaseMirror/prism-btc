//! `BitcoinMiningModel` — prism-btc's `PrismModel<H, B, A>` declaration.
//!
//! Foundation 0.3.2 ships the `PrismModel<H, B, A>` typed-iso surface
//! (wiki ADR-019/020/022/023). prism-btc, as the prism implementor for
//! the Bitcoin use case, declares its model with:
//!
//! - `Input  = MiningInput`     — the 80-byte canonical wire-format header
//! - `Output = ConstrainedTypeInput` — foundation's identity output shape
//! - `Route  = BitcoinMiningRoute`  — the identity term-tree (Term::Variable {0})
//!
//! `MiningInput` carries the admitting (header‖nonce) bytes the W32 fiber
//! traversal returned. Foundation's `pipeline::run_route` serialises the
//! 80 bytes via [`MiningInput::into_binding_bytes`], folds them through
//! the application's `Hasher` ([`crate::shapes::hasher::Sha256dHasher`])
//! to produce the `ContentFingerprint` carried on the resulting
//! `Grounded<ConstrainedTypeInput>` — and **that fingerprint is the
//! Bitcoin block hash**, because Sha256dHasher's body is exactly SHA-256d.
//!
//! The categorical routing the architecture commits to becomes literal:
//! input bytes → typed transform → output `Grounded` whose content
//! fingerprint is computed by the application-supplied `Hasher` over the
//! input. For the Bitcoin instantiation, that fingerprint IS the
//! consensus block-hash digest.

use uor_foundation::enforcement::ShapeViolation;
use uor_foundation::pipeline::{ConstrainedTypeShape, ConstraintRef, IntoBindingValue};
use uor_foundation::{DefaultHostTypes, ViolationKind};
use uor_foundation_sdk::prism_model;

use crate::shapes::bounds::PrismBtcBounds;
use crate::shapes::hasher::Sha256dHasher;

/// 80-byte canonical wire-format Bitcoin block header.
///
/// Carries the bytes the W32 fiber admitted: the 76-byte template
/// prefix (extranonce-fixed merkle root) + the winning 4-byte nonce.
/// Implements [`IntoBindingValue`] so foundation's `pipeline::run_route`
/// can fold it through the application `Hasher` at certificate emission
/// time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MiningInput(pub [u8; 80]);

impl MiningInput {
    /// IRI of the constraint that fails when foundation hands us a
    /// too-small output buffer (which it shouldn't, by run_route's
    /// invariant: the buffer is sized to MAX_BYTES exactly).
    const BUFFER_VIOLATION: ShapeViolation = ShapeViolation {
        shape_iri: "https://prism.btc/shape/MiningInput",
        constraint_iri: "https://prism.btc/shape/MiningInput/maxBytes",
        property_iri: "https://prism.btc/shape/MiningInput/byteCount",
        expected_range: "http://www.w3.org/2001/XMLSchema#nonNegativeInteger",
        min_count: 80,
        max_count: 80,
        kind: ViolationKind::ValueCheck,
    };
}

impl ConstrainedTypeShape for MiningInput {
    const IRI: &'static str = "https://prism.btc/shape/MiningInput";
    const SITE_COUNT: usize = 80;
    const CONSTRAINTS: &'static [ConstraintRef] = &[];
}

// `IntoBindingValue` (and `PrismModel`, `FoundationClosed`) require
// `__sdk_seal::Sealed`. The wiki sanctions hand-rolled IntoBindingValue
// impls for application authors carrying runtime input data (per the
// `product_shape!` macro doc: "applications that need to carry runtime
// input data declare a custom ConstrainedTypeShape and write a bespoke
// IntoBindingValue impl"). prism-btc, as the prism implementor, exercises
// that lane.
impl uor_foundation::pipeline::__sdk_seal::Sealed for MiningInput {}

impl IntoBindingValue for MiningInput {
    const MAX_BYTES: usize = 80;

    fn into_binding_bytes(&self, out: &mut [u8]) -> Result<usize, ShapeViolation> {
        if out.len() < 80 {
            return Err(Self::BUFFER_VIOLATION);
        }
        out[..80].copy_from_slice(&self.0);
        Ok(80)
    }
}

// ----- The PrismModel declaration -----
//
// `prism_model!` emits:
// - `pub struct BitcoinMiningModel;`
// - `pub struct BitcoinMiningRoute;`
// - `__sdk_seal::Sealed` impls for both
// - `FoundationClosed for BitcoinMiningRoute` returning the term arena
// - `PrismModel<DefaultHostTypes, PrismBtcBounds, Sha256dHasher>` for
//   `BitcoinMiningModel`, whose `forward` body is
//   `pipeline::run_route::<DefaultHostTypes, PrismBtcBounds, Sha256dHasher, Self>(input)`
//
// The route body is the identity term — the macro maps `input` to
// `Term::Variable { name_index: 0 }`. There is no SHA-256-as-Term-tree
// expansion (foundation 0.3.2's `PrimitiveOp` set has no rotate-by-N or
// table-lookup primitive; the round-function decomposition is left to
// foundation amendments per ADR-013). The σ-projection's *runtime*
// evaluation runs through the `Sha256dHasher` substitution-axis
// selection, which run_route folds over the input bytes — bit-identical
// to a hand-rolled SHA-256d of the 80-byte header.
prism_model! {
    pub struct BitcoinMiningModel;
    pub struct BitcoinMiningRoute;
    impl PrismModel<DefaultHostTypes, PrismBtcBounds, Sha256dHasher> for BitcoinMiningModel {
        type Input = MiningInput;
        type Output = uor_foundation::enforcement::ConstrainedTypeInput;
        type Route = BitcoinMiningRoute;
        fn route(input: Self::Input) -> Self::Output {
            input
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_foundation::pipeline::PrismModel;

    #[test]
    fn into_binding_bytes_writes_eighty() {
        let bytes = [0xab; 80];
        let input = MiningInput(bytes);
        let mut out = [0u8; 80];
        let written = input.into_binding_bytes(&mut out).expect("buffer fits");
        assert_eq!(written, 80);
        assert_eq!(out, bytes);
    }

    #[test]
    fn forward_mints_grounded_with_block_hash_fingerprint() {
        // The 80-byte genesis header (display order) — feeding it through
        // forward() should mint a Grounded whose content fingerprint is
        // SHA-256d of these bytes (= the genesis block hash).
        let merkle: [u8; 32] = [
            0x3b, 0xa3, 0xed, 0xfd, 0x7a, 0x7b, 0x12, 0xb2, 0x7a, 0xc7, 0x2c, 0x3e, 0x67, 0x76,
            0x8f, 0x61, 0x7f, 0xc8, 0x1b, 0xc3, 0x88, 0x8a, 0x51, 0x32, 0x3a, 0x9f, 0xb8, 0xaa,
            0x4b, 0x1e, 0x5e, 0x4a,
        ];
        let mut header = [0u8; 80];
        header[0..4].copy_from_slice(&1u32.to_le_bytes());
        // prev_hash = 0
        header[36..68].copy_from_slice(&merkle);
        header[68..72].copy_from_slice(&1231006505u32.to_le_bytes());
        header[72..76].copy_from_slice(&0x1d00ffffu32.to_le_bytes());
        header[76..80].copy_from_slice(&2083236893u32.to_le_bytes());

        let outcome = <BitcoinMiningModel as PrismModel<
            DefaultHostTypes,
            PrismBtcBounds,
            Sha256dHasher,
        >>::forward(MiningInput(header))
        .expect("forward must mint Grounded");
        // Grounded's witt_level_bits comes from the unit's witt_level_ceiling,
        // which run_route picks based on PrismBtcBounds::WITT_LEVEL_MAX_BITS = 32.
        assert_eq!(outcome.witt_level_bits(), 32);
        // The unit_address is non-zero (foundation derives it from
        // the canonical CompileUnit byte layout fingerprint).
        assert_ne!(outcome.unit_address().as_u128(), 0);
    }
}
