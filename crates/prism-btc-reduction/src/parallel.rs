//! Parallel, cancellable σ-convergence over the W32 nonce ring.
//!
//! The nonce space `Z/(2^32)Z` is partitioned into `threads` contiguous
//! cosets — natural with respect to the `W32` ring structure — and each
//! worker scans its slice in parallel. The first satisfying nonce wins;
//! losers observe the cancel flag set by the winner and bail.
//!
//! An external [`AtomicBool`] cancel handle lets the caller (e.g. a tip
//! watcher in the orchestrator) abort an in-flight search when the chain
//! advances and the current template becomes stale.
//!
//! Available behind the `parallel` cargo feature.

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use prism_btc_types::BlockHeader;
use rayon::prelude::*;
use uor_foundation::enforcement::{ProjectionMapKind, Total};

use crate::block_hash_shape_certificate;
use crate::certificate::BlockCertificate;

/// Why a parallel σ-convergence search returned without a certificate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoMatch {
    /// All `2^32` nonces in the W32 fiber were tried; none satisfied the target.
    Exhausted,
    /// The external cancel flag fired before exhaustion.
    Cancelled,
}

/// How often each worker checks the cancel flag (in nonces). The check is a
/// single relaxed atomic load, so picking a moderate granularity keeps the
/// inner loop tight while still letting the orchestrator stop within a few ms.
const CANCEL_CHECK_STRIDE: u32 = 8192;

/// Run a parallel, cancellable σ-convergence search.
///
/// `hash_fn` is the σ-projection (header‖nonce → 32-byte digest); for Bitcoin
/// that is SHA256d-of-(serialized 80-byte header). It must be `Sync` because
/// it's called from every rayon worker.
///
/// `cancel` is the external abort handle — the orchestrator sets it to `true`
/// when the chain tip advances or when the user requests stop.
///
/// `threads` is the number of cosets to partition the W32 ring into. Pass
/// `None` to use rayon's global thread pool (typically `num_cpus`). Threads
/// are scheduled on rayon's existing pool; this function does not spawn its
/// own pool.
pub fn run_convergence_parallel<Sigma, F>(
    header: BlockHeader,
    target_bytes: [u8; 32],
    hash_fn: F,
    cancel: &AtomicBool,
    threads: Option<usize>,
) -> Result<BlockCertificate<Sigma>, NoMatch>
where
    Sigma: ProjectionMapKind + Total,
    F: Fn(u32) -> [u8; 32] + Sync,
{
    let grounded = block_hash_shape_certificate();

    let pool_threads = threads.unwrap_or_else(rayon::current_num_threads).max(1);

    // The W32 ring has 2^32 elements; partition into `pool_threads` contiguous
    // cosets. Each coset spans `chunk = ceil(2^32 / pool_threads)` nonces.
    let total: u64 = 1u64 << 32;
    let chunk: u64 = total.div_ceil(pool_threads as u64);

    // Shared slot for the winning (nonce, hash). Atomic u32 gives lock-free
    // first-finder-wins; we encode "no winner" as the sentinel `u32::MAX` plus
    // a separate flag (since u32::MAX is itself a valid nonce). We pair the
    // nonce atomic with a "found" AtomicBool.
    let found = AtomicBool::new(false);
    let winning_nonce = AtomicU32::new(0);
    let mut winning_hash: [u8; 32] = [0u8; 32];
    let winning_hash_slot = std::sync::Mutex::new(&mut winning_hash);

    let ranges: Vec<(u64, u64)> = (0..pool_threads as u64)
        .map(|i| {
            let start = i * chunk;
            let end = (start + chunk).min(total);
            (start, end)
        })
        .collect();

    ranges.par_iter().for_each(|&(start, end)| {
        let mut n = start;
        while n < end {
            // Periodic cancel / found check.
            if ((n - start) as u32).is_multiple_of(CANCEL_CHECK_STRIDE)
                && (cancel.load(Ordering::Relaxed) || found.load(Ordering::Relaxed))
            {
                return;
            }
            let nonce_u32 = n as u32;
            let hash = hash_fn(nonce_u32);
            if hash <= target_bytes {
                // First writer wins.
                if !found.swap(true, Ordering::AcqRel) {
                    winning_nonce.store(nonce_u32, Ordering::Release);
                    if let Ok(mut slot) = winning_hash_slot.lock() {
                        **slot = hash;
                    }
                }
                return;
            }
            n += 1;
        }
    });

    if found.load(Ordering::Acquire) {
        let nonce = winning_nonce.load(Ordering::Acquire);
        let hash = winning_hash;
        Ok(BlockCertificate::new(grounded, nonce, hash, header))
    } else if cancel.load(Ordering::Relaxed) {
        Err(NoMatch::Cancelled)
    } else {
        Err(NoMatch::Exhausted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_foundation::enforcement::DigestProjectionMap;

    fn easy_header() -> BlockHeader {
        use prism_btc_types::{Bits, MerkleRoot, Timestamp, Version};
        BlockHeader {
            version: Version(1),
            prev_hash: [0u8; 32],
            merkle_root: MerkleRoot::from_bytes([0u8; 32]),
            timestamp: Timestamp(1_700_000_000),
            bits: Bits(0x207fffff),
        }
    }

    /// Synthetic σ-projection: returns a 32-byte hash that is `0x00...00<nonce>`
    /// — easy target satisfied iff nonce ≤ some value.
    fn synth_hash(nonce: u32) -> [u8; 32] {
        let mut h = [0u8; 32];
        h[28..].copy_from_slice(&nonce.to_be_bytes());
        h
    }

    #[test]
    fn parallel_finds_satisfying_nonce() {
        // target accepts hashes ≤ 0x...00_0000_0010 → nonces 0..=16
        let mut target = [0u8; 32];
        target[31] = 0x10;
        let cancel = AtomicBool::new(false);
        let cert = run_convergence_parallel::<DigestProjectionMap, _>(
            easy_header(),
            target,
            synth_hash,
            &cancel,
            Some(4),
        )
        .expect("nonces 0..=16 satisfy this target; search must succeed");
        assert!(cert.digest()[31] <= 0x10);
    }

    #[test]
    fn parallel_respects_cancel() {
        // Impossible target (all-zero) so the search would otherwise sweep all 2^32.
        let target = [0u8; 32];
        let cancel = AtomicBool::new(false);
        // Pre-set cancel so the workers exit on the first stride check.
        cancel.store(true, Ordering::Relaxed);
        let result = run_convergence_parallel::<DigestProjectionMap, _>(
            easy_header(),
            target,
            synth_hash,
            &cancel,
            Some(2),
        );
        assert_eq!(result.err(), Some(NoMatch::Cancelled));
    }
}
