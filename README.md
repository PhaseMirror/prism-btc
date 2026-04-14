# prism-btc

Bitcoin mining as **σ-convergence** over the nonce fiber of the 32×W8 Datum space.

The block header (80 bytes) is the source space. The σ-projection (SHA256d)
maps each (header, nonce) pair to a candidate 32-byte Datum. The target shape
constraint defines a sub-bundle of the Datum space. Mining is a search for a
nonce whose image under σ lands in that sub-bundle and passes the UOR
`run_pipeline` certification, returning an un-fabricatable `Grounded<BlockHash>`.

> **Note on terminology.** SHA256d is the **σ-projection** (ingestion hash),
> NOT the UOR ψ-map. UOR Foundation reserves ψ for the categorical functor
> chain ψ_1..ψ_9 (Constraints → Nerve → Chain → Homology → … → KInvariants),
> which a deliberately non-shape-preserving avalanche function does not satisfy.

## Workspace

| Crate | Role |
|---|---|
| [`prism-btc-primitives`](crates/prism-btc-primitives/) | Bitcoin scalar newtypes (`Version`, `Timestamp`, `Bits`, …) bound to the UOR ring vocabulary |
| [`prism-btc-types`](crates/prism-btc-types/) | Domain types (`BlockHash`, `Target`, `TriadicCoords`, …) as `#[derive(ConstrainedType)]` ring elements |
| [`prism-btc-reduction`](crates/prism-btc-reduction/) | σ-convergence loop (`run_convergence`), wire-bytes certification (`certify_wire_bytes`), `BlockCertificate` |
| [`prism-btc`](crates/prism-btc/) | Public API: `MiningRound`, `BlockCertificate`, `Boundary`, `genesis()` |
| [`prism-btc-wasm`](crates/prism-btc-wasm/) | `wasm-bindgen` wrapper: `JsBlockHeader`, `mine_block()` (distributed via wasm-pack, not crates.io) |
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
- **Certificate output**: `BlockCertificate` — sealed `Grounded<BlockHash>` + `TriadicCoords` (nonce never observable)
- **Domain types**: `BlockHash`, `BlockHeader`, `MerkleRoot`, `Target`, `TriadicCoords`
- **Primitives**: `Version`, `Timestamp`, `Bits`, `BlockHeight`, `Satoshi`, `FeeRate`, `Address`
- **Boundary trait**: `Boundary` (decode/encode), `BoundaryDecodeError`, `Triadic`
- **Failure type**: `ConvergenceFailure` (`FiberExhausted` | `ReductionStall { reason }`)
- **UOR enforcement**: `Grounded`, `Validated`
- **Genesis**: `genesis()` — formally-grounded genesis block hash constant

No raw bytes, no nonce values, and no convergence mechanics appear in the public surface.

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

// Run the σ-convergence loop. The nonce is internal and never observable.
let cert: BlockCertificate = MiningRound::new(header, Target::new(0x207fffff))
    .converge()
    .expect("easy target must converge");

// Certified hash — sealed Grounded<BlockHash>, cannot be fabricated.
let grounded: &Grounded<BlockHash> = cert.hash();
assert_eq!(grounded.witt_level_bits(), 8); // 32×W8 sites

// PRISM triadic coordinates.
let coords: &TriadicCoords = cert.coords();
println!("datum: {:?}", coords.datum);
println!("stratum: {}", coords.stratum);
println!("spectrum: {}", coords.spectrum);
```

## Genesis hash as a grounded constant

```rust
use prism_btc::genesis;

// Runs uor_ground! at call time — produces a Grounded<BlockHash> that
// cannot be fabricated. The pipeline runs over BlockHash's constraint
// shape and panics at compile-time if the grounding fails.
let grounded = genesis();
assert_ne!(grounded.unit_address(), 0);
assert_eq!(grounded.witt_level_bits(), 32); // W32 grounding level
```

## Boundary: decoding wire-format headers

The `Boundary` trait crosses the raw-bytes / certified-types boundary. `decode`
always re-runs the full pipeline — it cannot be bypassed.

```rust
use prism_btc::prelude::*;

// Decode an 80-byte wire header. Returns Err if length ≠ 80 or if the
// hash fails the run_pipeline certification.
let cert = BlockCertificate::decode(&wire_bytes)?;

// Round-trip back to wire format using the certificate's private fields.
let bytes: Vec<u8> = cert.encode();
```

## σ-convergence loop

```text
for nonce in 0..=u32::MAX:
    raw      = serialize_header(header, nonce)            // 80-byte wire format
    candidate = sha256d(raw)                               // σ-projection (NOT ψ-map)
    if candidate > target_bytes: continue                  // fast pre-filter
    grounded = run_pipeline(BlockHash(candidate), W8)      // formal SAT certification
    return BlockCertificate { grounded, coords, ... }
return Err(FiberExhausted)
```

`#[uor_grounded(level = "W32")]` on `converge_at_w32()` emits a static Witt level
assertion that the nonce iterates in Z/(2^32)Z. The `Grounded<BlockHash>` wrapper
has a sealed constructor — only `run_pipeline` and `uor_ground!` can produce it,
which structurally enforces `freeRank = 0` without any runtime `FreeRank` object.

Convergence termination is formally proven in Lean
([`prism-btc-lean/PrismBtc/ConvergenceProtocol.lean`](prism-btc-lean/PrismBtc/ConvergenceProtocol.lean)):
the loop either returns a certificate or exhausts the finite nonce fiber — no
third outcome.

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
