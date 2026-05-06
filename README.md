# prism-btc

Bitcoin mining as **σ-convergence** over the nonce fiber of the 32×W8 Datum space.

The block header (80 bytes) is the source space. The σ-projection (SHA256d)
maps each (header, nonce) pair to a candidate 32-byte Datum. The target shape
constraint defines a sub-bundle of the Datum space. Mining is a search for a
nonce whose image under σ lands in that sub-bundle; the UOR shape certification
runs exactly once per round via `pipeline::run_const` (v0.3.1), producing an
un-fabricatable `Grounded<ConstrainedTypeInput, BlockHashTag>` that is cloned
into every winning candidate's certificate.

## Type-level morphism kinds

prism-btc carries two distinct foundation `MorphismKind` markers at the type level:

| Morphism | Kind | Properties |
|---|---|---|
| **σ-projection** (header‖nonce → 32-byte digest) | `DigestProjectionMap` | `Total`. Not `Invertible`, not `PreservesStructure`, not `PreservesMetric`. |
| **Wire round-trip** (`BlockCertificate` ↔ 80 bytes) | `BinaryGroundingMap` ↔ `BinaryProjectionMap` | `Total + Invertible` in both directions — a zero-cost isomorphism. |

The σ-projection kind is the phantom parameter on
`BlockCertificate<Sigma: ProjectionMapKind + Total>`. The wire-isomorphism kinds
are the `Boundary` trait's `Ingest`/`Emit` associated types. Together they
make the only two morphisms in the design type-level explicit: nothing the
mining loop does can confuse them, and the foundation's sealed `MorphismKind`
hierarchy means downstream cannot smuggle in a different `Sigma`.

> **Note on terminology.** SHA256d is the **σ-projection** (ingestion hash),
> NOT the UOR ψ-map. UOR Foundation reserves ψ for the categorical functor
> chain ψ_1..ψ_9 (Constraints → Nerve → Chain → Homology → … → KInvariants),
> which a deliberately non-shape-preserving avalanche function does not satisfy.

## Workspace

| Crate | Role |
|---|---|
| [`prism-btc-types`](crates/prism-btc-types/) | Domain types (`BlockHash`, `Target`, `TriadicCoords`, `BlockHeader`, `Version`/`Timestamp`/`Bits`) and the `BlockHashTag` phantom |
| [`prism-btc-reduction`](crates/prism-btc-reduction/) | σ-convergence loop (`run_convergence`), wire serialization, `block_hash_shape_certificate`, `BlockCertificate<Sigma>`, `Fnv1aHasher16` |
| [`prism-btc`](crates/prism-btc/) | Public API: `MiningRound`, `BlockCertificate`, `Boundary`, `genesis()` |
| [`prism-btc-wasm`](crates/prism-btc-wasm/) | `wasm-bindgen` wrapper: `JsBlockHeader`, `mine_block()` (distributed via wasm-pack, not crates.io) |
| [`prism-btc-node`](crates/prism-btc-node/) | Bitcoin Core RPC integration. Two layers: `PrismMiner` (single-shot, regtest-friendly) and `MiningSession` (long-running: parallel σ-convergence over the W32 nonce ring via rayon, coinbase extranonce rolling, tip-staleness watcher with mid-search cancellation, hash-rate reporter, mainnet airlock). The `prism-mine` CLI drives both modes and connects to any network bitcoind supports — regtest, signet, testnet3, testnet4, mainnet |
| [`prism-btc-lean/`](prism-btc-lean/) | Lean 4 formal proofs: ring identity (W8/W32), triadic coords, FreeRank protocol, shape constraint monotonicity, σ-convergence termination |

## Quick start

```bash
# Prerequisites: Rust stable + just
cargo install just

just build      # cargo build --workspace
just test       # cargo test --workspace --exclude prism-btc-wasm
just lint       # cargo clippy -D warnings
just fmt-check  # cargo fmt --check

# Formal proofs (requires elan / Lean 4)
just verify     # lake update && lake build

# WebAssembly (requires wasm-pack)
just build-wasm
```

## Public API

The entire client-facing surface is re-exported from `prism_btc::prelude`:

```rust
use prism_btc::prelude::*;
```

This brings into scope:
- **Mining context**: `MiningRound` — wraps a `(BlockHeader, Target)` pair
- **Certificate output**: `BlockCertificate<Sigma>` — sealed `Grounded<ConstrainedTypeInput, BlockHashTag>` + `TriadicCoords`, phantom-typed by σ-projection morphism kind
- **Domain types**: `BlockHash`, `BlockHeader`, `MerkleRoot`, `Target`, `TriadicCoords`, `Version`, `Timestamp`, `Bits`
- **Type tag & alias**: `BlockHashTag`, `BlockHashGrounded`
- **Boundary trait**: `Boundary` (decode/encode), `BoundaryDecodeError`
- **Failure type**: `ConvergenceFailure::FiberExhausted` (the only way σ-convergence can fail — the shape pipeline is infallible)
- **UOR enforcement**: `Grounded`, `Validated`, `ConstrainedTypeInput`
- **Morphism kinds**: `DigestProjectionMap`, `BinaryGroundingMap`, `BinaryProjectionMap`, plus the bound traits `ProjectionMapKind`, `GroundingMapKind`, `Total`, `Invertible`
- **Genesis**: `genesis()` — formally-grounded block-hash shape certificate

There is no `u32` nonce accessor in the public surface; the nonce lives only inside `BlockCertificate::encode_wire()` bytes, because the Bitcoin protocol places it there.

## Mining

```rust
use prism_btc::prelude::*;

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

// Run the σ-convergence loop. There is no `u32` nonce accessor on the cert.
// `Sigma = DigestProjectionMap` — the foundation kind for SHA256d.
let cert: BlockCertificate<DigestProjectionMap> =
    MiningRound::new(header, Target::new(0x207fffff))
        .converge()
        .expect("easy target must converge");

// Certified shape — sealed Grounded<ConstrainedTypeInput, BlockHashTag>,
// cannot be fabricated. The BlockHashGrounded alias keeps the type readable.
let grounded: &BlockHashGrounded = cert.grounded();
assert_eq!(grounded.witt_level_bits(), 32); // W32 ceiling from CompileUnit

// The 32-byte block hash, surfaced directly.
let digest: &[u8; 32] = cert.digest();

// PRISM triadic coordinates (datum, stratum, spectrum).
let coords: &TriadicCoords = cert.coords();
println!("digest: {:?}", digest);
println!("stratum: {}", coords.stratum);
println!("spectrum: {}", coords.spectrum);
```

## Genesis hash as a grounded constant

```rust
use prism_btc::genesis;

// v0.3.1 path: the CompileUnit is validated at compile time via
// validate_compile_unit_const; pipeline::run_const executes at call time and
// the result is tagged with BlockHashTag. Panics at compile time if the
// CompileUnit is malformed; at runtime if the pipeline rejects it.
let grounded = genesis();
assert_ne!(grounded.unit_address().as_u128(), 0);
assert_eq!(grounded.witt_level_bits(), 32); // W32 ceiling
```

## Boundary: decoding wire-format headers

The `Boundary` trait crosses the raw-bytes / certified-types boundary. `decode`
always re-runs the full pipeline — it cannot be bypassed.

```rust
use prism_btc::prelude::*;

// Decode an 80-byte wire header. Returns Err if length ≠ 80 or if the
// hash fails the run_pipeline certification. The decode/encode pair forms a
// `BinaryGroundingMap` ↔ `BinaryProjectionMap` isomorphism on wire bytes.
let cert = BlockCertificate::decode(&wire_bytes)?;
let bytes: Vec<u8> = cert.encode();
```

## σ-convergence loop

```text
grounded = block_hash_shape_certificate()                 // v0.3.1 pipeline::run_const, once
for nonce in 0..=u32::MAX:
    raw       = serialize_header(header, nonce)           // 80-byte wire format
    candidate = sha256d(raw)                               // σ-projection (NOT ψ-map)
    if candidate > target_bytes: continue                  // fast pre-filter
    return BlockCertificate { grounded: grounded.clone(), coords, ... }
return Err(FiberExhausted)
```

The UOR pipeline certifies the *shape declaration* (the `CompileUnit`), not
individual hash values, so it runs exactly once per `MiningRound::converge()`
call — before the nonce loop. The CompileUnit itself is `const`-validated at
compile time via `validate_compile_unit_const`; `pipeline::run_const::<_,
BinaryGroundingMap, Fnv1aHasher16>` then folds the substrate hasher over the
canonical byte layout to mint the `Grounded<ConstrainedTypeInput,
BlockHashTag>` that is cloned into every winning candidate's certificate.
The structural enforcement of `freeRank = 0` comes from `Grounded`'s sealed
constructor plus the `BlockHashTag` phantom that distinguishes prism-btc's
certificate at the type level. A module-scope
`const _: () = assert!(WittLevel::W32.witt_length() == 32);` anchors the
nonce ring at compile time.

Convergence termination is formally proven in Lean
([`prism-btc-lean/PrismBtc/ConvergenceProtocol.lean`](prism-btc-lean/PrismBtc/ConvergenceProtocol.lean)):
the loop either returns a certificate or exhausts the finite nonce fiber — no
third outcome.

## Real-network mining (`prism-btc-node`)

The `prism-mine` binary connects to any running `bitcoind` and drives the full
template → mine → submit cycle. Two modes:

**Single-shot** (default): one template, one 2^32 serial scan, submit once.
For regtest where every header at trivial difficulty satisfies.

```bash
just regtest-demo   # spins up bitcoind, mines 10 blocks
```

**Session** (`--session`): long-running loop with the four pieces real-network
mining needs.

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

What the session adds on top of single-shot:

| Layer | What it does | Why |
|---|---|---|
| **Tip-staleness watcher** | Independent RPC client polls `getbestblockhash` every 500 ms; on tip change, sets the inner cancel flag | A new block on the network invalidates the current parent — wasted work otherwise |
| **Coinbase extranonce rolling** | Bumps a u64 in the coinbase scriptSig on every `NoMatch::Exhausted`, recomputes merkle, retries | 2^32 nonces per template is a hard wall at non-trivial difficulty |
| **Parallel σ-convergence** | rayon partitions the W32 ring into one coset per worker; first finder wins via shared atomic | The natural Z/(2^32)Z partition; SIMD-clean within each coset |
| **Hash-rate reporter** | Sidecar thread reads a shared `AtomicU64` hash counter and prints instant + average MH/s every 5 s | Operational visibility for long runs |

**Safety airlocks:**
- **Chain-mismatch guard**: refuses to mine if `getblockchaininfo.chain` disagrees with the requested `--network`.
- **Mainnet opt-in**: `--network mainnet` requires `--i-know-what-im-doing`. Mainnet difficulty (~PH/s) means a CPU miner cannot find a block in any sane time, so the flag exists to prevent accidental misconfiguration of a long-running deployment.

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

The nonce is intentionally not exposed across the JS boundary — callers receive
the triadic coordinates and the certified hash bytes.

## License

Apache-2.0 — see [LICENSE](LICENSE).
