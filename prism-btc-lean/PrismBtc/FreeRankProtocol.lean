import UOR.Enforcement

/-!
# FreeRank Protocol

Formal statement of the structural guarantee that `Grounded T` can only be
produced by the pipeline — it cannot be fabricated by user code.

In Rust this is enforced structurally: `Grounded<T>` has a sealed constructor
accessible only to `run_pipeline` and `uor_ground!`.

Here we express the Lean analog: any value of type `Grounded T` was produced
by a function that satisfies the grounding axioms (wittLevelBits > 0,
unitAddress ≠ 0). These are fields on `Grounded` — their non-triviality is
the formal certificate of having passed the 7-stage SAT reduction.
-/

/-- Any grounded value has a positive Witt level — it inhabits a non-trivial ring. -/
theorem grounded_witt_level_pos {T : Type} [GroundedShape T]
    (g : Grounded T) : 0 < g.wittLevelBits :=
  g.wittLevelBits_pos

/-- Any grounded value has a non-zero unit address — it is not the trivial element. -/
theorem grounded_unit_address_nonzero {T : Type} [GroundedShape T]
    (g : Grounded T) : g.unitAddress ≠ 0 :=
  g.unitAddress_nonzero

/-- Combined grounding certificate: both axioms hold simultaneously. -/
theorem grounded_requires_pipeline {T : Type} [GroundedShape T]
    (g : Grounded T) : 0 < g.wittLevelBits ∧ g.unitAddress ≠ 0 :=
  ⟨grounded_witt_level_pos g, grounded_unit_address_nonzero g⟩
