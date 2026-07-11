#!/usr/bin/env bash
# optimize.sh — Build and optimize the contracts-registry WASM artifact.
#
# Usage:
#   cd contracts/registry
#   bash optimize.sh
#
# Prerequisites:
#   • Rust + wasm32-unknown-unknown target
#   • Stellar CLI (stellar contract optimize)
#     Install: cargo install --locked stellar-cli --features opt
#
# Steps:
#   1. Build the release WASM.
#   2. Print the unoptimized size.
#   3. Run stellar contract optimize.
#   4. Print the optimized size.
#   5. Print the delta so reviewers can verify the optimization was worthwhile.

set -euo pipefail

# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TARGET_DIR="$WORKSPACE_ROOT/target/wasm32v1-none/release"
WASM_RAW="$TARGET_DIR/contracts_registry.wasm"
WASM_OPT="$TARGET_DIR/contracts_registry.optimized.wasm"

# ---------------------------------------------------------------------------
# Check prerequisites
# ---------------------------------------------------------------------------

if ! command -v stellar &>/dev/null; then
    echo "Error: 'stellar' CLI not found."
    echo "Install with: cargo install --locked stellar-cli --features opt"
    exit 1
fi

# ---------------------------------------------------------------------------
# Step 1: build release WASM
# ---------------------------------------------------------------------------

echo "==> Building release WASM..."
cargo build \
    --target wasm32v1-none \
    --release \
    --manifest-path "$SCRIPT_DIR/Cargo.toml" \
    --package contracts-registry

# ---------------------------------------------------------------------------
# Step 2: print unoptimized size
# ---------------------------------------------------------------------------

SIZE_BEFORE=$(wc -c < "$WASM_RAW")
echo "==> Unoptimized WASM size: ${SIZE_BEFORE} bytes  ($(( SIZE_BEFORE / 1024 )) KiB)"

# ---------------------------------------------------------------------------
# Step 3: optimize
# ---------------------------------------------------------------------------

echo "==> Running stellar contract optimize..."
stellar contract optimize --wasm "$WASM_RAW"

# ---------------------------------------------------------------------------
# Step 4: print optimized size
# ---------------------------------------------------------------------------

SIZE_AFTER=$(wc -c < "$WASM_OPT")
echo "==> Optimized WASM size:   ${SIZE_AFTER} bytes  ($(( SIZE_AFTER / 1024 )) KiB)"

# ---------------------------------------------------------------------------
# Step 5: delta
# ---------------------------------------------------------------------------

DELTA=$(( SIZE_BEFORE - SIZE_AFTER ))
if (( SIZE_BEFORE > 0 )); then
    PCT=$(( DELTA * 100 / SIZE_BEFORE ))
else
    PCT=0
fi

echo "==> Saved ${DELTA} bytes (${PCT}% reduction)"
echo "==> Optimized artifact: $WASM_OPT"
