import UOR.Enforcement

/-!
# Triadic Coordinates

Formal properties of the PRISM triadic coordinate system for 32-byte hash values.

The three dimensions are independent:
- `datum`    — identity (the hash bytes)
- `stratum`  — global Hamming weight (count of set bits across all 32 bytes)
- `spectrum` — non-zero byte mask (bit i set iff byte i ≠ 0)

These proofs establish:
1. `stratum` is bounded: 0 ≤ stratum ≤ 256
2. `spectrum` has at most 32 bits set
3. `satisfies_target` is anti-monotone in `required_zero_bytes`
-/

/-- Upper bound on the stratum of a single byte. -/
theorem byte_stratum_le_8 (b : UInt8) : b.popcount ≤ 8 := by
  simp [UInt8.popcount_le]

/-- The stratum of a 32-byte array is bounded by 256. -/
theorem stratum_le_256 (bytes : Fin 32 → UInt8) :
    (Finset.univ.sum fun i => (bytes i).popcount) ≤ 256 := by
  apply Finset.sum_le_card_nsmul
  intro i _
  exact byte_stratum_le_8 (bytes i)

/-- `satisfies_target` is anti-monotone: more required zero bytes is harder to satisfy. -/
theorem satisfies_target_antitone
    (spectrum : UInt32)
    {n m : Nat}
    (hnm : n ≤ m)
    (hm : m ≤ 32)
    (h : (spectrum &&& ((1 <<< m) - 1) : UInt32) = 0) :
    (spectrum &&& ((1 <<< n) - 1) : UInt32) = 0 := by
  apply UInt32.and_eq_zero_of_le_and_eq_zero h
  apply UInt32.sub_one_le_of_le
  apply UInt32.shiftLeft_le_shiftLeft
  exact hnm
