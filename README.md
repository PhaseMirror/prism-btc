# prism-btc

Bitcoin mining reframed as a UOR shape-preserving morphism search.

SHA256d is the ψ-map: it maps an 80-byte block header (the *CompileUnit*) to a
32-byte candidate hash (the *Datum*).  `uor_foundation::pipeline::run_pipeline`
then formally certifies that the Datum's shape satisfies the target constraint,
returning an un-fabricatable `Grounded<BlockHash>` — the structural proof that
`freeRank = 0`.

## Workspace

| Crate | Role |
|---|---|
| [`prism-btc-primitives`](crates/prism-btc-primitives/) | Bitcoin scalar newtypes (`Version`, `Timestamp`, `Bits`, …) bound to the UOR ring vocabulary |
| [`prism-btc-types`](crates/prism-btc-types/) | Domain types (`BlockHash`, `Target`, `TriadicCoords`, …) as `#[derive(ConstrainedType)]` ring elements |
| [`prism-btc-reduction`](crates/prism-btc-reduction/) | `NonceIter`, `MiningCertificate`, `genesis_grounded()` |
| [`prism-btc`](crates/prism-btc/) | Public API: `mine()`, `genesis_block_hash()`, `sha256d()`, `serialize_header()` |
| [`prism-btc-wasm`](crates/prism-btc-wasm/) | `wasm-bindgen` wrapper: `JsBlockHeader`, `mine_block()` |
| [`prism-btc-lean/`](prism-btc-lean/) | Lean 4 formal proofs: ring identity (W8/W32), triadic coords, FreeRank protocol, shape constraint monotonicity |

## Quick start

```bash
# Prerequisites: Rust stable + just
cargo install just

just build          # cargo build --workspace
just test           # cargo test --workspace (mining tests are #[ignore] — see below)
just lint           # cargo clippy -D warnings
just fmt-check      # cargo fmt --check

# Formal proofs (requires elan / Lean 4)
just verify         # lake update && lake build

# WebAssembly (requires wasm-pack)
just build-wasm
```

## Architecture: the ψ-loop

```
for nonce in 0..=u32::MAX:
    raw      = serialize_header(header, nonce)   // 80-byte wire format
    hash     = sha256d(raw)                       // ψ-map (double-SHA256, BE output)
    if hash > target: continue                    // fast pre-filter
    grounded = run_pipeline(&BlockHash(hash), 8)  // 7-stage SAT certification
    return MiningCertificate { grounded, nonce }
```

`#[uor_grounded(level = "W32")]` on `mine()` emits a static Witt level assertion
that the nonce iterates in Z/(2^32)Z.  The `Grounded<BlockHash>` wrapper has a
sealed constructor — only `run_pipeline` and `uor_ground!` can produce it, which
structurally enforces `freeRank = 0` without any runtime `FreeRank` object.

## Mining the genesis block

```rust
use prism_btc::{mine, BlockHeader, Target, MerkleRoot};
use prism_btc_primitives::{Version, Timestamp, Bits};

let header = BlockHeader {
    version:     Version(1),
    prev_hash:   [0u8; 32],
    merkle_root: MerkleRoot::from_bytes([
        0x3b, 0xa3, 0xed, 0xfd, 0x7a, 0x7b, 0x12, 0xb2,
        0x7a, 0xc7, 0x2c, 0x3e, 0x67, 0x76, 0x8f, 0x61,
        0x7f, 0xc8, 0x1b, 0xc3, 0x88, 0x8a, 0x51, 0x32,
        0x3a, 0x9f, 0xb8, 0xaa, 0x4b, 0x1e, 0x5e, 0x4a,
    ]),
    timestamp:   Timestamp(1231006505),
    bits:        Bits(0x1d00ffff),
};

// Run with `--release`; iterates ~2.08 billion nonces.
let cert = mine(&header, &Target::new(Target::GENESIS_NBITS)).unwrap();
assert_eq!(cert.nonce(), 2083236893);
```

The genesis block can also be obtained instantly from the pre-verified constant:

```rust
let grounded = prism_btc::genesis_block_hash();
assert_eq!(grounded.witt_level_bits(), 32);
```

## Running slow tests

Mining tests are annotated `#[ignore]` to keep `cargo test` fast.  Run them
explicitly in release mode:

```bash
just test-slow   # cargo test --workspace --release -- --ignored
```

## WebAssembly

```bash
just build-wasm   # wasm-pack build → pkg/prism-btc-wasm/
```

```js
import init, { JsBlockHeader, mine_block } from './prism_btc_wasm.js';
await init();
const header = new JsBlockHeader(version, prevHashBytes, merkleBytes, timestamp, bits);
const result  = mine_block(header, 0x1d00ffff);
console.log(result.nonce, result.stratum, result.spectrum);
```

## License

Apache-2.0 — see [LICENSE](LICENSE).
