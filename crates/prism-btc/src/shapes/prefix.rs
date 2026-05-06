//! `TemplatePrefixShape` — the input ConstrainedTypeShape.
//!
//! 76 W8 sites: the bytes [0..76) of a Bitcoin block header (every
//! field except the trailing 4-byte nonce). The Grounding admits a
//! 76-byte slice as a `Datum`; ill-formed lengths emit a typed
//! impossibility witness.

use uor_foundation::pipeline::{ConstrainedTypeShape, ConstraintRef};

/// The 76-byte template prefix: `version‖prev_hash‖merkle_root‖timestamp‖bits`.
pub struct TemplatePrefixShape;

impl ConstrainedTypeShape for TemplatePrefixShape {
    const IRI: &'static str = "https://prism.btc/shape/TemplatePrefix";
    const SITE_COUNT: usize = 76;
    const CONSTRAINTS: &'static [ConstraintRef] = &[];
}
