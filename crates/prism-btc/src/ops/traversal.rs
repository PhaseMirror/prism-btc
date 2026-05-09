//! `NonceFiberTraversal` — the W32 nonce fiber traversal runtime.
//!
//! prism-btc is the prism implementor for the Bitcoin use case;
//! foundation provides the substrate (sealed types, vocabulary, mint
//! primitives) and prism-btc provides the runtime that traverses the
//! typed structure. The W32 fiber traversal is therefore prism-btc's
//! own code, not a foundation primitive.
//!
//! Structural identity (compile-time, declarative): the W32 fiber is
//! `Z/(2^32)Z`, declared at the type level via `WittLevel::W32`; the
//! per-index map is `Sha256dProjection ∘ HeaderSerialization`,
//! declared in [`crate::ops::term`]; the halt predicate is admission
//! to `TargetSubBundle`. The Term composition is the **categorical
//! routing** between the input type (template prefix) and the output
//! type (admitting digest).
//!
//! Runtime evaluation: this module walks the W32 ring in canonical
//! successor order, computes the σ-projection for each fiber point,
//! tests admission, and halts at the first admitting nonce.
//! Determinism: same prefix + same target ⇒ same admitting nonce
//! (or `Exhausted` if none in W32 satisfies).

use crate::ops::omega::OmegaBtc;

/// Outcome of a W32 fiber traversal for a fixed (prefix, target) pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiberOutcome {
    /// Admitted at this nonce (the first satisfying point in canonical W32 order).
    Admitted { nonce: u32, digest: [u8; 32] },
    /// All 2^32 fiber points evaluated; none admitted under the target.
    Exhausted,
}

/// Traverse the W32 fiber sequentially, grounded in the Ω operator.
///
/// `prefix` is the 76-byte header prefix (extranonce-fixed merkle root);
/// `target` is the 32-byte big-endian threshold (display order).
///
/// `cancel` is a no-cost trait-object hook the boundary uses to abort
/// in-flight traversals on tip-change. Pass `&NeverCancel` if no abort
/// is desired.
pub fn traverse_sequential(
    prefix: &[u8; 76],
    target: &[u8; 32],
    cancel: &dyn Cancel,
) -> Result<FiberOutcome, Cancelled> {
    let omega = OmegaBtc::default();
    let mut nonce: u32 = 0;
    loop {
        if (nonce & 0x3FFF) == 0 && cancel.is_cancelled() {
            return Err(Cancelled);
        }
        
        // Spectral evaluation via specialized Ω_btc
        let digest = omega.project(prefix, nonce);
        
        // Admissibility gate
        if omega.is_admissible(&digest, target) {
            return Ok(FiberOutcome::Admitted { nonce, digest });
        }
        
        match nonce.checked_add(1) {
            Some(n) => nonce = n,
            None => return Ok(FiberOutcome::Exhausted),
        }
    }
}

/// Traverse the W32 fiber across `threads` workers in the natural coset
/// partition, grounded in the Ω operator. First admitting nonce wins; 
/// losers observe a shared found/cancel flag set by the winner and bail.
#[cfg(feature = "std")]
pub fn traverse_parallel(
    prefix: &[u8; 76],
    target: &[u8; 32],
    threads: usize,
    cancel: &(dyn Cancel + Sync),
) -> Result<FiberOutcome, Cancelled> {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Mutex;
    use std::thread;

    let threads = threads.max(1);

    if threads == 1 {
        return traverse_sequential(prefix, target, cancel);
    }

    let found = AtomicBool::new(false);
    let cancelled_external = AtomicBool::new(false);
    let winner: Mutex<Option<(u32, [u8; 32])>> = Mutex::new(None);

    thread::scope(|s| {
        for worker_id in 0..threads {
            // Each worker walks every `threads`-th nonce starting at its
            // index (the natural Z/(2^32)Z coset partition).
            let found_ref = &found;
            let cancel_ref = &cancelled_external;
            let winner_ref = &winner;
            s.spawn(move || {
                let omega = OmegaBtc::default();
                let mut nonce = worker_id as u32;
                let stride = threads as u32;
                let mut local_check_counter: u32 = 0;
                loop {
                    if found_ref.load(Ordering::Relaxed) {
                        return;
                    }
                    if local_check_counter == 0 && cancel.is_cancelled() {
                        cancel_ref.store(true, Ordering::Relaxed);
                        found_ref.store(true, Ordering::Relaxed);
                        return;
                    }
                    local_check_counter = (local_check_counter + 1) & 0x3FFF;

                    // Spectral evaluation via specialized Ω_btc
                    let digest = omega.project(prefix, nonce);
                    
                    // Admissibility gate
                    if omega.is_admissible(&digest, target) {
                        if found_ref
                            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                            .is_ok()
                        {
                            if let Ok(mut slot) = winner_ref.lock() {
                                *slot = Some((nonce, digest));
                            }
                        }
                        return;
                    }
                    nonce = match nonce.checked_add(stride) {
                        Some(n) => n,
                        None => return,
                    };
                }
            });
        }
    });

    if cancelled_external.load(Ordering::Acquire) {
        return Err(Cancelled);
    }
    if let Ok(slot) = winner.lock() {
        if let Some((nonce, digest)) = *slot {
            return Ok(FiberOutcome::Admitted { nonce, digest });
        }
    }
    Ok(FiberOutcome::Exhausted)
}

/// Cancel signal interface: the boundary's tip-watcher implements this
/// to interrupt an in-flight traversal.
pub trait Cancel {
    fn is_cancelled(&self) -> bool;
}

/// Sentinel: never cancelled. Used when no abort is desired.
pub struct NeverCancel;
impl Cancel for NeverCancel {
    fn is_cancelled(&self) -> bool {
        false
    }
}

/// Cancellation result, distinct from `FiberOutcome` so callers can
/// distinguish "the chain advanced, drop this and start over" from
/// "the W32 fiber was searched in full".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cancelled;

#[cfg(feature = "std")]
impl<T> Cancel for T
where
    T: AsRef<std::sync::atomic::AtomicBool>,
{
    fn is_cancelled(&self) -> bool {
        self.as_ref().load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Bits, BlockHeader, MerkleRoot, Timestamp, Version};
    use crate::ops::header::serialize_prefix;

    fn easy_prefix() -> [u8; 76] {
        let merkle: [u8; 32] = [
            0x3b, 0xa3, 0xed, 0xfd, 0x7a, 0x7b, 0x12, 0xb2, 0x7a, 0xc7, 0x2c, 0x3e, 0x67, 0x76,
            0x8f, 0x61, 0x7f, 0xc8, 0x1b, 0xc3, 0x88, 0x8a, 0x51, 0x32, 0x3a, 0x9f, 0xb8, 0xaa,
            0x4b, 0x1e, 0x5e, 0x4a,
        ];
        let h = BlockHeader {
            version: Version(1),
            prev_hash: [0u8; 32],
            merkle_root: MerkleRoot::from_bytes(merkle),
            timestamp: Timestamp(1700000000),
            bits: Bits(0x207fffff),
        };
        serialize_prefix(&h)
    }

    #[test]
    fn sequential_admits_against_easy_target() {
        // Target 0xff... (all-ones) admits the very first fiber point (any digest ≤ all-ones).
        let target = [0xffu8; 32];
        let prefix = easy_prefix();
        let outcome = traverse_sequential(&prefix, &target, &NeverCancel).expect("not cancelled");
        match outcome {
            FiberOutcome::Admitted { nonce: 0, .. } => {}
            other => panic!("expected admit at nonce 0, got {other:?}"),
        }
    }

    #[test]
    fn sequential_respects_cancel() {
        let target = [0u8; 32]; // unsatisfiable (digest can't be ≤ all-zero unless it IS all-zero)
        let prefix = easy_prefix();
        // Use AtomicBool as the cancel signal.
        let flag = std::sync::atomic::AtomicBool::new(true); // pre-cancelled
        struct Flag<'a>(&'a std::sync::atomic::AtomicBool);
        impl<'a> Cancel for Flag<'a> {
            fn is_cancelled(&self) -> bool {
                self.0.load(std::sync::atomic::Ordering::Relaxed)
            }
        }
        let result = traverse_sequential(&prefix, &target, &Flag(&flag));
        assert_eq!(result, Err(Cancelled));
    }

    #[cfg(feature = "std")]
    #[test]
    fn parallel_admits_against_easy_target() {
        let target = [0xffu8; 32];
        let prefix = easy_prefix();
        let outcome = traverse_parallel(&prefix, &target, 4, &NeverCancel).expect("not cancelled");
        match outcome {
            FiberOutcome::Admitted { .. } => {}
            other => panic!("expected admit, got {other:?}"),
        }
    }
}
