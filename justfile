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

# Compile-only check on prism-btc — triggers const-validation of the BlockHash CompileUnit
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

# End-to-end regtest demo: spin up bitcoind, mine 10 blocks via prism-btc, show chain state.
# Requires: bitcoind + bitcoin-cli on PATH (e.g. installed under ~/bin/).
# Default datadir: $HOME/regtest-data; override by exporting PRISM_REGTEST_DATA.
regtest-demo:
    #!/usr/bin/env bash
    set -euo pipefail
    DATADIR="${PRISM_REGTEST_DATA:-$HOME/regtest-data}"
    mkdir -p "$DATADIR"
    cat > "$DATADIR/bitcoin.conf" <<EOF
    regtest=1
    server=1
    fallbackfee=0.00001000
    [regtest]
    rpcuser=prism
    rpcpassword=demo
    rpcbind=127.0.0.1
    rpcallowip=127.0.0.1
    rpcport=18443
    EOF
    bitcoind -datadir="$DATADIR" -daemon || true
    bitcoin-cli -datadir="$DATADIR" -rpcwait createwallet "prism" 2>/dev/null || true
    ADDR=$(bitcoin-cli -datadir="$DATADIR" getnewaddress "" "bech32")
    echo "Payout address: $ADDR"
    cargo build --release -p prism-btc-node
    ./target/release/prism-mine \
      --rpc-url http://127.0.0.1:18443 \
      --rpc-user prism --rpc-pass demo \
      --network regtest \
      --payout "$ADDR" \
      --blocks 10
    echo "--- chain state ---"
    bitcoin-cli -datadir="$DATADIR" getblockcount
    bitcoin-cli -datadir="$DATADIR" getbalances

# End-to-end regtest integration test for prism-btc-node (requires bitcoind running).
regtest-test:
    #!/usr/bin/env bash
    set -euo pipefail
    DATADIR="${PRISM_REGTEST_DATA:-$HOME/regtest-data}"
    ADDR=$(bitcoin-cli -datadir="$DATADIR" getnewaddress "" "bech32")
    PRISM_RPC_URL=http://127.0.0.1:18443 \
    PRISM_RPC_USER=prism \
    PRISM_RPC_PASS=demo \
    PRISM_PAYOUT="$ADDR" \
      cargo test -p prism-btc-node --release -- --ignored --nocapture
