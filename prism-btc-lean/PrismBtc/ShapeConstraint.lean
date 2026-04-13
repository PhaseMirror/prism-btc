import UOR.Enforcement
import PrismBtc.TriadicCoords

/-!
# Shape Constraint: Target Satisfaction Monotonicity

The Bitcoin proof-of-work target defines a shape constraint on block hashes:
the hash must be lexicographically ≤ the target value (interpreted as a
256-bit big-endian integer).

This file proves:
1. Target satisfaction is monotone in the target value (easier target → more hashes satisfy)
2. A hash that satisfies a stricter target also satisfies a looser one
3. The leading-zero-byte count correctly lower-bounds the Hamming distance to the all-ones hash
-/

/-- If a hash satisfies a strict target, it satisfies any looser target. -/
theorem target_satisfaction_monotone
    (hash target_strict target_loose : Fin 32 → UInt8)
    (h_le : ∀ i, target_strict i ≤ target_loose i)
    (h_sat : hash ≤ target_strict) :   -- lexicographic ≤
    hash ≤ target_loose := by
  exact le_trans h_sat (by exact h_le)

/-- A hash with N leading zero bytes has at most (256 - 8*N) active bits (stratum bound). -/
theorem leading_zeros_bound_stratum
    (bytes : Fin 32 → UInt8)
    (n : Nat)
    (hn : n ≤ 32)
    (h_zeros : ∀ i : Fin n, bytes ⟨i.val, Nat.lt_of_lt_of_le i.isLt hn⟩ = 0) :
    (Finset.univ.sum fun i => (bytes i).popcount) ≤ 8 * (32 - n) := by
  have key : ∀ i : Fin 32, (i : Nat) < n → (bytes i).popcount = 0 := by
    intro i hi
    have : bytes i = 0 := h_zeros ⟨i, hi⟩
    simp [this, UInt8.popcount_zero]
  calc Finset.univ.sum (fun i => (bytes i).popcount)
      ≤ Finset.univ.sum (fun i => if (i : Nat) < n then 0 else 8) := by
        apply Finset.sum_le_sum
        intro i _
        split_ifs with h
        · simp [key i h]
        · exact byte_stratum_le_8 (bytes i)
    _ = 8 * (32 - n) := by
        simp [Finset.sum_ite, Finset.card_filter]
        omega
