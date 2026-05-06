//! Compile-time structural identity of prism-btc's operations,
//! expressed in foundation `PrimitiveOp` + `Term` vocabulary.
//!
//! These constants are the type-level "categorical routing" between
//! input and output types: each named operation is declared as the
//! structural composition that maps its inputs to its outputs. The
//! foundation pipeline does not execute these Terms at runtime
//! (architecture §13's substrate-vs-implementor split); they are
//! attached to `CompileUnit::root_term` to record the operation's
//! identity in the canonical byte layout that the foundation Hasher
//! folds into a `ContentFingerprint`.
//!
//! The actual evaluator for each operation lives in a sibling module
//! ([`crate::ops::sha256`], [`crate::ops::header`], [`crate::ops::sigma`],
//! [`crate::ops::merkle`], [`crate::ops::traversal`]). Foundation's
//! 0.3.1 `Term` enum admits 9 variants and the `PrimitiveOp` enum
//! admits 10 dihedral generators (`Neg, Bnot, Succ, Pred, Add, Sub,
//! Mul, Xor, And, Or`); prism-btc composes them as documented per
//! operation.
//!
//! The architecture's stricter elaboration — full `Term::Application`
//! trees expressing every SHA-256 round, every byte-pack, every
//! pairwise merkle step — is omitted here for the reconciled state:
//! the foundation pipeline does not evaluate them, the canonical
//! byte layout the Hasher folds is unchanged, and authoring 64-round
//! SHA-256 as nested `Term::Application` constants would be hundreds
//! of literals without changing what the pipeline certifies. When
//! foundation 0.3.1's pipeline gains a Term evaluator, this module
//! is the place those literals live.

use uor_foundation::enforcement::Term;
use uor_foundation::WittLevel;

/// The canonical "BlockHash shape" root term placed in the prism-btc
/// `CompileUnit`. A single W8-level literal site, representing the
/// 32 bytes of an admitted block-hash digest.
///
/// One W8-level literal site is sufficient for foundation 0.3.1's pipeline to
/// produce a `Grounded<ConstrainedTypeInput, MiningTag>` whose
/// fingerprint depends only on `(witt_level_bits, budget, IRI,
/// SITE_COUNT, CONSTRAINTS)`. The shape's site count and IRI are
/// the load-bearing identity bits; the term tree contributes the
/// foundation's nominal "compute against" anchor.
pub static BLOCK_HASH_SHAPE_TERM: &[Term] = &[Term::Literal {
    value: 0,
    level: WittLevel::W8,
}];
