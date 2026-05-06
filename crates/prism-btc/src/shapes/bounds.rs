//! `PrismBtcBounds` — prism-btc's `HostBounds` selection.
//!
//! ADR-018 routes every capacity bound on the principal data path
//! through `HostBounds`. Architecture §3.2 specifies the four constants
//! prism-btc commits to.

use uor_foundation::HostBounds;

/// prism-btc's capacity profile.
///
/// - `FINGERPRINT_MIN_BYTES = 32` — matches SHA-256 output width.
/// - `FINGERPRINT_MAX_BYTES = 32` — fixed; one Hasher (`Sha256dHasher`).
/// - `TRACE_MAX_EVENTS = 64` — bounds per-`pipeline::run` traces at a
///   small constant; one event per stage transition (architecture §6.4),
///   not per fiber visit.
/// - `WITT_LEVEL_MAX_BITS = 32` — the W32 nonce ring is the largest
///   algebraic level on the principal data path.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PrismBtcBounds;

impl HostBounds for PrismBtcBounds {
    const FINGERPRINT_MIN_BYTES: usize = 32;
    const FINGERPRINT_MAX_BYTES: usize = 32;
    const TRACE_MAX_EVENTS: usize = 64;
    const WITT_LEVEL_MAX_BITS: u32 = 32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds_constants_match_architecture_3_2() {
        assert_eq!(<PrismBtcBounds as HostBounds>::FINGERPRINT_MIN_BYTES, 32);
        assert_eq!(<PrismBtcBounds as HostBounds>::FINGERPRINT_MAX_BYTES, 32);
        assert_eq!(<PrismBtcBounds as HostBounds>::TRACE_MAX_EVENTS, 64);
        assert_eq!(<PrismBtcBounds as HostBounds>::WITT_LEVEL_MAX_BITS, 32);
    }
}
