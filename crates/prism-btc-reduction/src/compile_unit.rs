//! The single CompileUnit that certifies the `BlockHash` shape.
//!
//! There is exactly one shape certificate in prism-btc: the BlockHash shape
//! (32 W8 sites, W32 ceiling, ComposedAlgebraic verification domain). That
//! certificate is identical for genesis, every nonce candidate within a
//! mining round, and every wire-decoded header. The CompileUnit is validated
//! at compile time via `validate_compile_unit_const`; `run_const` folds the
//! Fnv1aHasher16 substrate over the canonical byte layout to produce the
//! [`BlockHashGrounded`] at call time.
//!
//! `BinaryGroundingMap` is the morphism kind of the CompileUnit's own
//! ingestion (raw bytes → grounded shape). It is `Total + Invertible` —
//! the bound `pipeline::run_const` requires.

use prism_btc_types::{BlockHashGrounded, BlockHashTag};
use uor_foundation::enforcement::{
    BinaryGroundingMap, CompileTime, CompileUnit, CompileUnitBuilder, ConstrainedTypeInput, Term,
    Validated,
};
use uor_foundation::pipeline::{run_const, validate_compile_unit_const};
use uor_foundation::{VerificationDomain, WittLevel};

use crate::hasher::Fnv1aHasher16;

static TERMS: &[Term] = &[Term::Literal {
    value: 0,
    level: WittLevel::W8,
}];

static DOMAINS: &[VerificationDomain] = &[VerificationDomain::ComposedAlgebraic];

const BUILDER: CompileUnitBuilder<'static> = CompileUnitBuilder::new()
    .root_term(TERMS)
    .witt_level_ceiling(WittLevel::W32)
    .thermodynamic_budget(4_294_967_295_u64)
    .target_domains(DOMAINS)
    .result_type::<ConstrainedTypeInput>();

/// The const-validated BlockHash CompileUnit. Baked at compile time via
/// `validate_compile_unit_const`. Panics at compile time if the unit is
/// malformed — there is no runtime path that observes this constant.
pub(crate) const BLOCK_HASH_UNIT: Validated<CompileUnit<'static>, CompileTime> =
    match validate_compile_unit_const(&BUILDER) {
        Ok(unit) => unit,
        Err(_) => panic!("BlockHash CompileUnit must validate at compile time"),
    };

// W32 is the nonce ring — anchor it at compile time.
const _: () = assert!(WittLevel::W32.witt_length() == 32);

/// Run the BlockHash CompileUnit through `pipeline::run_const` and tag the
/// result with `BlockHashTag`. Used by genesis (one-shot) and by the mining
/// loop (once per round, before the nonce iteration).
///
/// The morphism kind is `BinaryGroundingMap` — the byte ingest from the
/// canonical CompileUnit layout into the grounded shape. This is the
/// well-formedness certificate; the σ-projection (header‖nonce → 32-byte
/// digest) is a separate `DigestProjectionMap` carried at the
/// `BlockCertificate` type level.
///
/// **Infallible.** `pipeline::run_const`'s only failure mode is
/// `PipelineFailure::ShapeMismatch` between the unit's declared
/// `result_type_iri` and the caller's `T::IRI`. Both are pinned to
/// `ConstrainedTypeInput` here, so the check always passes — and the
/// CompileUnit is `const`-validated, so any structural malformation would
/// have triggered a compile-time panic. We preserve that infallibility in
/// the public signature.
#[must_use]
pub fn block_hash_shape_certificate() -> BlockHashGrounded {
    let grounded =
        run_const::<ConstrainedTypeInput, BinaryGroundingMap, Fnv1aHasher16>(&BLOCK_HASH_UNIT)
            .expect(
                "BlockHash CompileUnit is const-validated and the result-type IRI matches by \
                 construction; run_const cannot fail",
            );
    grounded.tag::<BlockHashTag>()
}
