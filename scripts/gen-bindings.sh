#!/usr/bin/env bash
#
# Generate TypeScript bindings for the contracts.
#
# Output goes to packages/contract-bindings/src/ and is gitignored. Bindings
# are derived from the contract spec, so committing them guarantees that one
# day they will disagree with the contracts they claim to describe, and the
# disagreement will be discovered by a user rather than by CI. Regenerate
# instead: `task bindings`.
#
# Usage:
#   ./scripts/gen-bindings.sh

set -euo pipefail

NETWORK="${STELLAR_NETWORK:-testnet}"
OUT_ROOT="packages/contract-bindings"
TARGET_DIR="${CARGO_TARGET_DIR:-contracts/target}"
WASM_DIR="${TARGET_DIR}/wasm32v1-none/release"

info() { printf '\033[0;34m==>\033[0m %s\n' "$*"; }
fail() { printf '\033[0;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

command -v stellar >/dev/null 2>&1 || fail "stellar CLI not found. See docs/development/SETUP.md"

info "building contracts"
(cd contracts && stellar contract build)

mkdir -p "${OUT_ROOT}/src"

# Bindings are generated from the local wasm rather than from a deployed
# contract id, so this works before anything has been deployed and stays
# reproducible in CI.
for contract in escrow_vault circle_factory; do
  wasm="${WASM_DIR}/${contract}.wasm"
  [ -f "$wasm" ] || fail "missing $wasm — did the build succeed?"

  info "generating bindings for ${contract}"
  stellar contract bindings typescript \
    --wasm "$wasm" \
    --output-dir "${OUT_ROOT}/src/${contract}" \
    --overwrite
done

info "bindings written to ${OUT_ROOT}/src/ (gitignored)"
