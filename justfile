default:
    @just --list

build:
    cargo build --workspace

build-release:
    cargo build --workspace --release

build-wasm:
    wasm-pack build crates/prism-btc-wasm --target web --out-dir ../../pkg/prism-btc-wasm --release

test:
    cargo test --workspace --exclude prism-btc-wasm

test-wasm:
    wasm-pack test crates/prism-btc-wasm --node

check:
    cargo check --workspace

lint:
    cargo clippy --workspace -- -D warnings

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

# Compile-only check on prism-btc — triggers uor_ground! genesis grounding
check-genesis:
    cargo check -p prism-btc

verify:
    cd prism-btc-lean && lake update && lake build

verify-check:
    cd prism-btc-lean && lake check

# Fast CI (excludes Lean and wasm-pack — run separately)
ci:
    just fmt-check
    just check
    just lint
    just test
    just check-genesis

bench:
    cargo bench -p prism-btc

doc:
    cargo doc --workspace --no-deps --open
