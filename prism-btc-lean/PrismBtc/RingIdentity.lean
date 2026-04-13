import UOR.Enforcement

/-!
# Ring Identity: neg(bnot(x)) = succ(x)

The core algebraic identity of Z/(2^n)Z:
  `-(~~~x) = x + 1`

Proved at two Witt levels used by Bitcoin types:
- W8  (Z/(2^8)Z)  — per-byte hash ring — exhaustive via `decide` (256 values)
- W32 (Z/(2^32)Z) — nonce ring         — symbolic via `omega` + `simp`

Division of labor with Rust:
- The Rust `uor!` assertions in prism-btc-types/src/assertions.rs cover 4 spot checks
  at compile time. These Lean proofs cover the full universal statement.
- The Rust `ring_identity_u8` / `ring_identity_u32` functions in prism-btc-primitives
  verify at runtime; these proofs verify formally.
-/

-- Exhaustive proof for UInt8 (W8 level) — 256 values, decidable by computation
theorem neg_bnot_eq_succ_u8 (x : UInt8) : -(~~~x) = x + 1 := by decide

-- Symbolic proof for UInt32 (W32 level) — omega handles Z/(2^32)Z modular arithmetic
theorem neg_bnot_eq_succ_u32 (x : UInt32) : -(~~~x) = x + 1 := by
  simp [UInt32.neg_def, UInt32.complement_def]
  omega
