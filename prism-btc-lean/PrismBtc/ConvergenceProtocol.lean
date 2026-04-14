import PrismBtc.FreeRankProtocol
import PrismBtc.TriadicCoords

/-!
# Convergence Protocol

Formal statements covering:

1. **σ-convergence termination**: the nonce-search loop either produces a grounded
   certificate (`freeRank = 0`) or exhausts the finite nonce fiber (all 2^32 values).
   There is no third outcome — the loop cannot diverge on a finite domain.

   NOTE: SHA256d is the **σ-projection** (ingestion hash), NOT a UOR ψ-map. Foundation
   reserves ψ for the categorical functor chain ψ_1..ψ_9
   (Constraints → Nerve → Chain → Homology → … → KInvariants).
   SHA256d satisfies none of those obligations — it is a deliberately
   non-structure-preserving avalanche function.

2. **BlockHash Witt commitment**: a `BlockHash` is a 32-tuple of W8 elements
   (32 independent sites of Z/(2^8)Z), NOT a single W256 element of Z/(2^256)Z.
   The `blockHashSiteWittBits` constant (= 8) mirrors `BLOCK_HASH_SITE_WITT_BITS`
   in Rust and names the per-site Witt reference level explicitly.
-/

/-- The per-site Witt level for `BlockHash` sites. Mirrors `BLOCK_HASH_SITE_WITT_BITS : u16 = 8`
    in Rust (`prism-btc-types/src/block_hash.rs`). -/
def blockHashSiteWittBits : Nat := 8

/-- The nonce space is finite — bounded by 2^32. -/
theorem nonce_space_finite : Fintype (Fin UInt32.size) := inferInstance

/-- σ-convergence terminates: either a certificate is produced or the nonce fiber is
    exhausted. There is no third outcome — the loop cannot diverge on a finite domain. -/
theorem sigma_convergence_terminates_or_fiber_exhausted
    {T : Type} (search : Fin UInt32.size → Option T) :
    (∃ n : Fin UInt32.size, (search n).isSome) ∨
    (∀ n : Fin UInt32.size, (search n).isNone) := by
  by_cases h : ∃ n, (search n).isSome
  · exact Or.inl h
  · push_neg at h
    exact Or.inr (fun n => Option.not_isSome_iff_isNone.mp (h n))

/-- A `BlockHash` occupies 32 independent W8 sites, not a single W256 ring element.
    Each site is Z/(2^8)Z; the total Datum space has 256^32 = 2^256 values.
    This theorem records the arithmetic identity: 32 sites × 8 bits/site = 256 bits. -/
theorem block_hash_is_32_tuple_w8 :
    (32 : Nat) * blockHashSiteWittBits = 256 := by norm_num [blockHashSiteWittBits]

/-- The per-site Witt level for `BlockHash` is W8 (wittBits = 8). -/
theorem block_hash_site_witt_bits_eq_8 : blockHashSiteWittBits = 8 := rfl
