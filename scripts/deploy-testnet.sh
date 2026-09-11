#!/usr/bin/env bash
#
# Deploy AJO Onchain to Stellar testnet.
#
# Builds the contracts, uploads the vault WASM, deploys the factory, points
# the factory at the vault hash, and then prints a command a stranger can run
# to check the result for themselves.
#
# That last step is not decoration. An unverifiable trustlessness claim is
# marketing; a one-line command anyone can run is evidence.
#
# Usage:
#   ./scripts/deploy-testnet.sh
#
# Requires: stellar CLI, a funded testnet identity. See docs/development/SETUP.md.

set -euo pipefail

NETWORK="${STELLAR_NETWORK:-testnet}"
IDENTITY="${STELLAR_DEPLOY_IDENTITY:-ajo-deployer}"
OUT_DIR="deployments"

# `stellar contract build` writes here. Honour CARGO_TARGET_DIR if the
# developer has moved the build directory (common on Windows, where building
# inside a OneDrive-synced folder fails).
TARGET_DIR="${CARGO_TARGET_DIR:-contracts/target}"
WASM_DIR="${TARGET_DIR}/wasm32v1-none/release"

info() { printf '\033[0;34m==>\033[0m %s\n' "$*"; }
fail() { printf '\033[0;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

command -v stellar >/dev/null 2>&1 || fail "stellar CLI not found. See docs/development/SETUP.md"

if ! stellar keys address "$IDENTITY" >/dev/null 2>&1; then
  fail "identity '$IDENTITY' not found. Create and fund it:
    stellar keys generate --global $IDENTITY --network $NETWORK --fund"
fi

DEPLOYER_ADDRESS="$(stellar keys address "$IDENTITY")"
info "deploying to $NETWORK as $IDENTITY ($DEPLOYER_ADDRESS)"

# --- 1. Build --------------------------------------------------------------

info "building contracts"
(cd contracts && stellar contract build)

VAULT_WASM="${WASM_DIR}/escrow_vault.wasm"
FACTORY_WASM="${WASM_DIR}/circle_factory.wasm"
[ -f "$VAULT_WASM" ] || fail "missing $VAULT_WASM — did the build succeed?"
[ -f "$FACTORY_WASM" ] || fail "missing $FACTORY_WASM — did the build succeed?"

# --- 2. Upload the vault WASM ----------------------------------------------

# The vault is uploaded but not deployed. The factory instantiates one vault
# per circle from this hash; there is no single shared vault instance, because
# one contract holding every circle's funds would put unrelated groups in the
# same blast radius.
info "uploading escrow-vault wasm"
VAULT_WASM_HASH="$(stellar contract upload \
  --source-account "$IDENTITY" \
  --network "$NETWORK" \
  --wasm "$VAULT_WASM")"
info "vault wasm hash: $VAULT_WASM_HASH"

# --- 3. Deploy the factory --------------------------------------------------

info "deploying circle-factory"
FACTORY_ID="$(stellar contract deploy \
  --source-account "$IDENTITY" \
  --network "$NETWORK" \
  --wasm "$FACTORY_WASM")"
info "factory contract id: $FACTORY_ID"

# --- 4. Initialise the factory ----------------------------------------------

info "initialising factory with the vault hash"
stellar contract invoke \
  --id "$FACTORY_ID" \
  --source-account "$IDENTITY" \
  --network "$NETWORK" \
  -- initialize \
  --admin "$DEPLOYER_ADDRESS" \
  --vault_wasm "$VAULT_WASM_HASH"

# --- 5. Record the deployment ----------------------------------------------

mkdir -p "$OUT_DIR"
DEPLOYMENT_FILE="${OUT_DIR}/${NETWORK}.json"
cat > "$DEPLOYMENT_FILE" <<JSON
{
  "network": "${NETWORK}",
  "deployedAt": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "deployer": "${DEPLOYER_ADDRESS}",
  "factoryContractId": "${FACTORY_ID}",
  "vaultWasmHash": "${VAULT_WASM_HASH}"
}
JSON
info "wrote $DEPLOYMENT_FILE"

# --- 6. Tell the user how to verify it --------------------------------------

cat <<BANNER

-------------------------------------------------------------------------------
 Deployed to ${NETWORK}.

 Add these to your .env:

   FACTORY_CONTRACT_ID=${FACTORY_ID}
   VAULT_WASM_HASH=${VAULT_WASM_HASH}

 Verify it yourself — this command needs no key and no trust in us:

   stellar contract invoke --id ${FACTORY_ID} --network ${NETWORK} -- list_circles

 It returns every circle this factory has deployed, read straight from the
 ledger. Read the source of escrow-vault while you are here and note what is
 not in it: there is no admin withdrawal function.
-------------------------------------------------------------------------------

BANNER
