# ADR-024: Multiplicity-Stratified Mining Parallelism (MSMP) via Profinite Fiber Stratification (PFS)

## Status
Proposed

## Context
The `prism-btc` architecture currently models Bitcoin proof-of-work as a W32 nonce ring traversal. While functional and formally verified for $\sigma$-convergence, it treats the nonce space as a flat 1D search. Multiplicity Theory suggests that this space possesses internal structure—specifically $p$-adic strata—that can be exploited for both semantic completeness and stratified parallelism.

As identified in `PRISM-BTC-MULTIPLICITY.md`, the output `MiningOutcome` already carries `TriadicCoords` (datum, stratum, spectrum). However, these are currently treated as post-hoc annotations rather than fundamental scheduling primitives.

## Decision
We will integrate Multiplicity Theory into the core mining engine by implementing **Profinite Fiber Stratification (PFS)**.

1.  **Nonce Redefinition:** The nonce ring $H = \mathbb{Z}/2^{32}\mathbb{Z}$ will be treated as the image of the canonical projection $\pi_{32} : \hat{\mathbb{Z}} \twoheadrightarrow \mathbb{Z}/2^{32}\mathbb{Z}$, where $\hat{\mathbb{Z}}$ is the profinite completion of the integers.
2.  **Stratified Partitioning:** The search space $H$ will be partitioned into $p$-adic strata $S_{p,m} = \{ n \in \hat{\mathbb{Z}} : v_p(n) = m \}$.
3.  **MSMP Scheduler:** The `NonceFiberTraversal` runtime will be refactored to dispatch workers to specific strata. Workers will hunt for fiber points within their assigned $S_{p,m}$.
4.  **Typed Attestation:** Each worker will manage a `Grounded<ConstrainedTypeInput, MiningTag>` candidate shell, which is only promoted to a full witness upon $\sigma$-satisfaction within the assigned stratum.

### Architectural Parameters
The following constants are added to `PrismBtcBounds`:

| Constant | Value | Justification |
|---|---|---|
| `STRATA_DEPTH` | `32` | log₂(|W32|); the decomposition basis of the 2-adic partition — distinct from `WITT_LEVEL_MAX_BITS` which is the ring ceiling. |
| `STRATA_MAX_WORKERS` | `32` | One worker per depth level; bounded by `STRATA_DEPTH`. |

### Trace Event Resolution
To preserve the `TRACE_MAX_EVENTS = 64` ceiling and ensure TC-05 compliance, only the trace of the winning worker (the one that finds the admitted fiber) is emitted to the boundary. Exhausted-worker traces are discarded. This ensures that the final `Trace` remains compact and structurally consistent.

### Double-Attestation Guard (FreeRank)
The `FreeRank` protocol is enforced by ensuring only the first worker to return `FiberOutcome::Admitted` is permitted to invoke `BitcoinMiningModel::forward`. All other workers are immediately signaled to cancel via the existing `&dyn Cancel` interface, preventing redundant or conflicting attestations.

## Consequences

### Positive
- **Predictable TriadicCoords:** The 2-adic stratum coordinate of a solution becomes predictable based on the worker's assigned partition.
- **Natural Load Balancing:** Strata density ($2^{-(m+1)}$ for depth $m$) allows for deterministic worker allocation without complex coordination.
- **Semantic Depth:** The mining process becomes a physical realization of Multiplicity Theory's prime axes ($p=2$ for identity, $p=5$ for ceiling, $p=11$ for witness).

### Negative / Risks
- **Implementation Overhead:** Lifting the ring to $\hat{\mathbb{Z}}$ introduces mathematical complexity into the `HostBounds` and `PrimitiveOp` compositions.
- **Proof Obligations:** New Lean 4 proofs are required to demonstrate that $\pi_{32}$ is a ring homomorphism and that the $p$-adic partition covers $H$ exhaustively.

## Implementation Path
1.  **Research:** Formalize the profinite lift in `prism-btc-lean`.
2.  **Refactor:** Update `PrismBtcBounds` to include stratification constants.
3.  **Implement:** Add `StratifiedTraversal` to `crates/prism-btc/src/ops/traversal.rs`.
4.  **Verify:** Capture `MiningOutcome` with predictable `TriadicCoords` against Bitcoin regtest.

## Mirror Dissonance (Technical Tensions)
- **Claim vs Mechanism:** We claim "inference" but use "stratified search." The mechanism remains a traversal; the stratification merely indexes it.
- **Ontology vs Instrument:** Using strata for "load balancing" instrumentalizes a structural claim. We must ensure the structure remains prior to the optimization.
- **Completeness vs Throughput:** MSMP prioritizes semantic completeness (knowing the stratum before the find) over raw hash rate.
