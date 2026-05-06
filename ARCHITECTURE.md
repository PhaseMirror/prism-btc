# prism-btc: Defined Architecture

> **Status:** Normative for prism-btc. This document is the authoritative
> specification of what prism-btc is, what it claims, and how it realizes
> those claims through Prism + uor-foundation. The repository state is
> reconciled to this document, not the other way around.
>
> **Frame of reference:** the [UOR-Framework wiki](https://github.com/UOR-Foundation/UOR-Framework/wiki),
> which is itself the normative specification of Prism — the three-crate
> system (`uor-foundation`, `prism`, `prism-verify`) realising the boundary
> properties TC-01..TC-06.

---

## 1. What prism-btc is

prism-btc is a **Bitcoin application of Prism**. Its public artifact is the
binary `prism-mine`, which produces blocks accepted byte-for-byte by Bitcoin
Core. Its architectural artifact is the discipline that every step of the
production path is expressed in Prism + uor-foundation vocabulary, with no
opaque external computation crossing the boundary.

This is the load-bearing distinction. A traditional Bitcoin miner is a
black-box hash-and-compare loop: it imports an opaque SHA-256 implementation,
iterates a `u32`, compares bytes against a threshold, and emits a block.
The miner's process is invisible to the type system and untraced.
prism-btc's claim is the converse: **bit-identical output to a traditional
miner, derived through Prism's vocabulary alone.** Every operation that
contributes to the output is either a foundation `PrimitiveOp` composition,
a sealed `pipeline::run` traversal, or a `prism-verify` replay walk — all
type-checked at compile time (TC-04, ADR-006) and recorded in a `Trace`
that an independent verifier can replay without re-hashing (TC-05).

What prism-btc does **not** claim:

- It does **not** invert SHA-256. SHA-256 is a sealed cryptographic
  one-way digest under foundation's `Hasher` substitution-axis contract
  (ADR-010); prism-btc evaluates it on candidates exactly as the
  protocol requires. The novelty is *where* the evaluation happens
  (inside foundation's `PrimitiveOp` composition surface, not inside an
  opaque crate import) and *how* it is recorded (as `TraceEvent`s).
- It does **not** speed up mining. Wall-clock time to a winning nonce is
  the same as any other CPU implementation at the same hash rate.
- It does **not** add new primitive operations to the framework
  (ADR-013): everything is a composition of foundation's closed
  `PrimitiveOp` set (bit-rotation, content-comparison,
  depth-projection, integer-handling, lookup, observable-arithmetic).

The value is architectural, not algorithmic.

---

## 2. The Bitcoin domain in Prism vocabulary

Each Bitcoin object maps onto a foundation type:

| Bitcoin object | Prism type | Notes |
|---|---|---|
| 80-byte serialized header | `Datum` (foundation, sealed) | minted once via `mint_datum` after `Grounding` admits the host bytes |
| Triadic decomposition of any 32-byte digest | `Triad` (foundation, sealed) | `Triad = (datum, stratum, spectrum)` where stratum is the 2-adic valuation and spectrum is the Walsh–Hadamard image (per wiki Glossary) |
| σ-projection's compositional record | `Derivation` (foundation, sealed) | minted by the `pipeline::run` stage that walks the SHA-256d composition of `PrimitiveOp`s |
| Number of free coordinates remaining in the W32 nonce fiber | `FreeRank` (foundation, sealed) | starts at 1 (nonce free); collapses to 0 when a satisfying nonce is grounded |
| Compile-unit-validated mining problem | `Validated<CompileUnit, FinalPhase>` (prism, sealed) | declared `ConstrainedTypeShape` capturing the BlockHash shape + target sub-bundle constraint |
| Mined block witness | `Grounded<BlockSolution>` (prism, sealed) | sole product of `pipeline::run` for a successful mining session |
| Replayed block certificate | `Certified<GroundingCertificate>` (prism, sealed) | produced by `prism-verify::certify_from_trace` from the session's `Trace` |

The seven sealed types above are the ones TC-02 mandates; prism-btc
introduces no additional sealed types. `BlockCertificate<Sigma>` (which
the current code defines as a wrapper) is reframed in §6 below.

The foundation's three substitution axes (ADR-007) are bound as follows:

- `HostTypes` = the host-side byte and integer types (already the foundation default).
- `HostBounds` = `prism_btc::PrismBtcBounds` (a single canonical instance: 32 fingerprint bytes, trace event ceiling sized to `2^32 + ε` to admit a complete W32 traversal trace, W32 algebraic level).
- `Hasher` = `prism_btc::Sha256dHasher` — a foundation-conforming `Hasher` impl whose state is computed entirely through foundation `PrimitiveOp` compositions (see §4). It satisfies determinism, fixed 32-byte width, idempotence (ADR-010).

---

## 3. The Bitcoin verbs as `PrimitiveOp` compositions (ADR-014)

ADR-014 commits prism to ship vocabulary, not pre-implemented operations:
"authors declare operations as `PrimitiveOp` compositions." prism-btc
therefore **declares** every Bitcoin computation as such a composition.
The closed primitive set per Building Block View is:

- `bit-rotation` (the SHA-256 round-function rotors `Σ0 Σ1 σ0 σ1`)
- `integer-handling` (modular `add`, `xor`, `and`, `not`)
- `lookup` (the `K` round-constants table; the initial-state vector)
- `content-comparison` (the lexicographic target check)
- `depth-projection` (extracting a fixed-width slice of bytes)
- `observable-arithmetic` (Hamming weight = stratum-relevant primitive)

From these primitives prism-btc declares three compositional operations:

### 3.1 `Sha256Compression` (`PrimitiveOp` composition)

The 64-round SHA-256 compression function over a 512-bit message block,
declared as a deterministic chain of `bit-rotation`, `integer-handling`,
and `lookup` against the foundation-fixed K-table. The composition is
total and pure (ADR-013 closure: no new primitives). Output: 256-bit
working state.

### 3.2 `Sha256dProjection` (`PrimitiveOp` composition)

`Sha256Compression`-twice on the canonical 80-byte header‖nonce padding,
followed by `depth-projection` to extract the 32 most-significant bytes
in display order. This is the σ-projection. It is a foundation `PrimitiveOp`
composition end-to-end; nothing imports `sha2` or any external crate.

### 3.3 `TargetSubBundle` (`ConstrainedTypeShape`)

A constrained type whose 32 W8 sites are bounded above by the target
byte sequence, declared via `content-comparison` PrimitiveOps in the
shape's `CONSTRAINTS` table. A `Datum` admits to this shape iff its
bytes are lexicographically ≤ target.

The mining problem becomes: produce a `Validated<CompileUnit,
FinalPhase>` whose result type is `TargetSubBundle` and whose Grounding
is the composition `Sha256dProjection ∘ HeaderSerialization` over the
W32 nonce fiber.

---

## 4. The W32 nonce fiber as a foundation traversal

The free dimension in mining is the nonce, an element of `Z/(2^32)Z`.
The foundation's `kernel::convergence` machinery (per Building Block
View: "convergence machinery" inside kernel) gives the natural
traversal of a W32 ring. The nonce iteration is **not** a Rust
`for nonce in 0..=u32::MAX` loop; it is a foundation-typed convergence
traversal whose every step is a `TraceEvent`.

For each fiber point `n ∈ Z/(2^32)Z`:

1. Foundation's `integer-handling` composes `n` into the canonical
   80-byte payload via `depth-projection` substitutions at offsets `[76, 80)`.
2. `Sha256dProjection` (§3.2) projects to a 32-byte `Datum` candidate.
3. The candidate is admitted (or rejected) against `TargetSubBundle`'s
   `Grounding` impl.
4. The traversal records each (n, projected datum, admit/reject) as a
   `TraceEvent` and continues until admission succeeds.

The traversal is **deterministic** in the framework's sense (Runtime
View, Scenario 1: "the pipeline is a deterministic traversal of input
data through pre-compiled stages"): given the same template and the
same `Hasher` substitution, it visits the same sequence of fiber
points and admits the same nonce.

**This is not search-by-iteration externally; it is a typed traversal
of the W32 ring whose every step is foundation-recorded.** The
algorithmic content is identical to traditional mining (cannot avoid
evaluating SHA-256d on each candidate — see §1's non-claim); the
architectural content is that every step is in-vocabulary.

---

## 5. The pipeline shape for one mining session

Per Runtime View Scenario 1, one full mining session is one invocation
of `pipeline::run`:

1. Application obtains a block template from `bitcoind` via
   `getblocktemplate` (the only opaque external interaction; outside
   Prism's scope per §3 Context-and-Scope, `Distribution Channel
   External to Prism`).
2. Application fixes the 75-byte template prefix
   (`version‖prev_hash‖merkle_root‖timestamp‖bits`).
3. Application invokes the prism-btc `Grounding` impl on the prefix —
   admits as a `Datum` if structurally well-formed.
4. Application invokes `CompileUnitBuilder` declaring
   `TargetSubBundle` and the Grounding composition
   `Sha256dProjection ∘ HeaderSerialization`.
5. Builder transitions through validation phases; emits
   `Validated<CompileUnit, FinalPhase>` or `ShapeViolation`.
6. Application invokes `pipeline::run` once. The pipeline traverses
   the W32 nonce fiber (§4), emits a `Grounded<TargetSubBundle>` whose
   internal Datum is the satisfying 32-byte digest, and emits the
   accompanying `Trace`.
7. Application splices the satisfying nonce (carried in the `Trace`'s
   final `TraceEvent`) into the wire-format block via foundation's
   `depth-projection`, and submits to `bitcoind` via `submitblock`.
8. The user (potentially a third party) receives the `Trace` and
   invokes `prism-verify::certify_from_trace`. Per TC-05 the verifier
   walks the trace structurally without invoking `Sha256dProjection`,
   producing a `Certified<GroundingCertificate>` that the same nonce
   was correctly derived under the declared shape.

**Exactly one `pipeline::run` per mined block** (TC-03: pipeline
singularity). No second pathway. No alternate constructor for
`Grounded<T>`.

---

## 6. The repository layout under this architecture

The framework's three crates plus prism-btc's application layer:

| Crate | Role |
|---|---|
| `uor-foundation` (external dep) | Substrate. Provides four UOR-domain sealed types, `PrimitiveOp` discriminants, the closed primitives set, the substitution-axis traits, and `mint_*` functions. |
| `prism` (external dep) | Runtime. Provides three Prism-mechanism sealed types, `pipeline::run`, the seal regime. |
| `prism-verify` (external dep) | Replay façade. Provides `certify_from_trace`. |
| `prism-btc-shapes` (this repo) | The domain types. Declares `BlockHashShape`, `MerkleRootShape`, `TargetShape`, `TargetSubBundle` as `ConstrainedTypeShape` impls. Declares prism-btc's `HostBounds` and `Sha256dHasher` substitution-axis selections. |
| `prism-btc-ops` (this repo) | The verbs. Declares `Sha256Compression`, `Sha256dProjection`, `HeaderSerialization`, and the W32 fiber traversal as `PrimitiveOp` compositions. No `sha2` dependency; no rayon dependency; no opaque crypto. |
| `prism-btc-pipeline` (this repo) | The application of `pipeline::run` to mining. Declares `MiningCompileUnit`, the `Grounding` impl over the template prefix, the `MinedBlock` result type wrapping `Grounded<TargetSubBundle>`. |
| `prism-btc-node` (this repo) | The bitcoind boundary. Sole external-system glue: `getblocktemplate` + `submitblock` + chain-mismatch guard + mainnet airlock. Constructs the wire-format `Block` from a `MinedBlock`'s carried fields. **No mining algorithm lives here**; the algorithm is `pipeline::run`. |
| `prism-mine` (binary in `prism-btc-node`) | CLI. |

`prism-btc-wasm` is unchanged in role; it imports `prism-btc-pipeline`
and exposes a `mine_block` JS surface.

What disappears from the current repository:

- `prism-btc-reduction/src/parallel.rs` (the rayon for-loop) — replaced
  by the foundation W32 traversal in `prism-btc-ops`.
- `prism-btc-reduction/src/sha256d.rs` (the `sha2`-crate call) — replaced
  by `Sha256dProjection` in `prism-btc-ops`.
- `prism-btc-reduction/src/serialize.rs` (the hand-rolled byte layout) —
  replaced by `HeaderSerialization` in `prism-btc-ops` (foundation
  `depth-projection` composition).
- `prism-btc-reduction/src/certificate.rs::BlockCertificate<Sigma>` —
  replaced by the unsealed `Grounded<TargetSubBundle>` minted by
  `pipeline::run`. The phantom `Sigma` parameter goes away because
  the σ-projection is now a concrete `PrimitiveOp` composition that
  `pipeline::run` executes, not a marker.
- `prism-btc-reduction/src/hasher.rs::Fnv1aHasher16` — replaced by
  `prism-btc-shapes::Sha256dHasher`, which is the only `Hasher` axis
  selection prism-btc admits and which is itself a `PrimitiveOp`
  composition.
- `Cargo.toml` `sha2` and `rayon` workspace deps — gone. The only
  external runtime deps prism-btc admits are `uor-foundation`, `prism`,
  `prism-verify`, `bitcoincore-rpc`, `rust-bitcoin`, and CLI utilities.

---

## 7. The seven boundary properties in prism-btc terms

| Constraint | How prism-btc realizes it |
|---|---|
| TC-01 zero-cost runtime | All shape declarations, substitution-axis selections, and PrimitiveOp compositions are resolved at compile time; the executable contains no runtime UORassembly enforcement. |
| TC-02 sealing | prism-btc constructs zero sealed types directly; every `Datum`, `Triad`, `Derivation`, `FreeRank`, `Validated`, `Grounded`, `Certified` arrives via `mint_*` or a `pipeline::run` return value. |
| TC-03 path singularity | One `pipeline::run` per mined block, period. No alternative constructor for `Grounded<TargetSubBundle>` exists in prism-btc. |
| TC-04 UORassembly bilateral | The `prism-btc-shapes` and `prism-btc-ops` crates' types must satisfy prism's bounds, checked by `rustc` at compile time on every build. |
| TC-05 replayability without deciders | The `Trace` emitted alongside a `MinedBlock` is sufficient for `prism-verify` to issue a `Certified<GroundingCertificate>` without re-running `Sha256dProjection`. |
| TC-06 no author infrastructure | After distribution, `prism-mine` and any verifier run entirely on user hardware. The only network dependency is the user's own bitcoind. |

---

## 8. Non-goals

- **Speed.** A traditional CPU miner using `sha2`'s assembly-tuned
  SHA-256 will be faster per hash than `Sha256Compression`-as-PrimitiveOp-
  composition. We accept the gap; the value is architectural, not
  performance.
- **Cryptanalysis.** prism-btc does not weaken SHA-256 or escape
  proof-of-work. Per §1, finding a satisfying nonce still requires the
  same expected number of digest evaluations as any other miner.
- **Foundation amendments.** Per ADR-013, `prism` is closed under
  `uor-foundation`. If prism-btc finds it cannot express something
  through the existing primitive set, the answer is to amend the
  foundation, not to import an opaque crate. (The current architecture
  asserts the existing primitive set suffices; if implementation reveals
  a gap, the foundation amendment is the corrective step.)

---

## 9. Reconciliation plan

The current repository state is non-conforming to this architecture in
the ways enumerated in §6. Reconciliation produces these effects:

1. The σ-projection becomes a foundation PrimitiveOp composition.
2. The W32 fiber traversal becomes a foundation operation.
3. The single `BlockCertificate<DigestProjectionMap>` wrapper is
   replaced by `Grounded<TargetSubBundle>` directly; phantom morphism
   kinds (`DigestProjectionMap`, `BinaryGroundingMap`,
   `BinaryProjectionMap`) are removed because the operations they were
   meant to mark are now first-class `PrimitiveOp` compositions whose
   types capture the same information concretely.
4. `parallel.rs`, `rayon`, `sha2`, `Fnv1aHasher16`, the entire `Boundary`
   trait, and `MiningSession` (the orchestrator that drives the
   non-Prism rayon loop) are deleted. Their roles are absorbed by
   `pipeline::run` and the foundation traversal. Tip-staleness, extranonce
   rolling, and progress reporting belong to `prism-btc-node` as glue
   around `getblocktemplate`/`submitblock`, not inside prism-btc's pipeline.
5. The `prism-btc` (top-level) crate becomes the public façade
   re-exporting `prism-btc-shapes` + `prism-btc-pipeline`. The current
   `prism-btc-reduction` crate is dissolved; its surviving content
   (the genesis CompileUnit) folds into `prism-btc-pipeline`.

The reconciliation is not staged or phased: every step above must hold
together for the architecture to be coherent. Partial reconciliation
leaves the system non-conforming.

---

## 10. Open questions for the architect (you)

The following points the wiki does not pin down and prism-btc must commit
to. The current draft makes assumptions, which are flagged here for
explicit resolution:

1. **Is `kernel::convergence` the right foundation primitive for the
   W32 fiber traversal?** The Building Block View names "convergence
   machinery" inside kernel without specifying its surface. If
   convergence is the wrong primitive, name the correct one.
2. **Is `2^32 + ε` an acceptable `HostBounds::TRACE_MAX_EVENTS` for a
   complete W32 traversal trace?** A naive trace records every fiber
   visit; a stricter discipline records only admit events. The latter
   greatly reduces trace size but loses replayability of the traversal
   path. Wiki gives no preference.
3. **Should `prism-btc-shapes` and `prism-btc-ops` be separate crates
   or one?** The split reflects the foundation's split between bridge
   (shapes) and primitives (ops). Either is wiki-conforming; one crate
   is simpler.
4. **Does prism-btc declare `Sha256dHasher` as the `Hasher` axis, or
   does it use a separate cheaper hasher (e.g. BLAKE3) for the
   `ContentFingerprint` of the certificate, with `Sha256dProjection`
   used inside the pipeline as a `PrimitiveOp` composition only?**
   ADR-010 lets us pick any conforming Hasher; the choice is
   independent of the σ-projection. The current draft conflates them;
   they should likely be separate.
