//! `TargetSubBundle` — the output ConstrainedTypeShape.
//!
//! 32 W8 sites whose admission rule is `digest ≤ target` lexicographically
//! in display order. The target value is template-dependent (decoded from
//! the compact nBits field); the structural CONSTRAINTS list captures the
//! comparison rule, with the runtime target value carried through the
//! traversal.

use uor_foundation::pipeline::{ConstrainedTypeShape, ConstraintRef};

/// The 32-byte digest sub-bundle dominated by a target.
pub struct TargetSubBundle;

impl ConstrainedTypeShape for TargetSubBundle {
    const IRI: &'static str = "https://prism.btc/shape/TargetSubBundle";
    const SITE_COUNT: usize = 32;
    /// foundation 0.3.1's `ConstraintRef` enum does not yet expose a
    /// content-comparison variant for runtime-bound thresholds; the
    /// admission rule is therefore evaluated by prism-btc's traversal
    /// runtime ([`crate::ops::traversal`]). Each fiber visit's digest
    /// is checked against the (template-dependent) target byte vector
    /// in display order. The Term-level structural identity of this
    /// shape is "32 W8 sites bounded by a runtime byte sequence";
    /// foundation 0.3.1 does not provide the ConstraintRef vocabulary
    /// to express that bound declaratively, so prism-btc evaluates it
    /// procedurally per architecture §13's substrate-vs-implementor
    /// split.
    const CONSTRAINTS: &'static [ConstraintRef] = &[];
}
