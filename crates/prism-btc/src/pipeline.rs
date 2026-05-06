//! prism-btc's mining pipeline: traverses the W32 fiber over a (prefix,
//! target) pair, mints the foundation-sealed shape `Grounded` on
//! admission, returns the (witness, nonce, digest).
//!
//! The architecture's `mine()` entry point. Foundation provides the
//! sealed `Grounded` mint via `pipeline::run_const`; prism-btc
//! provides the W32 traversal that finds the admitting fiber point.

use uor_foundation::enforcement::{
    BinaryGroundingMap, CompileTime, CompileUnit, CompileUnitBuilder, ConstrainedTypeInput,
    Validated,
};
use uor_foundation::pipeline::{run_const, validate_compile_unit_const};
use uor_foundation::{VerificationDomain, WittLevel};

use crate::domain::{BlockHeader, MiningTag, MiningWitness, Target, TriadicCoords};
use crate::ops::header::serialize_prefix;
use crate::ops::term::BLOCK_HASH_SHAPE_TERM;
use crate::ops::traversal::{traverse_sequential, Cancel, Cancelled, FiberOutcome};
use crate::shapes::hasher::Sha256dHasher;

const VERIFICATION_DOMAINS: &[VerificationDomain] = &[VerificationDomain::ComposedAlgebraic];

/// The const-validated BlockHash CompileUnit. Baked at compile time;
/// any structural malformation panics during compilation.
const BLOCK_HASH_BUILDER: CompileUnitBuilder<'static> = CompileUnitBuilder::new()
    .root_term(BLOCK_HASH_SHAPE_TERM)
    .witt_level_ceiling(WittLevel::W32)
    .thermodynamic_budget(4_294_967_295_u64)
    .target_domains(VERIFICATION_DOMAINS)
    .result_type::<ConstrainedTypeInput>();

const BLOCK_HASH_UNIT: Validated<CompileUnit<'static>, CompileTime> =
    match validate_compile_unit_const(&BLOCK_HASH_BUILDER) {
        Ok(unit) => unit,
        Err(_) => panic!("BlockHash CompileUnit must validate at compile time"),
    };

const _: () = assert!(WittLevel::W32.witt_length() == 32);

/// Mint the prism-btc shape `Grounded` via foundation's
/// `pipeline::run_const`. **Infallible** by construction
/// (const-validated unit + matching `T::IRI`).
fn block_hash_witness() -> MiningWitness {
    let grounded =
        run_const::<ConstrainedTypeInput, BinaryGroundingMap, Sha256dHasher>(&BLOCK_HASH_UNIT)
            .expect("BlockHash CompileUnit is const-validated; run_const cannot fail");
    grounded.tag::<MiningTag>()
}

/// The result of a successful `mine` invocation.
#[derive(Debug)]
pub struct MiningOutcome {
    pub witness: MiningWitness,
    pub nonce: u32,
    pub digest: [u8; 32],
    pub coords: TriadicCoords,
}

/// Failure modes from `mine`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiningFailure {
    /// All 2^32 fiber points evaluated; none satisfied the target.
    NoMatch,
    /// The boundary cancelled the in-flight traversal.
    Cancelled,
}

/// **prism-btc's public entry point** — the mining inference task per
/// architecture §5.
pub fn mine(
    header: &BlockHeader,
    target: Target,
    cancel: &dyn Cancel,
) -> Result<MiningOutcome, MiningFailure> {
    let prefix = serialize_prefix(header);
    let target_bytes = target.to_bytes();

    let outcome = traverse_sequential(&prefix, &target_bytes, cancel)
        .map_err(|Cancelled| MiningFailure::Cancelled)?;

    match outcome {
        FiberOutcome::Admitted { nonce, digest } => Ok(MiningOutcome {
            witness: block_hash_witness(),
            nonce,
            digest,
            coords: TriadicCoords::from_hash(&digest),
        }),
        FiberOutcome::Exhausted => Err(MiningFailure::NoMatch),
    }
}

/// Parallel variant — partitions the W32 ring across `threads`
/// workers in the natural coset partition; first-finder wins.
#[cfg(feature = "std")]
pub fn mine_parallel(
    header: &BlockHeader,
    target: Target,
    threads: usize,
    cancel: &(dyn Cancel + Sync),
) -> Result<MiningOutcome, MiningFailure> {
    use crate::ops::traversal::traverse_parallel;
    let prefix = serialize_prefix(header);
    let target_bytes = target.to_bytes();

    let outcome = traverse_parallel(&prefix, &target_bytes, threads, cancel)
        .map_err(|Cancelled| MiningFailure::Cancelled)?;

    match outcome {
        FiberOutcome::Admitted { nonce, digest } => Ok(MiningOutcome {
            witness: block_hash_witness(),
            nonce,
            digest,
            coords: TriadicCoords::from_hash(&digest),
        }),
        FiberOutcome::Exhausted => Err(MiningFailure::NoMatch),
    }
}

/// Re-derive the foundation-sealed shape attestation without running a
/// fiber traversal. Successor of the previous `genesis()` API.
pub fn block_hash_grounded() -> MiningWitness {
    block_hash_witness()
}
