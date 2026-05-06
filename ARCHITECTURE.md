# prism-btc: Defined Architecture

> **Status:** Normative for prism-btc. This document is the authoritative
> specification of what prism-btc is, what it claims, and how it realises
> those claims through Prism + uor-foundation. The repository state is
> reconciled to this document, not the other way around.
>
> **Frame of reference:** the [UOR-Framework wiki](https://github.com/UOR-Foundation/UOR-Framework/wiki),
> which is itself the normative specification of Prism — the three-crate
> system (`uor-foundation`, `prism`, `prism-verify`) realising the boundary
> properties TC-01..TC-06 and the architectural commitments ADR-001..ADR-018.

---

## 1. The claim: real-time structural inference

prism-btc is a **real-time inference engine for Bitcoin proof-of-work**,
realised as a Prism application. The artifact is the binary `prism-mine`,
which produces blocks accepted byte-for-byte by Bitcoin Core.

The load-bearing distinction between prism-btc and a traditional miner is
that the satisfying nonce is not *searched for* — it is *inferred at
runtime by deterministic structural traversal* of a foundation-defined
shape. Specifically:

- The mining problem is declared as a `ConstrainedTypeShape` whose
  inhabitants are the 32-byte digests dominated by the target, derived
  from the W32 nonce fiber by the σ-projection composition.
- `prism::pipeline::run` consumes a `Validated<CompileUnit, FinalPhase>`
  carrying that shape declaration and produces a `Grounded<T>` paired
  with a `Trace` (Runtime View Scenario 1). The pipeline is, by
  framework definition, *"a deterministic traversal of input data
  through pre-compiled stages"* (Runtime View, Scenario 1) — not a
  search.
- Determinism + finite domain (`|W32| = 2^32`) + unique-first-admission
  in the structural ordering of the W32 ring together mean: given the
  template prefix and the foundation `Hasher` substitution, the
  pipeline derives the same nonce on every invocation. There is no
  randomness, no choice, no "search and check." The answer is uniquely
  determined by the structure; the pipeline computes it.

This is what "real-time inference" means here:

- **Inference**, not search: the answer is structurally entailed by
  (template, target, σ-projection composition). The pipeline derives it.
- **Real-time**, not compile-time: templates arrive at runtime; the
  inference runs on user hardware at the moment a block is to be mined,
  with time bounded by the structural complexity of the inference task
  (the count of fiber points the deterministic traversal visits before
  admission). It is **not** a precomputed table, **not** an oracle
  query, **not** a service call — every stage executes locally
  (TC-06).
- **Bit-identical output to traditional mining**, by a different path:
  the (header, nonce) pair the pipeline emits, when serialised under
  Bitcoin's standard 80-byte layout and SHA-256d-projected by Bitcoin
  Core, satisfies the protocol's target. The block is accepted by
  `submitblock` exactly as any other miner's block would be. What
  differs is the path: every step of prism-btc's derivation is a
  composition of foundation `PrimitiveOp` discriminants and a sealed
  `pipeline::run` traversal — never an opaque crate import, never a
  hand-rolled loop.

What prism-btc does **not** claim:

- It does **not** invert SHA-256, escape proof-of-work, or weaken any
  cryptographic primitive. SHA-256 is a one-way digest under the
  framework's `Hasher` substitution-axis contract (ADR-010). Per §1's
  inference framing the digest is *evaluated structurally* on each
  fiber point that the traversal visits; "no per-candidate evaluation"
  was ruled out earlier in the discussion. The structural-inference
  framing is about the *path*, not about avoiding the digest's
  computational cost.
- It does **not** speed up mining. The number of digest evaluations
  required to reach an admitting fiber point is the same expectation
  as any other miner at the same target.
- It does **not** introduce primitive operations beyond the foundation's
  closed set (ADR-013). Every Bitcoin verb used by prism-btc reduces
  to a `PrimitiveOp` composition.

The value is architectural and epistemic: a mined block carries with
it a `Trace` that an independent verifier can replay (TC-05) without
invoking SHA-256, without invoking any decider written by prism-btc's
author, and without contacting any service — yielding a
`Certified<GroundingCertificate>` that the trace's claimed nonce was
derived under the declared shape via the structural traversal the
trace records.

---

## 2. Conceptual model

> Cross-reference: this section follows the [UOR-Framework wiki's Conceptual-Model page](https://github.com/UOR-Foundation/UOR-Framework/wiki/Conceptual-Model)
> convention — OPM (ISO 19450) entities and processes declared in OPL.
> The wiki's Prism-level entities (Application Author, Application User,
> Rust Toolchain, Prism, Trace, etc.) are inherited; this section
> declares prism-btc's specialisations and adds Bitcoin-domain entities
> and processes.

### 2.1 Inherited Prism entities (from the wiki)

`Application Author` is a stakeholder. `Application Author` distributes
`prism-mine`. `prism-mine` is an `Application` (in the wiki's sense:
the executable a Prism application author distributes).

`Application User` is a stakeholder. `Application User` runs `prism-mine`.
`Application User` may invoke `prism-verify::certify_from_trace` on
the resulting `Trace`.

`Rust Toolchain` is an enabler. `Rust Toolchain` compiles `prism-mine`,
exhibiting compile-time UORassembly enforcement (TC-04, ADR-006).

`Prism` is a system. `Prism` consists of `uor-foundation`, `prism`, and
`prism-verify`. `Prism` exhibits the boundary properties TC-01..TC-06.

`Trace` is an output object. `Trace` consists of a fixed sequence of
`TraceEvent` values, a `ContentFingerprint`, a hasher identifier, and
a format version (per Building Block View, bridge::trace::Trace).

`Grounded<T>` is a sealed object. `pipeline::run` yields `Grounded<T>`
and `Trace` simultaneously (Runtime View Scenario 1, step 8).

`Certified<GroundingCertificate>` is a sealed object.
`certify_from_trace` yields `Certified<GroundingCertificate>` from a
`Trace` and a hasher instance (Runtime View Scenario 2, step 9).

### 2.2 prism-btc-specific entities

`Bitcoin Core node` is an external system. `prism-mine` requires a
`Bitcoin Core node` for `getblocktemplate` and `submitblock`. The
node is **outside Prism's scope** (ADR-004, distribution channel
external to Prism — applied here to the upstream block-template source
and downstream block-submission sink).

`Block template` is an input object obtained from a `Bitcoin Core
node`. `Block template` consists of: a previous-block hash, a target
`bits` value, a coinbase value, a transaction list, a height, a
current time, and the segwit witness commitment script.

`Coinbase transaction` is a derived object. `CoinbaseConstruction`
(§4.4) yields a `Coinbase transaction` from a `Block template`, an
`Extranonce`, and a payout address.

`Extranonce` is a free coordinate (`u64` value space). `Extranonce`
exhibits resolution by the session's outer loop (§6.1).

`Merkle root` is a derived object. `MerkleRootDerivation` (§4.5)
yields a `Merkle root` from the `Coinbase transaction` txid and the
`Block template`'s transaction txid list.

`Template prefix` is a derived object. `HeaderSerialization` (§4.3)
yields a 76-byte `Template prefix` from `(version, prev_hash,
merkle_root, timestamp, bits)`.

`Nonce` is a free coordinate (W32 = `Z/(2^32)Z` value space).
`NonceFiberTraversal` (§4.6) resolves `Nonce` by a deterministic W32
traversal.

`Block digest` is a derived object. `Sha256dProjection` (§4.2) yields
a `Block digest` from a `Template prefix` and a `Nonce`.

`Mining inference` is a process. `Mining inference` consists of
`HeaderSerialization`, `Sha256dProjection`, and admission against
`TargetSubBundle`. `Mining inference` is realised by exactly one
invocation of `pipeline::run` per (`Template prefix`, `Extranonce`)
pair.

`Mining session` is a process. `Mining session` consists of: an
outer loop over `Block template`s and `Extranonce`s; one or more
invocations of `Mining inference`; one `submitblock` call per
admitted result. `Mining session` is realised by `prism-btc-node`'s
`Session` (§7.5).

`Mined block` is an output object. `Mining session` yields a `Mined
block` (the wire-format Bitcoin block bytes) and an accompanying
`Trace`. The `Bitcoin Core node` confirms the `Mined block` by
returning a non-error result from `submitblock`.

### 2.3 Inherited Prism processes (from the wiki)

`Grounding` is a process. `Grounding` admits host bytes to a `Datum`
or rejects with a typed impossibility witness.

`CompileUnitConstruction` is a process. `CompileUnitConstruction`
yields `Validated<CompileUnit, FinalPhase>` from a `Datum`, a
`ConstrainedTypeShape` impl, and substitution-axis selections.

`PipelineRun` is a process. `PipelineRun` yields `Grounded<T>` and
`Trace` from `Validated<CompileUnit, FinalPhase>`.

`CertificateEmission` is a process. `CertificateEmission` invokes the
`Hasher` exactly once to compute the `ContentFingerprint`.

`TraceReplay` is a process. `TraceReplay` is realised by
`certify_from_trace`. `TraceReplay` does not invoke the `Hasher`'s
hashing method and does not invoke the application author's
deciders (TC-05).

### 2.4 prism-btc-specific processes

The six prism-btc-specific processes are the `PrimitiveOp` compositions
declared in §4. Each is a process that yields its specified output
object from its specified input objects, realised entirely as a
foundation `PrimitiveOp` composition (closed under ADR-013):

| Process | Input objects | Output object | §-ref |
|---|---|---|---|
| `Sha256Compression` | 512-bit message block, 256-bit prior state | 256-bit working state | §4.1 |
| `Sha256dProjection` | 80-byte serialised header | 32-byte `Block digest` | §4.2 |
| `HeaderSerialization` | (version, prev_hash, merkle_root, timestamp, bits, nonce) | 80-byte serialised header | §4.3 |
| `CoinbaseConstruction` | (height, extranonce, payout_address, coinbase_value, witness_commitment) | `Coinbase transaction` | §4.4 |
| `MerkleRootDerivation` | coinbase txid, other-tx txids | `Merkle root` | §4.5 |
| `NonceFiberTraversal` | template prefix, target | (winning nonce, winning digest) ∨ no-match | §4.6 |

### 2.5 Object-process relationships (OPL)

The complete prism-btc OPL declarations:

```
Application Author distributes prism-mine.
prism-mine requires Bitcoin Core node.
Mining session invokes Mining inference.
Mining session invokes CoinbaseConstruction.
Mining session invokes MerkleRootDerivation.
Mining session invokes HeaderSerialization.
Mining session invokes submitblock.
Mining inference is PipelineRun.
Mining inference invokes Grounding.
Mining inference invokes NonceFiberTraversal.
NonceFiberTraversal invokes HeaderSerialization (per fiber visit).
NonceFiberTraversal invokes Sha256dProjection (per fiber visit).
NonceFiberTraversal yields (Nonce, Block digest) ∨ no-match.
Sha256dProjection invokes Sha256Compression (twice).
CertificateEmission invokes Hasher (= Sha256dHasher).
CertificateEmission yields ContentFingerprint.
PipelineRun yields Grounded<T> and Trace simultaneously.
TraceReplay yields Certified<GroundingCertificate> from Trace and hasher_instance.
TraceReplay does not invoke Sha256dProjection.
TraceReplay does not invoke NonceFiberTraversal.
TraceReplay does not invoke Hasher.
```

Every OPL declaration above is grounded in either a wiki normative
source (Runtime View, Building Block View, ADR-*, TC-*) or a §-ref
back to this document's specification.

---

## 3. Substitution-axis bindings (ADR-007, ADR-010, ADR-018)

Every prism application binds the three substitution axes. prism-btc
binds them as follows:

### 3.1 `HostTypes`

prism-btc selects `uor_foundation::DefaultHostTypes`. The host string
type is `&'static str`; the host byte type is `u8`; integer types are
the standard fixed-width Rust types. No application-specific host-type
selections are required.

### 3.2 `HostBounds = prism_btc::PrismBtcBounds`

A unit struct in `prism-btc::shapes::bounds` with these associated
constants (ADR-018: every capacity bound flows through `HostBounds`):

| Constant | Value | Justification |
|---|---|---|
| `FINGERPRINT_MIN_BYTES` | `32` | matches SHA-256 output width; below this is insufficient for a 256-bit collision-resistant content fingerprint |
| `FINGERPRINT_MAX_BYTES` | `32` | fixed: prism-btc declares one Hasher (§3.3) at exactly 32 bytes |
| `TRACE_MAX_EVENTS` | `64` | bounds the per-`pipeline::run` trace at a small constant — the pipeline emits one event per stage transition (§6.4), not one per fiber visit. Headroom is for future stage subdivisions in the foundation. |
| `WITT_LEVEL_MAX_BITS` | `32` | the W32 nonce ring is the largest algebraic level the prism-btc principal data path computes against. |

`TRACE_MAX_EVENTS = 64` is a binding architectural commitment. It
forbids any implementation strategy that records every fiber visit.
The traversal of 2^32 fiber points is a *single* `PipelineRunEvent`
that carries (winning fiber index, count of fiber visits, terminal
digest) as scalar fields — not a sequence of per-visit events.
Replayability (TC-05) is preserved because the event's structural
validation depends on the scalar fields, not on enumerating visits.

### 3.3 `Hasher = prism_btc::Sha256dHasher`

A foundation-conforming `Hasher` impl whose body is a `PrimitiveOp`
composition (§4.1). Concrete properties (ADR-010):

- Deterministic: same input bytes → same output bytes, on every
  hardware, every Rust toolchain version, every build profile.
- Fixed output: `OUTPUT_BYTES = 32`.
- Distinct identifier: `HASHER_IDENTIFIER` is the IRI
  `https://prism.btc/hasher/Sha256dHasher` (a u32 derived from this
  IRI by foundation's identifier-derivation discipline).
- Idempotent under truncation: trivially, since `OUTPUT_BYTES =
  FINGERPRINT_MAX_BYTES`.

`Sha256dHasher` is bound to **two distinct foundation roles**, and the
architecture treats them as separate concerns (resolving an earlier
draft's conflation):

- **As the `Hasher` substitution axis**, invoked exactly once per
  `pipeline::run` at certificate-emission time (Runtime View Scenario 1
  step 9) to compute the `ContentFingerprint` over the CompileUnit's
  canonical byte layout.
- **As the σ-projection inside the pipeline's `PipelineRun` stage**,
  invoked on each fiber point during the W32 traversal as part of
  `Sha256dProjection` (§4.2).

These are the same algorithm, two roles. The trace records the
σ-projection invocations as part of the `PipelineRunEvent`; the Hasher
is identified but not invoked at replay (TC-05).

---

## 4. The Bitcoin verbs as `PrimitiveOp` compositions (ADR-014)

ADR-014 commits prism to ship vocabulary, not pre-implemented
operations: "authors declare operations as `PrimitiveOp` compositions."
prism-btc declares **six** compositional operations and **two**
constrained-type shapes that fully cover the mining computation. All
six compositions are closed under foundation's primitive set
(ADR-013): bit-rotation, integer-handling, lookup, content-comparison,
depth-projection, observable-arithmetic.

### 4.1 `Sha256Compression` (`PrimitiveOp` composition)

The 64-round SHA-256 compression function on a 512-bit message block.
Declared as:

- 8 working-state words (`a..h`) initialised by `lookup` against the
  foundation-fixed initial-state vector.
- 64 rounds, each composing `bit-rotation` (`Σ0 Σ1 σ0 σ1`),
  `integer-handling` (modular `add`, `xor`, `and`, bitwise `not`),
  and `lookup` against the K-round-constants table.
- Final `integer-handling` add of the working state into the input
  state vector.

Output: a 256-bit working state (8 × u32 words). Total, pure, no new
primitives required.

### 4.2 `Sha256dProjection` (`PrimitiveOp` composition)

The σ-projection: `Sha256Compression` applied twice on the canonical
80-byte header padded per the SHA-256 specification, followed by
`depth-projection` to extract the 32 most-significant bytes in
Bitcoin's display order (byte-reversed from the SHA-256 native
output). Closure: composition of `Sha256Compression` (§4.1) +
`depth-projection`.

`Sha256dHasher` (§3.3) is the `Hasher`-trait implementation that
internally invokes `Sha256dProjection` on the canonical CompileUnit
byte layout when the foundation pipeline calls it for fingerprinting.

### 4.3 `HeaderSerialization` (`PrimitiveOp` composition)

The fixed 80-byte wire layout of a Bitcoin block header. Declared as a
`depth-projection` composition that takes `(version, prev_hash,
merkle_root, timestamp, bits, nonce)` and emits the canonical byte
sequence:

```
[0..4)   version    (LE u32, integer-handling → depth-projection)
[4..36)  prev_hash  (32 bytes, depth-projection)
[36..68) merkle_root (32 bytes, depth-projection)
[68..72) timestamp  (LE u32, integer-handling → depth-projection)
[72..76) bits       (LE u32, integer-handling → depth-projection)
[76..80) nonce      (LE u32, integer-handling → depth-projection)
```

No primitive beyond `integer-handling` and `depth-projection` is
required.

### 4.4 `CoinbaseConstruction` (`PrimitiveOp` composition)

The Bitcoin coinbase transaction is the first transaction in every
block. Its scriptSig contains a BIP34 height push, an extranonce
field, and arbitrary data ("prism-btc" tag). prism-btc declares:

- `BIP34HeightPush`: `integer-handling` composition emitting the
  CompactSize-encoded block-height bytes.
- `ExtranoncePush`: `integer-handling` + `depth-projection` emitting
  little-endian u64 bytes.
- `ScriptSigAssembly`: `depth-projection` concatenating the height
  push, extranonce push, and the literal-byte tag from a `lookup`
  table.
- `CoinbaseTxAssembly`: `depth-projection` over the transaction
  envelope (version, inputs, outputs, lock_time, witnesses) producing
  the canonical serialised coinbase bytes.

Closure: `integer-handling` + `depth-projection` + `lookup`.

### 4.5 `MerkleRootDerivation` (`PrimitiveOp` composition)

Pairwise SHA-256d up the transaction tree. Declared as:

- For each transaction, `Sha256dProjection` (§4.2) applied to the
  serialised tx bytes → txid.
- A folded composition of `Sha256dProjection` over pairs at each tree
  level until a single 32-byte root remains.

Closure: `Sha256dProjection` (§4.2). No new primitives.

### 4.6 `NonceFiberTraversal` (prism-btc runtime)

The W32 nonce fiber traversal — the structural inference's
load-bearing operation. **prism-btc is the prism implementor for the
Bitcoin use case; the traversal is therefore prism-btc's runtime, not
a foundation-supplied primitive.** Foundation provides the substrate
(sealed types, `Hasher` and `HostBounds` traits, `Term` and
`PrismitiveOp` vocabulary, mint primitives, trace structure); prism-btc
provides the runtime that traverses the typed structure declared via
that substrate.

Structural declaration (compile-time):

- **Index domain**: the W32 ring. Declared at the type level via
  `WittLevel::W32` and the `Term::Application { operator: PrimitiveOp::Succ, .. }`
  successor composition.
- **Per-index map**: `Sha256dProjection ∘ HeaderSerialization`,
  declared at the type level as a `Term::Application` chain over
  prism-btc's chosen `PrimitiveOp` decomposition. The runtime that
  evaluates the composition is `prism_btc::ops::sigma::sha256d`
  (pure-Rust SHA-256d, no external crate).
- **Halt predicate**: admission of the projected 32-byte digest to
  `TargetSubBundle` (§4.8) — encoded structurally as `ConstraintRef`s
  on the shape, evaluated at runtime by prism-btc's traversal.

Runtime evaluation (prism-btc's job):

- The traversal visits W32 indices in canonical successor order
  starting at 0. prism-btc's runtime walks the fiber, applying the
  σ-projection at each point and testing admission. On first admit,
  the traversal terminates and prism-btc invokes foundation's
  `pipeline::run` (or `pipeline::run_const`) to mint a
  `Grounded<ConstrainedTypeInput, MiningTag>` certifying the shape.
- On exhaustion, prism-btc returns `MiningFailure::NoMatch`; the
  bitcoind boundary (`prism-btc-node`) increments the extranonce and
  re-invokes `prism_btc::mine` with a new `TemplatePrefixDatum`.
- Determinism: same template + same extranonce + same `Sha256dHasher`
  → same terminal index. No randomness.

Parallelism: prism-btc's runtime MAY partition the W32 ring across
threads (the natural coset partition over `Z/(2^32)Z`); first-finder
wins. This is a runtime implementation detail and does not change
the categorical structure.

**This is the operation that replaces the rayon for-loop currently
in `prism-btc-reduction/src/parallel.rs`.** The replacement is not a
foundation primitive — foundation never claimed to ship one — but a
prism-btc runtime that respects the foundation-typed structural
declaration (Term composition + ConstrainedTypeShape constraints).
The categorical routing claim holds at the type level: the shapes
and Term compositions declare the structure; the prism-btc runtime
walks it.

### 4.7 `TemplatePrefixShape` (`ConstrainedTypeShape`)

The input shape: an admission constraint on the 76-byte
template-prefix bytes (header bytes [0..76), the nonce field excluded).
- `IRI`: `https://prism.btc/shape/TemplatePrefix`
- `SITE_COUNT`: 76
- `CONSTRAINTS`: empty list (any well-formed 76-byte prefix admits;
  validity is enforced by the upstream `getblocktemplate` boundary,
  outside Prism's scope per ADR-004).

The Grounding for `TemplatePrefixShape` admits raw host bytes if the
slice length is exactly 76; otherwise emits a typed impossibility
witness.

### 4.8 `TargetSubBundle` (`ConstrainedTypeShape`)

The output shape: the 32-byte digest sub-bundle dominated by the
target.
- `IRI`: `https://prism.btc/shape/TargetSubBundle`
- `SITE_COUNT`: 32
- `CONSTRAINTS`: a list of `content-comparison` `ConstraintRef`
  entries, one per byte position, expressing "the digest's value at
  this byte ≤ the target's value at this byte under the lexicographic
  comparison rule for the leading non-equal byte." This is the
  Bitcoin protocol's target-satisfaction rule.

The `target` parameter (4-byte compact nBits) becomes part of the
CompileUnit's term tree at construction time, decoded into a 32-byte
target value via the standard `Target::to_bytes` rule (itself a
`depth-projection` + `integer-handling` composition).

The Grounding for `TargetSubBundle` admits a 32-byte candidate if and
only if its bytes ≤ the target bytes lexicographically. This is the
halt predicate of `NonceFiberTraversal` (§4.6).

---

## 5. The mining inference task

A single `pipeline::run` invocation infers a satisfying nonce for a
single (template, extranonce) pair. The structural picture is:

```
Inputs (Datum-admitted host bytes):
  TemplatePrefixDatum ← Grounding(76-byte header prefix)
  Target              ← Grounding(4-byte nBits) → 32-byte target value
  Extranonce          ← Grounding(8-byte u64)

CompileUnit:
  result_type   = TargetSubBundle (the constrained type whose
                  inhabitants are admitting digests)
  root_term     = the kernel::convergence term over W32 (§4.6)
                  composing HeaderSerialization → Sha256dProjection
                  → admission against TargetSubBundle
  witt_level    = W32
  budget        = 2^32 (the cardinality of the fiber)

pipeline::run output:
  Grounded<ConstrainedTypeInput, MiningTag>:
    inner Datum    = the 32-byte admitting digest
    derivation     = the foundation Derivation recording the
                     PrimitiveOp composition that produced the digest
                     from (TemplatePrefixDatum, winning nonce)
    free_rank      = 0 (the W32 fiber's free coordinate has been
                     resolved to a specific value)
    content fp     = Sha256dHasher applied to canonical layout
  Trace:
    TraceEvent sequence (§6.4) carrying the winning nonce as a scalar
    in the PipelineRunEvent.
```

The `MiningTag` phantom (per the foundation's `Grounded<T, Tag>`
contract; see §6) marks this Grounded as a Bitcoin block solution at
the type level — the same role the previous `BlockHashTag` filled,
unchanged.

---

## 6. The pipeline shape for one mining session

### 6.1 The session's outer loop

A "mining session" is the public-facing operation: from user-supplied
RPC credentials and payout parameters, run until a block is mined and
accepted, or until the user cancels. The session's outer loop lives
in `prism-btc-node`, the bitcoind-boundary crate; its responsibilities
are:

1. Acquire a fresh template from `bitcoind` via `getblocktemplate`.
2. Construct the coinbase via `CoinbaseConstruction` (§4.4).
3. Derive the merkle root via `MerkleRootDerivation` (§4.5).
4. Form the 76-byte template prefix via `HeaderSerialization` (§4.3),
   nonce field zero-filled.
5. Invoke `pipeline::run` once with that prefix and the current
   extranonce.
6. On success: assemble the wire-format block and submit via
   `submitblock`.
7. On `PipelineFailure::NoMatch`: increment the extranonce and goto 2.
8. Between iterations: poll `getbestblockhash`; if the chain has
   advanced, abandon the current template and goto 1.

### 6.2 The pipeline invocation (Runtime View Scenario 1)

Per `pipeline::run` invocation, the framework's Scenario 1 sequence
applies, instantiated for prism-btc as:

1. Application has the 76-byte prefix bytes.
2. Application invokes prism-btc's `Grounding<TemplatePrefixShape>`
   on the prefix → `TemplatePrefixDatum`.
3. Application builds a `CompileUnit` via `CompileUnitBuilder`,
   declaring (§4) `result_type = TargetSubBundle`, `root_term = the
   convergence term`, `witt_level = W32`, `budget = 2^32`,
   `target_domains = [ComposedAlgebraic]`.
4. The builder transitions through validation phases; emits
   `Validated<CompileUnit, FinalPhase>` or a structured `ShapeViolation`.
5. Application invokes `pipeline::run::<ConstrainedTypeInput, M, H>`
   with `M = BinaryGroundingMap` (the byte-ingest morphism kind for
   the prefix admission, foundation-supplied per ADR-007/ADR-010
   bound) and `H = Sha256dHasher` (§3.3).
6. The pipeline executes the `NonceFiberTraversal` (§4.6) — the
   deterministic W32 inference — emitting a `Grounded<...>` whose
   inner Datum is the admitting digest and a `Trace` recording the
   five stage transitions (§6.4).
7. The Hasher is invoked exactly once at certificate emission to
   compute the `ContentFingerprint` over the canonical layout. The
   Hasher's identifier is recorded in the trace.
8. Application receives `(Grounded<...>, Trace)`.

### 6.3 Path singularity (TC-03)

There is exactly one path from the host-byte input to a `Grounded<T>`:
through `Grounding<TemplatePrefixShape>` → `Datum` → `CompileUnitBuilder`
→ `Validated<CompileUnit, FinalPhase>` → `pipeline::run`. There is no
alternative constructor for `Grounded<...>` in prism-btc; the type is
sealed in the foundation/prism crates.

A mining session may invoke `pipeline::run` multiple times (once per
extranonce), but each invocation traverses the singular path. TC-03
prohibits second-pathways, not multiple traversals.

### 6.4 Trace structure for one `pipeline::run`

The trace is a sequence of exactly five `TraceEvent`s, one per stage
transition (Runtime View Scenario 1 plus prism-btc's stage
specialisations):

| # | Variant | Carries |
|---|---|---|
| 1 | `DatumAdmissionEvent` | TemplatePrefixDatum address; admission witness |
| 2 | `CompileUnitConstructionEvent` | result-type IRI (TargetSubBundle); root-term hash; witt-level; budget; target-domains |
| 3 | `ValidationPhaseEvent` | sequence of phase transitions reaching FinalPhase |
| 4 | `PipelineRunEvent` | the inference result: winning nonce (u32 scalar), winning digest (32 bytes), fiber-visit count (u64 scalar), derivation root address. Note: per §3.2 (`TRACE_MAX_EVENTS = 64`), this single event records the entire W32 traversal as scalar fields, not as a per-visit subsequence. |
| 5 | `CertificateEmissionEvent` | hasher identifier; ContentFingerprint bytes |

Trace size is bounded by a small constant (~64 events × ~few hundred
bytes = ~few KB), independent of fiber-visit count. This is the design
that makes replay tractable (TC-05): the verifier walks five events,
not 2^32.

### 6.5 Extranonce rolling, tip changes, and TC-03

Extranonce rolling and tip-change handling live in `prism-btc-node`
(§6.1, the outer loop). They are **not** inside `pipeline::run`; they
are the boundary's responsibility. Per invocation of `pipeline::run`:

- The (template, extranonce) pair is fully determined before the call.
- The pipeline traverses W32 deterministically.
- On admission (Grounded), the boundary submits.
- On exhaustion (`PipelineFailure::NoMatch`), the boundary increments
  extranonce and re-invokes `pipeline::run` with a new
  `TemplatePrefixDatum` (re-derived merkle root).
- On tip change between invocations, the boundary discards the in-flight
  state and starts fresh from §6.1 step 1.

The pipeline itself has no abort mechanism. A `pipeline::run` in
flight runs to completion; if its result is for a stale parent, the
boundary discards the result without submitting. At realistic CPU
hash rates (~30 MH/s on 8 cores), one W32 traversal completes in
~140 seconds; tip changes on testnet4/mainnet average 600 seconds, so
the discard rate is acceptable.

### 6.6 Replay (Runtime View Scenario 2)

A user receives `(Trace, Sha256dHasher_identifier)` out-of-band. The
user invokes `prism-verify::certify_from_trace(trace, hasher_instance)`.
Per Scenario 2:

1. The verifier decodes the trace bytes against
   `TRACE_REPLAY_FORMAT_VERSION`.
2. Confirms `hasher_identifier` matches the supplied hasher's
   identifier.
3. Walks the five `TraceEvent`s structurally, validating that each
   event's variant is well-typed against its successor (e.g., a
   `DatumAdmissionEvent` must be followed by a
   `CompileUnitConstructionEvent` carrying the same Datum address;
   `PipelineRunEvent.derivation_root` must match the
   `CompileUnitConstructionEvent.root_term`; etc.).
4. Confirms the `PipelineRunEvent`'s scalar nonce matches the digest
   that, under the trace's recorded structural relationships, the
   admitting fiber point produced.
5. **Does not invoke any hasher's hashing method** (TC-05); the
   hasher is provided so its identity can be confirmed.
6. **Does not invoke any decider written by prism-btc** (TC-05);
   `Sha256dProjection`, `NonceFiberTraversal`, `TargetSubBundle`'s
   admission rule are not run.
7. On success, mints `Certified<GroundingCertificate>`. On failure,
   emits a structured `ReplayError`.

The `Certified` output is a *structural* attestation: the trace is
internally consistent; its claimed nonce is a coherent record of a
valid pipeline traversal. It is **not** a re-derivation of the digest
or a re-check of the proof-of-work — that re-derivation is the
domain of Bitcoin Core's own validator (which any node receiving the
block performs independently). prism-btc's certification and
Bitcoin's are two distinct claims; they happen to commit to the same
nonce by construction.

---

## 7. Public API surface

This section enumerates the exact Rust signatures the reconciled
prism-btc presents. They are normative: any deviation between code
and these signatures is non-conforming.

### 7.1 `prism_btc` crate (the domain layer)

```rust
// src/lib.rs

/// The application-author entry point. Constructs a CompileUnit
/// declaring the mining problem, validates it, invokes
/// `prism::pipeline::run` exactly once, returns the witness + trace
/// or a structured failure.
pub fn mine(
    prefix: TemplatePrefix,       // 76-byte header prefix, nonce-field-zero
    extranonce: Extranonce,       // u64 free-coordinate value
    target: Target,               // compact nBits, decoded internally
) -> Result<MiningOutcome, MiningFailure>;

/// The grounded mining witness + accompanying trace.
pub struct MiningOutcome {
    pub witness: MiningWitness, // alias for Grounded<ConstrainedTypeInput, MiningTag>
    pub trace:   prism::Trace,
    pub nonce:   u32,            // the W32 fiber index admitted (also carried in witness/trace)
    pub digest:  [u8; 32],       // the admitting block hash
}

/// Type alias for the certificate prism-btc returns.
pub type MiningWitness = uor_foundation::enforcement::Grounded<
    uor_foundation::enforcement::ConstrainedTypeInput,
    MiningTag,
>;

/// Phantom tag distinguishing prism-btc's Grounded from other domains.
pub struct MiningTag;

/// Failure modes from `mine`. The pipeline itself is infallible
/// (per the const-validated CompileUnit; see §13); the only failures
/// are exhaustion of the W32 fiber for the supplied (prefix, extranonce)
/// pair, or upstream bytes that fail Grounding.
#[derive(Debug)]
pub enum MiningFailure {
    /// All 2^32 fiber points exhausted; no nonce admits this prefix.
    NoMatch,
    /// The prefix bytes were ill-formed for `TemplatePrefixShape`.
    InvalidPrefix,
}

/// Newtypes / domain types.
pub struct TemplatePrefix(pub [u8; 76]);
pub struct Extranonce(pub u64);
pub struct Target { pub bits: u32 }

// Submodules (private internals; only `mine` and types above are public)
mod ops {            // §4 PrimitiveOp compositions
    pub mod sha256;
    pub mod traversal;
    pub mod header;
    pub mod merkle;
    pub mod coinbase;
}
mod shapes {         // §3 substitution-axis selections + §4.7-4.8 shapes
    pub mod hasher;        // Sha256dHasher
    pub mod bounds;        // PrismBtcBounds
    pub mod prefix;        // TemplatePrefixShape + its Grounding
    pub mod target_sub_bundle;  // TargetSubBundle + its Grounding
}
mod compile_unit;    // §5 the per-invocation CompileUnit constructor
```

There is no `Boundary` trait. There is no `BoundaryDecodeError`. There
is no `MorphismKind` re-export. There is no `BlockCertificate<Sigma>`.
There is no `MiningRound`. There is no `MiningSession`. The domain
layer's only public verbs are `mine` and the value-type
constructors.

### 7.2 `prism_btc::Sha256dHasher` (foundation `Hasher` impl)

```rust
// src/shapes/hasher.rs

pub struct Sha256dHasher { /* internal SHA-256d state */ }

impl uor_foundation::enforcement::Hasher for Sha256dHasher {
    const OUTPUT_BYTES:      usize = 32;
    const HASHER_IDENTIFIER: u32   = /* identifier derived from
        "https://prism.btc/hasher/Sha256dHasher" */;
    fn initial() -> Self;
    fn fold_byte(self, byte: u8) -> Self;
    fn finalize(self) -> [u8; 32];
}
```

The body of `fold_byte` and `finalize` is a `PrimitiveOp` composition
(via the §4.1–§4.2 declarations); the trait impl is the foundation-side
binding. No `sha2` import; no opaque hashing code.

### 7.3 `prism_btc::PrismBtcBounds` (foundation `HostBounds` impl)

```rust
// src/shapes/bounds.rs

pub struct PrismBtcBounds;

impl uor_foundation::HostBounds for PrismBtcBounds {
    const FINGERPRINT_MIN_BYTES: usize = 32;
    const FINGERPRINT_MAX_BYTES: usize = 32;
    const TRACE_MAX_EVENTS:      usize = 64;
    const WITT_LEVEL_MAX_BITS:   u32   = 32;
}
```

### 7.4 Grounding impls

```rust
// src/shapes/prefix.rs

pub struct TemplatePrefixShape;

impl uor_foundation::pipeline::ConstrainedTypeShape for TemplatePrefixShape {
    const IRI:         &'static str = "https://prism.btc/shape/TemplatePrefix";
    const SITE_COUNT:  usize        = 76;
    const CONSTRAINTS: &'static [uor_foundation::pipeline::ConstraintRef] = &[];
}

/// The Grounding admits a 76-byte slice as a TemplatePrefixDatum.
/// Foundation pattern: a `Grounding` impl yields `Datum` or a typed
/// rejection witness.
pub struct TemplatePrefixGrounding;
impl uor_foundation::Grounding<TemplatePrefixShape> for TemplatePrefixGrounding {
    fn admit(bytes: &[u8]) -> Result<Datum, GroundingRejection>;
}
```

```rust
// src/shapes/target_sub_bundle.rs

pub struct TargetSubBundle;

impl uor_foundation::pipeline::ConstrainedTypeShape for TargetSubBundle {
    const IRI:         &'static str = "https://prism.btc/shape/TargetSubBundle";
    const SITE_COUNT:  usize        = 32;
    const CONSTRAINTS: &'static [uor_foundation::pipeline::ConstraintRef] = &[
        // 32 ConstraintRef::ContentComparison entries, one per byte position,
        // expressing the lexicographic target rule. Materialised at compile
        // time; the target value is bound to the CompileUnit's term tree
        // (template-dependent, runtime-bound).
    ];
}
```

### 7.5 `prism_btc_node::Session`

```rust
// crates/prism-btc-node/src/session.rs

pub struct Session {
    rpc_url:        String,
    auth:           bitcoincore_rpc::Auth,
    payout_address: bitcoin::Address,
    network:        bitcoin::Network,
    cfg:            SessionConfig,
}

pub struct SessionConfig {
    pub tip_poll:       Duration,    // poll bestblockhash every N
    pub progress_every: Duration,    // emit a hash-rate report every N
}

impl Session {
    pub fn new(
        rpc_url:        &str,
        auth:           bitcoincore_rpc::Auth,
        payout_address: &str,
        network:        bitcoin::Network,
        cfg:            SessionConfig,
    ) -> Result<Self, SessionInitError>;

    /// The session's outer loop (§6.1). Runs until a block is mined
    /// and `submitblock` returns success, or until `cancel` is set.
    /// Internally calls `prism_btc::mine` once per (template, extranonce);
    /// the mining algorithm is `pipeline::run`, not anything in this crate.
    pub fn mine_until_block(
        &self,
        cancel: Arc<AtomicBool>,
    ) -> Result<MinedBlockReceipt, SessionError>;
}

pub struct MinedBlockReceipt {
    pub hash:      bitcoin::BlockHash,
    pub height:    u64,
    pub witness:   prism_btc::MiningWitness, // §7.1
    pub trace:     prism::Trace,             // §6.4
    pub tx_count:  usize,
}
```

### 7.6 `prism_btc_node::bin::prism_mine` (CLI)

The CLI binary's surface is unchanged from the current state's
intent: `--rpc-url`, `--rpc-user`, `--rpc-pass`, `--network`,
`--payout`, `--blocks`, `--threads`, `--i-know-what-im-doing`. The
`--session` flag becomes redundant (the session is the only mode
under the reconciled architecture) and is removed. `--threads`
becomes a hint to foundation's `NonceFiberTraversal` parallelism
budget but is not load-bearing — the traversal is foundation-typed,
not user-orchestrated.

### 7.7 `prism_btc_wasm::mine_block`

```rust
// crates/prism-btc-wasm/src/api.rs

#[wasm_bindgen]
pub fn mine_block(js_header: &JsBlockHeader, nbits: u32)
    -> Result<JsMiningResult, JsValue>;
```

The body delegates to `prism_btc::mine` with the JS-encoded inputs.
The WASM surface does not expose `Trace`, `Grounded`, or any
foundation type directly; it exposes the digest bytes and the
triadic decomposition (which, under the reconciled architecture, are
queries against the foundation `Triad` minted alongside the witness;
see §5).

### 7.8 What is NOT in the public API

- No `Boundary` trait. Wire decode/encode is `prism-verify`'s
  `certify_from_trace` pathway.
- No `MorphismKind` markers (`DigestProjectionMap` etc.). The
  morphism is a `PrimitiveOp` composition, not a phantom marker.
- No `MiningRound`. The session is the entry point; `mine` is the
  per-invocation primitive.
- No `BlockCertificate<Sigma>`. The witness type is
  `MiningWitness = Grounded<ConstrainedTypeInput, MiningTag>`.
- No `Triadic`/`TriadicCoords` hand-rolled type with Hamming weight
  + nonzero-byte-mask. The triadic decomposition is the foundation
  `Triad` (datum, stratum, spectrum) with stratum being the 2-adic
  valuation and spectrum being the Walsh–Hadamard image (per the
  wiki Glossary). Existing tests that asserted Hamming-weight
  semantics are rewritten against the foundation Triad's actual
  semantics.

---

## 8. The repository layout

Three application crates in this repo, plus three external Prism
crates.

| Crate | Source | Role |
|---|---|---|
| `uor-foundation` | crates.io (`UOR-Foundation/UOR-Framework`, currently 0.3.1) | Substrate. Four UOR-domain sealed types, `PrimitiveOp` discriminants, the closed primitive operation set, the substitution-axis trait surface, `mint_*` primitives. |
| `prism` | crates.io (`UOR-Foundation/Prism`) | Runtime. Three Prism-mechanism sealed types, `pipeline::run`, the seal regime. |
| `prism-verify` | crates.io (`UOR-Foundation/Prism`) | Replay façade. `certify_from_trace`. |
| `prism-btc` | this repo, `crates/prism-btc/` | The application's pure domain layer. Declares all `ConstrainedTypeShape` impls (§4.7, §4.8), all `PrimitiveOp` compositions (§4.1–§4.6), the `HostBounds` selection (§3.2), the `Hasher` selection (§3.3), and the public entry point `mine` that constructs a CompileUnit and invokes `pipeline::run` once per (template, extranonce). No `sha2` dep, no `rayon` dep, no opaque crypto. |
| `prism-btc-node` | this repo, `crates/prism-btc-node/` | The bitcoind boundary. The only external-system glue: `getblocktemplate`, `submitblock`, `getbestblockhash` polling for tip-change. Holds the session's outer loop (§6.1). Also hosts the `prism-mine` CLI binary. Imports `bitcoincore-rpc` and `rust-bitcoin` as the RPC and serialisation surfaces. **No mining algorithm lives here**; the algorithm is `pipeline::run`. |
| `prism-btc-wasm` | this repo, `crates/prism-btc-wasm/` | The JavaScript surface. `wasm-bindgen` wrapper around `prism-btc::mine`. |

Three application crates, mirroring the framework's three-crate
substrate/runtime/replay split at the application scale (domain layer
/ boundary layer / wasm wrapper). The earlier draft's
`prism-btc-shapes` / `prism-btc-ops` / `prism-btc-pipeline`
sub-decomposition is rejected as over-engineering for this scale; the
domain layer is one crate.

### 8.1 Disappearances under reconciliation

These current artifacts are gone in the reconciled state:

| Removed artifact | Why |
|---|---|
| `crates/prism-btc-reduction/` (entire crate) | Its role (σ-projection + nonce iteration) is absorbed into `prism-btc`'s `PrimitiveOp` compositions and `pipeline::run`. |
| `crates/prism-btc-reduction/src/parallel.rs` | The rayon for-loop is replaced by `NonceFiberTraversal` (§4.6). |
| `crates/prism-btc-reduction/src/sha256d.rs` | The `sha2`-crate call is replaced by `Sha256dProjection` (§4.2). |
| `crates/prism-btc-reduction/src/serialize.rs` | The hand-rolled byte layout is replaced by `HeaderSerialization` (§4.3). |
| `crates/prism-btc-reduction/src/certificate.rs::BlockCertificate<Sigma>` | Replaced by `Grounded<ConstrainedTypeInput, MiningTag>` direct from `pipeline::run`. The phantom `Sigma` is removed because the σ-projection is now a concrete `PrimitiveOp` composition. |
| `crates/prism-btc-reduction/src/hasher.rs::Fnv1aHasher16` | Replaced by `Sha256dHasher` (§3.3). The Fnv1a substrate was a workaround when σ-projection and the foundation Hasher were conflated; under the §3.3 split they are now the same algorithm. |
| `Boundary` trait + `BoundaryDecodeError` (in `prism-btc/src/traits.rs`) | The wire-byte ↔ certificate isomorphism is no longer modelled as a separate trait; the trace IS the wire representation, and `prism-verify::certify_from_trace` IS the decode operation. |
| `MiningSession` (in `prism-btc-node/src/session.rs`) | The orchestrator that drives the rayon loop is gone; the session's outer loop (§6.1) — extranonce rolling, tip-change polling, hash-rate reporting — moves into a much smaller `prism-btc-node::session` module that does only what bitcoind requires of an external miner. The inner inference is `pipeline::run`. |
| `Cargo.toml` deps: `sha2`, `rayon` | gone. ADR-013 closure: SHA-256 is a `PrimitiveOp` composition; parallelism is a foundation-level concern within `NonceFiberTraversal`. |
| The `MorphismKind` / `DigestProjectionMap` / `BinaryGroundingMap` / `BinaryProjectionMap` / `ProjectionMapKind` / `GroundingMapKind` / `Total` / `Invertible` re-exports in the prelude | These markers were placeholders for what `PrimitiveOp` compositions now express concretely. With the operations declared as compositions, the markers are redundant. |

### 8.2 Imports outside the framework (closure under uor-foundation, ADR-013)

Per ADR-013 every prism-btc operation must be derivable from
`uor-foundation`'s closed primitive set. The architecture admits these
**non-foundation** dependencies, and only these:

| Dependency | Crate | Justification |
|---|---|---|
| Bitcoin RPC client | `bitcoincore-rpc` | The `bitcoind` boundary is an external system (ADR-004). `getblocktemplate`/`submitblock` are not Prism operations; they are calls into bitcoind's RPC. |
| Block / transaction serialisation | `bitcoin` (rust-bitcoin) | The block envelope, transaction format, script encoding, address parsing are Bitcoin protocol details outside Prism's scope. The σ-projection and merkle root are NOT delegated to rust-bitcoin; only the block-level container around a finished mining result is. |
| CLI argument parsing | `clap` | Outside Prism's scope. |
| Error reporting | `anyhow` | Outside Prism's scope. |
| Signal handling | `ctrlc` | Outside Prism's scope. |
| Serialisation glue for RPC | `serde`, `serde_json`, `hex` | Required by `bitcoincore-rpc`'s public surface; outside Prism's scope. |
| WebAssembly bindings | `wasm-bindgen`, `js-sys` | Outside Prism's scope (a JS interop concern). |

No cryptographic dependency (`sha2`, `blake3`, etc.). No
parallelism dependency (`rayon`, `tokio`, `crossbeam`). No
hand-rolled iteration utilities. The σ-projection, the nonce
traversal, the merkle derivation, the coinbase construction are all
foundation `PrimitiveOp` compositions.

---

## 9. The boundary properties (TC-01 .. TC-06) in prism-btc terms

| Constraint | How prism-btc realises it |
|---|---|
| **TC-01 zero-cost runtime** | Every `ConstrainedTypeShape` impl, `PrimitiveOp` composition, and substitution-axis selection is resolved by `rustc` at compile time. The executable contains no UORassembly enforcement code. The W32 traversal loop is a foundation-provided primitive; its body is monomorphised by the Rust compiler against `Sha256dHasher`. |
| **TC-02 sealing** | prism-btc constructs zero sealed types directly. Every `Datum`, `Triad`, `Derivation`, `FreeRank`, `Validated`, `Grounded`, `Certified` arrives via foundation's `mint_*` primitives or as a `pipeline::run` return value. The `BlockCertificate<Sigma>` wrapper is removed (§6.1); the Grounded is consumed directly. |
| **TC-03 path singularity** | `pipeline::run` is the only pathway to a `Grounded<...>` for prism-btc. Multiple invocations during extranonce rolling are permitted — TC-03 forbids alternative constructors, not iteration over the singular constructor. |
| **TC-04 UORassembly bilateral** | `prism-btc`'s ConstrainedTypeShape impls and PrimitiveOp compositions must satisfy `prism`'s trait bounds; checked by `rustc` on every build. Foundation amendments (ADR-013) are sequenced before prism-btc updates that depend on them. |
| **TC-05 replayability without deciders or hashing** | `prism-verify::certify_from_trace` walks the five-event trace structurally (§6.6). It does not invoke `Sha256dHasher`'s hashing method, does not invoke `Sha256dProjection`, does not invoke `TargetSubBundle`'s admission. It produces `Certified<GroundingCertificate>` from the trace's recorded fingerprint and structural relationships. |
| **TC-06 no author infrastructure** | `prism-mine` runs entirely on user hardware. The user supplies the bitcoind RPC. There is no prism-btc service, no callback to a content-addressed registry, no telemetry. After distribution, the binary is fully self-contained. |

---

## 10. Compile-time vs runtime separation

Per TC-01 + ADR-006 + Runtime View Scenario 3, the work split is
strict.

### At compile time (Scenario 3):

- `rustc` checks every `ConstrainedTypeShape` impl against `prism`'s
  `ConstrainedTypeShape` trait bounds.
- `rustc` checks every `PrimitiveOp` composition for closure under
  the foundation's primitive set (ADR-013).
- `rustc` monomorphises `pipeline::run::<ConstrainedTypeInput, M, H>`
  for `M = BinaryGroundingMap` and `H = Sha256dHasher` (§3.3).
- `rustc` validates `HostBounds::PrismBtcBounds`'s capacity constants
  against the wires they parameterise (ADR-018).
- The validated `CompileUnit`'s static structure (root term, witt
  level, target domains) is encoded into the executable as
  monomorphised constants. ConstrainedTypeShape `IRI`, `SITE_COUNT`,
  and `CONSTRAINTS` are static.
- The executable that `cargo build` produces contains no UORassembly
  validation code (TC-01).

### At runtime (Scenarios 1, 2, 4):

- `prism-btc-node` calls `bitcoind::getblocktemplate` (the only
  cross-boundary call).
- `prism-btc::mine` constructs the per-invocation CompileUnit
  (template-dependent), validates it (which is mostly a no-op since
  the structure is monomorphised; only template-specific sites need
  runtime validation), invokes `pipeline::run`.
- The pipeline executes `NonceFiberTraversal`'s deterministic
  traversal of the W32 ring. The runtime work is the σ-projection
  evaluation per fiber visit and the admission check per visit.
- `pipeline::run` returns `(Grounded<...>, Trace)`.
- `prism-btc-node` assembles the wire-format Block and submits.
- A user runs `prism-verify::certify_from_trace` on the trace; this
  runs structurally against the trace's events without invoking any
  decider or hasher (TC-05).

Compile time produces the executable; runtime produces the block.

---

## 11. Non-goals (explicit)

- **No SHA-256 inversion.** The strong cryptanalytic claim was ruled
  out earlier in the architecture discussion. The structural-inference
  framing applies to the *path*, not to the cost of evaluating the
  digest on each fiber point. Every fiber visit incurs one
  `Sha256dProjection` evaluation; the count of visits required to
  reach an admitting point is the same expectation as any other
  miner's at the same target.
- **No speedup vs. assembly-tuned SHA-256.** A traditional CPU miner
  using `sha2`'s assembly intrinsics will, per hash, outperform
  `Sha256Compression`-as-`PrimitiveOp`-composition. The performance
  gap is acceptable; the architectural value (closure under ADR-013,
  structurally-traced derivation, replayability without re-hashing)
  is the deliverable.
- **No foundation amendments asserted by this document.** §3 commits
  that the existing foundation primitive set (bit-rotation,
  integer-handling, lookup, content-comparison, depth-projection,
  observable-arithmetic) suffices for SHA-256d, the nonce traversal,
  the merkle derivation, and the coinbase construction. If
  implementation reveals an irreducible gap (e.g., `kernel::convergence`
  in foundation 0.3.1 does not provide the W32-ring fold-with-halt
  surface §4.6 requires), the corrective step is a foundation
  amendment per ADR-013, sequenced before prism-btc's amendment per
  ADR-015. This document forbids importing an opaque external crate
  in lieu of an amendment.
- **No mining-pool integration.** Stratum protocol, share submission,
  pool wallet management — all out of scope. prism-btc is solo-mining
  only; the bitcoind it talks to is the user's own.
- **No support for chains other than Bitcoin Core's accepted networks.**
  prism-btc supports `regtest`, `signet`, `testnet`, `testnet4`,
  `mainnet`. Other PoW chains (Litecoin, Bitcoin Cash, etc.) require
  different `Sigma` (§1, ruled out for this document) or different
  ConstrainedTypeShapes; they are scope for a different architecture
  document.

---

## 12. Reconciliation plan

The current repository state is non-conforming to this architecture
in the ways enumerated in §6.1. Reconciliation is one coherent change
set, not a sequence of phases:

1. **Replace the σ-projection.** Delete `prism-btc-reduction/src/sha256d.rs`
   and the `sha2` workspace dependency. Declare `Sha256Compression`
   and `Sha256dProjection` as `PrimitiveOp` compositions in
   `crates/prism-btc/src/ops/sha256.rs`.
2. **Replace the nonce iteration.** Delete `prism-btc-reduction/src/parallel.rs`
   and the `rayon` workspace dependency. Declare `NonceFiberTraversal`
   as a `kernel::convergence`-driven W32 fold in
   `crates/prism-btc/src/ops/traversal.rs`.
3. **Replace the wire serialisation.** Delete `prism-btc-reduction/src/serialize.rs`.
   Declare `HeaderSerialization` as a `depth-projection` composition
   in `crates/prism-btc/src/ops/header.rs`.
4. **Add merkle derivation and coinbase construction.** New
   compositions `MerkleRootDerivation`, `CoinbaseConstruction` in
   `crates/prism-btc/src/ops/{merkle,coinbase}.rs`. These replace the
   current rust-bitcoin merkle/coinbase logic in
   `prism-btc-node/src/session.rs`.
5. **Replace the certificate type.** Delete
   `prism-btc-reduction/src/certificate.rs` (the `BlockCertificate<Sigma>`
   wrapper). The result type of `prism-btc::mine` is
   `Grounded<ConstrainedTypeInput, MiningTag>` (alias
   `prism_btc::MiningWitness`), accompanied by a `Trace`.
6. **Replace the Hasher.** Delete `prism-btc-reduction/src/hasher.rs`
   (Fnv1aHasher16). Define `Sha256dHasher` in
   `crates/prism-btc/src/shapes/hasher.rs` as a foundation `Hasher`
   impl whose body is the `Sha256dProjection` PrimitiveOp composition.
7. **Replace the shape declarations.** The current
   `BlockHashShape`/`MerkleRootShape`/`TargetShape` are gone (§6.1
   already removed them in an earlier commit; they stay gone). New:
   `TemplatePrefixShape` (§4.7) and `TargetSubBundle` (§4.8) in
   `crates/prism-btc/src/shapes/{prefix,target_sub_bundle}.rs`.
8. **Define `PrismBtcBounds`.** A unit struct in
   `crates/prism-btc/src/shapes/bounds.rs` with the four
   `HostBounds` constants per §3.2.
9. **Declare the public entry point.** A single
   `crates/prism-btc/src/lib.rs::mine(prefix: [u8; 76], extranonce:
   u64, target_bits: u32) -> Result<MiningOutcome, PipelineFailure>`
   that constructs the per-invocation CompileUnit, calls
   `pipeline::run`, and returns the witness + trace.
10. **Dissolve `prism-btc-reduction`.** Remove the crate from the
    workspace; remove the dependency from `prism-btc` and
    `prism-btc-node`. The crate is gone.
11. **Rewire `prism-btc-node` to invoke `prism-btc::mine`.** Delete
    `prism-btc-node/src/session.rs::MiningSession`. Replace with a
    minimal `Session` that does only: tip polling, extranonce
    iteration, calling `prism-btc::mine`, calling `submitblock`. No
    rayon, no closures, no parallel orchestration logic.
12. **Update `prism-btc-wasm`.** Re-target its `mine_block` against
    `prism-btc::mine`'s new signature.
13. **Delete the `Boundary` trait, `BoundaryDecodeError`, the
    `MorphismKind` re-exports.** Update prelude and lib re-exports.
14. **Update the README.** Replace the existing "Mining as
    σ-convergence" framing with the §1 real-time-inference framing.
    Remove the testnet4 demo paragraph (its framing was
    rejected); replace with a description of what the architecture
    achieves.
15. **Delete or update the existing tests.** The parallel/rayon tests
    are gone. Add new tests: a regtest end-to-end through
    `prism-btc::mine` + `submitblock`; a trace-replay test that
    `prism-verify::certify_from_trace` produces a `Certified` from
    the regtest run's emitted trace.

The reconciliation is non-conforming if any of the 15 points above is
incomplete: prism-btc is either fully in this state, or it is
non-conforming. There is no partial conformance.

---

## 13. Responsibility split: foundation substrate vs prism implementor

The wiki distinguishes two roles, and prism-btc occupies the second:

- **`uor-foundation` is the substrate.** It provides sealed types
  (`Datum`, `Triad`, `Derivation`, `FreeRank`, `Validated`,
  `Grounded`, `Certified`), the closed `PrimitiveOp` vocabulary, the
  `Term` AST variants, the substitution-axis traits (`Hasher`,
  `HostBounds`, `HostTypes`, `GroundingMapKind`), the mint primitives
  (`mint_datum`, `mint_triad`, `mint_derivation`, `mint_freerank`),
  the `Trace`/`TraceEvent` structure, and the
  `enforcement::replay::certify_from_trace` function. It does **not**
  ship a runtime that evaluates `Term`s, a fold-with-halt-on-predicate
  primitive, a SHA-256 implementation, or any other "operations
  helper". `Term` is compile-time metadata; the substrate's
  `pipeline::run` is a deterministic certification path
  (preflights → SAT → hash → mint), not an evaluator.
- **`prism-btc` is the prism implementor for the Bitcoin use case.**
  It is responsible for everything the substrate delegates to the
  prism layer: the actual evaluation of the typed structure for
  Bitcoin mining, including the σ-projection runtime (pure-Rust
  SHA-256d), the W32 fiber traversal, the merkle-tree derivation,
  the coinbase scriptSig assembly, the header serialisation, the
  `Grounding` impls, the `ConstrainedTypeShape` impls, the
  `Hasher` and `HostBounds` impls, and the `mine` entry point that
  ties them together.

The architecture above (§§1–11) is therefore a specification of
prism-btc's runtime, expressed in foundation vocabulary, not a list
of demands on foundation. Foundation does not need to be amended for
prism-btc to reach the defined state; prism-btc just needs to be
written.

### 13.1 What foundation 0.3.1 supplies, used as-is

| Surface | Foundation path | prism-btc usage |
|---|---|---|
| Sealed `Datum`, `Triad`, `Derivation`, `FreeRank` | `enforcement::{Datum, Triad, Derivation, FreeRank}` | Returned via mint primitives during admission. |
| Sealed `Validated`, `Grounded`, `Certified` | `enforcement::{Validated, Grounded}` + `enforcement::replay::certify_from_trace` returning `Certified` | Returned by `pipeline::run` (Grounded) or replay (Certified). prism-btc never constructs them directly. |
| `mint_*` primitives | `enforcement` module | Foundation's pipeline / replay machinery calls these; prism-btc does not. |
| `CompileUnitBuilder` + `Validated<CompileUnit, FinalPhase>` | `enforcement::{CompileUnit, CompileUnitBuilder}` | Used to declare the BlockHash shape unit; const-validated via `validate_compile_unit_const`. |
| `pipeline::run` / `pipeline::run_const` | `pipeline::{run, run_const}` | Used to mint the shape `Grounded` after prism-btc's traversal admits. The pipeline does not drive the traversal; it certifies the shape declaration. |
| Closed `PrimitiveOp` set (10 generators) | `enums::PrimitiveOp` | Used as `Term::Application` operators in prism-btc's compile-time structural declarations. The 10 generators (`Neg, Bnot, Succ, Pred, Add, Sub, Mul, Xor, And, Or`) are sufficient to *type-encode* prism-btc's operations as compositions. They are not a runtime; the runtime is prism-btc's. |
| `Term` (9 variants) | `enforcement::Term` | Used in `CompileUnitBuilder::root_term` to declare prism-btc's structural shape. Compile-time metadata. |
| `ConstrainedTypeShape` trait + `ConstraintRef` | `pipeline::{ConstrainedTypeShape, ConstraintRef}` | Implemented by `TemplatePrefixShape` and `TargetSubBundle`. |
| `HostBounds` trait | `enforcement::HostBounds` | Implemented by `PrismBtcBounds`. |
| `Hasher` trait | `enforcement::Hasher` | Implemented by `Sha256dHasher` with arbitrary Rust code (the trait permits this; ADR-010 requires only determinism, fixed width, idempotence, distinct identifier). |
| `Trace` and `TraceEvent` | `enforcement::{Trace, TraceEvent}` | Emitted by foundation's pipeline, consumed by `prism-verify` replay. |
| `enforcement::replay::certify_from_trace` | `enforcement::replay::certify_from_trace` | Used by users to mint `Certified` from a `Trace` without invoking prism-btc's deciders or `Sha256dHasher`'s body (TC-05). |

### 13.2 What prism-btc supplies as the prism implementor

| Surface | prism-btc path | Role |
|---|---|---|
| `Sha256dHasher` | `prism_btc::shapes::hasher::Sha256dHasher` | Foundation `Hasher` impl. Body is pure-Rust SHA-256d. |
| `PrismBtcBounds` | `prism_btc::shapes::bounds::PrismBtcBounds` | `HostBounds` impl with the four constants (§3.2). |
| `TemplatePrefixShape`, `TargetSubBundle` | `prism_btc::shapes::{prefix, target_sub_bundle}` | `ConstrainedTypeShape` impls (§4.7, §4.8). |
| `Sha256Compression`, `Sha256dProjection`, `HeaderSerialization`, `MerkleRootDerivation`, `CoinbaseConstruction` | `prism_btc::ops::*` | Each operation has two surfaces: a compile-time `Term::Application` declaration (for type-level routing) and a runtime function (the actual computation). |
| `NonceFiberTraversal` | `prism_btc::ops::traversal` | The runtime W32 fiber walk. prism-btc's responsibility; not a foundation primitive. |
| `mine()` | `prism_btc::lib::mine` | The public entry point. Orchestrates the runtime, calls `pipeline::run` to mint the shape `Grounded` after admission, returns `MiningOutcome`. |

The substrate-vs-implementor split above is the architecture's
load-bearing distinction. Where I previously wrote that 0.3.1 was
"missing" things — `kernel::convergence`-as-search, `Term`-driven
runtime evaluation, SHA-256 as a `PrimitiveOp` composition — those
were category errors. Foundation does not ship those because it
delegates them to the prism implementor. Reconciling prism-btc to
the architecture is therefore a matter of prism-btc writing what is
its responsibility to write, not waiting for foundation amendments.

---

## 14. Wiki cross-reference index

Every architectural commitment in this document traces back to a
specific page or clause of the [UOR-Framework wiki](https://github.com/UOR-Foundation/UOR-Framework/wiki).
This section is the round-trip index: every wiki entry that prism-btc
relies on, with the §-refs of this document that depend on it.

### 14.1 Boundary properties (TC-01..TC-06)

Source: [02 Architecture Constraints](https://github.com/UOR-Foundation/UOR-Framework/wiki/02-Architecture-Constraints).

| Wiki entry | prism-btc commitment | §-refs |
|---|---|---|
| TC-01 zero-cost runtime | No UORassembly enforcement code in `prism-mine`. All work compile-time-resolved. | §1, §9, §10 |
| TC-02 sealing of certified types | prism-btc constructs zero sealed types directly; uses `mint_*` and `pipeline::run` returns. | §6.3, §9 |
| TC-03 path singularity | One `pipeline::run` per (template, extranonce). No alternative constructor. | §1, §6.3, §6.5, §9 |
| TC-04 UORassembly bilateral | `prism-btc`'s impls must satisfy `prism`'s bounds; checked by `rustc`. | §9 |
| TC-05 replayability without deciders or hashing | `prism-verify::certify_from_trace` walks 5 events; no Hasher invocation; no decider invocation. | §6.6, §9 |
| TC-06 no author infrastructure | `prism-mine` runs entirely on user hardware. | §9 |

### 14.2 Architecture decisions (ADR-001..ADR-018)

Source: [09 Architecture Decisions](https://github.com/UOR-Foundation/UOR-Framework/wiki/09-Architecture-Decisions).

| ADR | prism-btc impact | §-refs |
|---|---|---|
| ADR-001 Prism system definition | Wiki is normative. This doc is reconciled to the wiki. | preamble |
| ADR-002 Boundary properties normative | All six TC-* enforced. | §9 |
| ADR-003 Verification local-by-construction | `prism-verify` runs on user hardware; no service. | §6.6 |
| ADR-004 Distribution channel external to Prism | bitcoind RPC and submitblock are external; outside Prism's scope. | §6.1, §8.2 |
| ADR-005 Three-crate decomposition | foundation/prism/prism-verify are separate external crates; prism-btc is independent. | §8 |
| ADR-006 UORassembly bilateral compile-time | Compile-time-only validation; no runtime UORassembly. | §10 |
| ADR-007 Substitution-axes allocation | `HostTypes` / `HostBounds` / `Hasher` selected at the application crate. | §3 |
| ADR-008 Trace wire format normative | prism-btc emits trace bytes per the wiki's wire format. | §6.4 |
| ADR-009 Certificate format normative | Certificates carry `(CertificateKind, ContentAddress)` per the wiki. | §6.6 |
| ADR-010 Hasher contract | `Sha256dHasher` satisfies determinism + fixed width + idempotence + distinct identifier. | §3.3, §7.2 |
| ADR-011 Sealing via Rust visibility | All seven sealed types use `pub(crate)` constructors; prism-btc never bypasses. | §9 |
| ADR-012 Pipeline lives in prism, not foundation | prism-btc imports `prism::pipeline::run`; foundation provides primitives. | §6.2 |
| ADR-013 Prism closed under uor-foundation | All prism-btc operations are `PrimitiveOp` compositions. No `sha2`, no `rayon`. | §1, §4, §8.2, §11 |
| ADR-014 Operation declaration vs. shipment | prism-btc declares its six operations as `PrimitiveOp` compositions. | §4 |
| ADR-015 Repository split strategy | Foundation amendments sequenced before prism-btc updates. | §11, §13 |
| ADR-016 Cross-crate seal mechanism via mint primitives | prism-btc never calls `mint_*` directly; `pipeline::run` does. | §9 |
| ADR-017 Canonical UOR-address surface | prism-btc's IRIs are `https://prism.btc/...` for stable schema. | §4.7, §4.8, §7.2 |
| ADR-018 HostBounds capacity completeness | All capacity values flow through `PrismBtcBounds`. | §3.2 |

### 14.3 Building Block View

Source: [05 Building Block View](https://github.com/UOR-Foundation/UOR-Framework/wiki/05-Building-Block-View).

| Wiki block | prism-btc dependency | §-refs |
|---|---|---|
| `enforcement::resolver` (Hasher contract) | `Sha256dHasher` impls the contract. | §3.3, §7.2 |
| `enforcement::calibrations` | implicit via `PrismBtcBounds`. | §3.2 |
| `enforcement::transcendentals` | foundation-fixed wire-format constants used in trace serialisation. | §6.4 |
| `enforcement::combinators` | composing UOR-domain values inside the pipeline. | §5 |
| `mint primitives` (`mint_datum`, `mint_triad`, `mint_derivation`, `mint_freerank`) | invoked by `pipeline::run` at admission stages; not by prism-btc. | §6.2, §9 |
| `bridge::ConstrainedTypeShape` trait | `TemplatePrefixShape`, `TargetSubBundle` impl this. | §4.7, §4.8 |
| `bridge::Grounding` trait | prism-btc's Grounding impls. | §7.4 |
| `bridge::trace::{Trace, TraceEvent}` | prism-btc's trace structure. | §6.4 |
| `bridge::cert::{Certificate, ContentFingerprint, ContentAddress}` | the certificate the pipeline emits and `prism-verify` certifies. | §6.6 |
| `kernel::HostTypes`, `kernel::HostBounds` traits | `DefaultHostTypes` and `PrismBtcBounds` impl. | §3.1, §3.2 |
| `kernel::convergence` | `NonceFiberTraversal` is a convergence-driven W32 fold. | §4.6 |
| `kernel::primitives` (closed primitive set) | every prism-btc operation is closed under this set. | §4 |
| `prism::pipeline::run` | the single entry point to a `Grounded<T>`. | §1, §5, §6.2 |
| `prism::seal regime` | `Validated`, `Grounded`, `Certified` are sealed; prism-btc consumes via mint primitives. | §6.3 |
| `prism::replay::certify_from_trace` | trace replay yielding `Certified<GroundingCertificate>`. | §6.6 |

### 14.4 Runtime View

Source: [06 Runtime View](https://github.com/UOR-Foundation/UOR-Framework/wiki/06-Runtime-View).

| Wiki scenario | prism-btc usage | §-refs |
|---|---|---|
| Scenario 1 Principal data path execution | One `pipeline::run` per (template, extranonce); produces Grounded + Trace simultaneously. | §6.2 |
| Scenario 2 Trace-replay verification | `prism-verify::certify_from_trace` walks the 5-event trace structurally. | §6.6 |
| Scenario 3 Compile-time UORassembly enforcement | `cargo build` checks all impls + bounds; emits `prism-mine`. | §10 |
| Scenario 4 Distribute and run | `prism-mine` distributed externally; user runs on own hardware with own bitcoind. | §1, §9 |

### 14.5 Concepts and Glossary

Source: [08 Concepts](https://github.com/UOR-Foundation/UOR-Framework/wiki/08-Concepts) and [12 Glossary](https://github.com/UOR-Foundation/UOR-Framework/wiki/12-Glossary).

| Wiki term | prism-btc usage | §-refs |
|---|---|---|
| Datum | `TemplatePrefixDatum`; the 32-byte digest minted as Datum at admission. | §5 |
| Triad (datum, stratum=2-adic-valuation, spectrum=Walsh-Hadamard image) | minted alongside the digest; available via `MiningWitness` accessors. | §7.7, §7.8 |
| Derivation | the foundation Derivation recording the `Sha256dProjection ∘ HeaderSerialization` composition. | §5 |
| FreeRank | starts at 1 (nonce free); collapses to 0 on admission. | §5 |
| Validated, Grounded, Certified | `Validated<CompileUnit, FinalPhase>`, `Grounded<ConstrainedTypeInput, MiningTag>`, `Certified<GroundingCertificate>`. | §5, §6.6, §7.1 |
| ConstrainedTypeShape | `TemplatePrefixShape`, `TargetSubBundle`. | §4.7, §4.8 |
| Grounding | `TemplatePrefixGrounding`, `TargetSubBundleGrounding`. | §7.4 |
| Hasher | `Sha256dHasher`. | §3.3, §7.2 |
| HostTypes, HostBounds | `DefaultHostTypes`, `PrismBtcBounds`. | §3.1, §3.2 |
| Trace | five-event sequence per `pipeline::run`. | §6.4 |
| Resolution | the W32 fiber's free coordinate is resolved by `NonceFiberTraversal`. | §4.6, §5 |

### 14.6 Context and Scope

Source: [03 Context and Scope](https://github.com/UOR-Foundation/UOR-Framework/wiki/03-Context-and-Scope).

| Wiki boundary | prism-btc placement | §-refs |
|---|---|---|
| Application Author input | `prism-btc::mine`'s arguments (prefix, extranonce, target). | §7.1 |
| Application Author output | `MiningOutcome` (witness + trace). | §7.1 |
| Verification (Author → User) | trace + hasher_identifier passed out-of-band. | §6.6 |
| Verification output | `Certified<GroundingCertificate>` or `ReplayError`. | §6.6 |
| Out-of-scope: distribution channels | bitcoind RPC, `submitblock`, JS distribution are outside Prism. | §6.1, §8.2 |

### 14.7 Conceptual Model

Source: [Conceptual Model](https://github.com/UOR-Foundation/UOR-Framework/wiki/Conceptual-Model).

prism-btc's §2 follows the wiki's OPM convention. The wiki's
`Application`, `Application Author`, `Application User`, `Rust
Toolchain`, `Prism` entities are inherited (§2.1). prism-btc's
specialisations are the Bitcoin-domain entities and processes (§2.2,
§2.4). All OPL declarations (§2.5) reference back to either a wiki
normative source or a §-ref of this document.

### 14.8 Lifecycle

Source: [Lifecycle Technical Processes](https://github.com/UOR-Foundation/UOR-Framework/wiki/Lifecycle-Technical-Processes).

| Wiki process | prism-btc realisation |
|---|---|
| System Requirements Definition | TC-01..TC-06 + ADR-007's three substitution axes are inputs to this document. |
| System Architecture Definition | this document is prism-btc's architecture definition. |
| Design Definition | §4 (operations) + §7 (API surface) constitute the design. |
| Integration | §10 commits to compile-time-only integration via UORassembly bilateral enforcement. |
| Implementation | §12 reconciliation enumerates the implementation-level deltas required. |
| Verification (in lifecycle sense) | §6.6 (replay) + §9 (boundary properties) + the regtest end-to-end test (§12 step 15). |

---

> **End of normative content.** Subsequent edits to this document
> change prism-btc's defined state. Implementation reconciliation
> follows §12.
