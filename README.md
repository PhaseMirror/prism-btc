# prism-btc

Bitcoin proof-of-work as **real-time structural inference** — a Prism
application of the [UOR Foundation](https://github.com/UOR-Foundation/UOR-Framework).
prism-btc is the prism implementor for the Bitcoin use case: it
provides the runtime that walks the foundation-typed structure
declared via `ConstrainedTypeShape` + `Term::Application` compositions,
finds the admitting fiber point in the W32 nonce ring, and produces a
foundation-sealed `Grounded<ConstrainedTypeInput, MiningTag>` whose
wire bytes are accepted byte-for-byte by Bitcoin Core.

> **Defined architecture:** see [ARCHITECTURE.md](ARCHITECTURE.md).
> The repository state is reconciled to that document; ARCHITECTURE.md
> is normative.
>
> **Frame of reference:** the
> [UOR-Framework wiki](https://github.com/UOR-Foundation/UOR-Framework/wiki),
> which is itself the normative specification of Prism.

## The claim

Traditional Bitcoin miners blackbox-import SHA-256, iterate a `u32`,
compare bytes to a threshold, and emit a block. Their process is
invisible to the type system and untraced.

prism-btc is the converse: **bit-identical output to a traditional
miner, derived through Prism's vocabulary alone.** No `sha2` import,
no `rayon`, no opaque crate imports. SHA-256d is a pure-Rust
foundation `Hasher` impl ([`Sha256dHasher`](crates/prism-btc/src/shapes/hasher.rs)).
The W32 nonce traversal is prism-btc's own runtime
([`NonceFiberTraversal`](crates/prism-btc/src/ops/traversal.rs)),
expressed as a deterministic walk over the foundation-typed ring.

The architecture's **categorical routing through types** is realised
on foundation 0.3.2 by a `PrismModel<H, B, A>` declaration
([`BitcoinMiningModel`](crates/prism-btc/src/model.rs)): `Input =
MiningInput` (80 W8 sites — the canonical wire-format header), `Output =
ConstrainedTypeInput` (foundation's identity), `Hasher = Sha256dHasher`.
prism-btc's runtime walks the W32 fiber to the admitting input;
foundation's `pipeline::run_route` mints the
`Grounded<ConstrainedTypeInput, MiningTag>` shape attestation over it.

## Workspace

| Crate | Role |
|---|---|
| [`prism-btc`](crates/prism-btc/) | The prism implementor. Public `mine()` entry point. Pure-Rust SHA-256/SHA-256d. W32 fiber traversal (sequential + parallel). Domain types, `ConstrainedTypeShape` impls, `HostBounds` impl, `Hasher` impl. **No external crypto dep.** |
| [`prism-btc-node`](crates/prism-btc-node/) | Bitcoin Core RPC boundary. `getblocktemplate` → `prism_btc::mine` → `submitblock`. Two layers: `PrismMiner` (single-shot) and `MiningSession` (extranonce + tip-watcher + parallel). `prism-mine` CLI binary. |
| [`prism-btc-wasm`](crates/prism-btc-wasm/) | `wasm-bindgen` JS surface around `prism_btc::mine`. |
| [`prism-btc-lean/`](prism-btc-lean/) | Lean 4 formal proofs: ring identity (W8/W32), triadic coords, FreeRank protocol, shape constraint monotonicity, σ-convergence termination. |
| [`docs/adr/`](docs/adr/) | Architecture Decision Records. ADR-024 specifies Multiplicity Theory integration via Profinite Fiber Stratification (PFS) and MSMP. |

Three application crates plus three external Prism crates
(`uor-foundation`, `prism`, `prism-verify`).

## The substrate-vs-implementor split

prism-btc reconciliation makes ARCHITECTURE.md §13's split explicit:

**`uor-foundation` provides** the substrate — sealed types (`Datum`,
`Triad`, `Derivation`, `FreeRank`, `Validated`, `Grounded`,
`Certified`), the `PrimitiveOp` enum (10 dihedral generators), `Term`
variants, the `Hasher`/`HostBounds`/`HostTypes`/`GroundingMapKind`
substitution-axis traits, the `mint_*` primitives, the `Trace` and
`TraceEvent` structures, and `enforcement::replay::certify_from_trace`.
It does **not** ship a runtime that evaluates `Term`s, nor a
fold-with-halt-on-predicate primitive, nor SHA-256, nor any "operations
helper". Foundation is substrate, not runtime.

**prism-btc provides** what the substrate delegates to the prism
implementor: the [Sha256dHasher](crates/prism-btc/src/shapes/hasher.rs),
the [W32 fiber traversal](crates/prism-btc/src/ops/traversal.rs), the
[merkle-tree derivation](crates/prism-btc/src/ops/merkle.rs),
the [coinbase + header serialisation](crates/prism-btc/src/ops/header.rs),
the [σ-projection runtime](crates/prism-btc/src/ops/sigma.rs),
the [`ConstrainedTypeShape` impls](crates/prism-btc/src/shapes/),
the [`PrismBtcBounds`](crates/prism-btc/src/shapes/bounds.rs)
substitution-axis selection, and the public
[`mine()`](crates/prism-btc/src/pipeline.rs) entry point. Foundation
provides the type vocabulary; prism-btc provides the Bitcoin
realisation.

## Public API

```rust
use prism_btc::{
    mine, mine_parallel, block_hash_grounded,
    BitcoinMiningModel, MiningInput,
    BlockHeader, MerkleRoot, Target, Bits, Timestamp, Version,
    NeverCancel, MiningOutcome, MiningFailure, MiningWitness,
    Sha256dHasher, PrismBtcBounds,
};

let header = BlockHeader {
    version: Version(1),
    prev_hash: [0u8; 32],
    merkle_root: MerkleRoot::from_bytes([
        0x3b, 0xa3, 0xed, 0xfd, 0x7a, 0x7b, 0x12, 0xb2,
        0x7a, 0xc7, 0x2c, 0x3e, 0x67, 0x76, 0x8f, 0x61,
        0x7f, 0xc8, 0x1b, 0xc3, 0x88, 0x8a, 0x51, 0x32,
        0x3a, 0x9f, 0xb8, 0xaa, 0x4b, 0x1e, 0x5e, 0x4a,
    ]),
    timestamp: Timestamp(1231006505),
    bits: Bits(0x1d00ffff),
};

// Real-time structural inference: mine() walks the W32 fiber under the
// declared σ-projection composition, halts at the first admitting
// nonce, mints the foundation-sealed Grounded shape attestation.
let outcome = mine(&header, Target::new(0x207fffff), &NeverCancel)
    .expect("easy target must admit");

assert_eq!(outcome.witness.witt_level_bits(), 32); // W32 ceiling
assert!(Target::new(0x207fffff).is_satisfied_by_bytes(&outcome.digest));
```

For long-running real-network mining, use the parallel variant:

```rust
let outcome = mine_parallel(&header, target, /* threads */ 8, &NeverCancel)?;
```

The `MiningOutcome` carries the foundation-sealed `MiningWitness =
Grounded<ConstrainedTypeInput, MiningTag>`, the admitting nonce, the
digest, and the digest's `TriadicCoords` (datum + 2-adic stratum +
spectrum parity).

## Real-network mining (`prism-btc-node`)

The `prism-mine` CLI drives `prism_btc::mine` against any running
bitcoind. Two modes:

**Single-shot** (default): one template, one `prism_btc::mine` call,
one submit. For regtest where the W32 traversal admits trivially.

```bash
just regtest-demo   # mines 10 blocks against a local bitcoind
```

**Session** (`--session`): long-running with extranonce rolling,
tip-staleness watcher, and `mine_parallel` per (template, extranonce).

```bash
prism-mine \
  --rpc-url http://127.0.0.1:8332 \
  --rpc-user RPCUSER --rpc-pass RPCPASS \
  --network testnet4 \
  --payout TB1Q... \
  --session \
  --threads 8 \
  --blocks 1
```

**Safety airlocks:**
- **Chain-mismatch guard**: refuses to mine if `getblockchaininfo.chain` disagrees with the requested `--network`.
- **Mainnet opt-in**: `--network mainnet` requires `--i-know-what-im-doing`.

## Foundation `Hasher` and `HostBounds`

`Sha256dHasher` is the foundation `Hasher` substitution-axis selection
for prism-btc. Per ADR-010 it is deterministic, fixed-width (32 bytes),
and idempotent. The body is pure-Rust SHA-256 (FIPS-180-4) applied
twice; no external crypto dependency.

`PrismBtcBounds` is the foundation `HostBounds` selection:

| Constant | Value |
|---|---|
| `FINGERPRINT_MIN_BYTES` | `32` |
| `FINGERPRINT_MAX_BYTES` | `32` |
| `TRACE_MAX_EVENTS` | `64` |
| `WITT_LEVEL_MAX_BITS` | `32` |

The `TRACE_MAX_EVENTS = 64` ceiling is the architectural commitment
that the trace records one event per stage transition, not one per
W32 fiber visit.

## Quick start

```bash
cargo install just

just build      # cargo build --workspace
just test       # cargo test --workspace
just lint       # cargo clippy --workspace -- -D warnings
just fmt-check  # cargo fmt --check

# Formal proofs (requires elan / Lean 4)
just verify     # lake update && lake build

# WebAssembly
just build-wasm

# End-to-end regtest demo
just regtest-demo
```

## WebAssembly

```bash
just build-wasm   # wasm-pack build → pkg/prism-btc-wasm/
```

```js
import init, { JsBlockHeader, mine_block } from './prism_btc_wasm.js';
await init();

const header = new JsBlockHeader(version, prevHashBytes, merkleBytes, timestamp, bits);
const result = mine_block(header, 0x1d00ffff);
console.log(result.stratum, result.spectrum, result.hash());
```

The wasm `mine_block` calls `prism_btc::mine` directly. The 2-adic
stratum and the Walsh-Hadamard parity spectrum are returned alongside
the digest.

## License

Apache-2.0 — see [LICENSE](LICENSE).
