# Development setup

If any step here does not work, that is our bug. Open an issue — a broken
setup guide costs us contributors, and you are not the only person hitting
it.

## Prerequisites

| Tool | Version | Why | Install |
|---|---|---|---|
| Rust | 1.84+ | Contracts. The `wasm32v1-none` target needs 1.84 or newer. | [rustup.rs](https://rustup.rs) |
| `wasm32v1-none` target | — | The only wasm target the Soroban runtime accepts. | `rustup target add wasm32v1-none` |
| Stellar CLI | 23+ | Building, deploying and invoking contracts. | `cargo install --locked stellar-cli` or a [release binary](https://github.com/stellar/stellar-cli/releases) |
| Bun | 1.2+ | Indexer and web app. | [bun.sh](https://bun.sh) |
| Task | 3+ | Task runner. | [taskfile.dev/installation](https://taskfile.dev/installation/) |
| Git | any | — | — |

`contracts/rust-toolchain.toml` pins the channel, the wasm target and the
`rustfmt`/`clippy` components, so `rustup` installs what is needed the first
time you run a cargo command in `contracts/`. You do not need to pick a
version yourself.

## First run

```bash
git clone https://github.com/ajo-onchain/ajo-onchain
cd ajo-onchain

cp .env.example .env       # placeholders are fine for tests
bun install
task check                 # fmt, clippy, tests, wasm build, lint
```

`task check` runs everything CI runs. Green here should mean green there.

**If `task contracts:test` does not pass on a clean clone, that is our bug,
not yours.** Open an issue.

Run `task` with no arguments to list every available task.

## Funding a testnet account

```bash
# Create a key in the local keystore. The secret stays there; it is never
# written to .env and never leaves your machine.
stellar keys generate --global ajo-deployer --network testnet --fund

stellar keys address ajo-deployer
```

`--fund` asks friendbot for test XLM. If it fails, friendbot is occasionally
down; retry, or fund the address at
[laboratory.stellar.org](https://laboratory.stellar.org).

## Deploying to testnet

```bash
task contracts:deploy:testnet
```

This builds the contracts, uploads the vault WASM, deploys the factory,
initialises it, writes `deployments/testnet.json`, and prints the contract
ids to put in your `.env`.

It finishes by printing a verification command:

```bash
stellar contract invoke --id <FACTORY_ID> --network testnet -- list_circles
```

Run it. It reads straight from the ledger and needs no key and no trust in
us. That is the point — a claim about trustlessness that you cannot check is
just marketing.

## Running the indexer

```bash
task indexer:dev
```

It needs `SOROBAN_RPC_URL`, `STELLAR_NETWORK_PASSPHRASE`,
`FACTORY_CONTRACT_ID`, `SUPABASE_URL` and `SUPABASE_SERVICE_ROLE_KEY`, and
fails immediately at startup if any is missing rather than halfway through
its first poll.

Most of its work is still stubbed — `grep -rn "TODO(#" apps/indexer` to see
what is claimable.

## Common problems

### `error[E0463]: can't find crate for 'core'` / unknown target `wasm32v1-none`

The wasm target is missing:

```bash
rustup target add wasm32v1-none
```

**This recurs after every Rust update** — targets are installed per
toolchain, so updating Rust silently leaves you without it. If a build that
worked yesterday fails today with a missing-target error, this is why.

Also check you are not following an older guide: `wasm32-unknown-unknown` is
out of date for Soroban.

### Clippy passes locally but fails in CI

CI runs `cargo clippy --all-targets -- -D warnings`. Locally, `cargo clippy`
without `-D warnings` prints the same lints and exits 0, so it is easy to
miss them.

Run what CI runs:

```bash
task contracts:lint
```

### `cargo fmt` passes locally but CI disagrees

`rustfmt` output can differ between toolchain versions. Make sure you are on
the pinned toolchain — run cargo from inside `contracts/` so
`rust-toolchain.toml` applies, rather than from the repository root with an
explicit `--manifest-path`.

### Windows: `failed to create directory ... target` / `Access is denied`

Building inside a OneDrive-synced folder can fail, and is slow even when it
works, because the sync client holds files open. Point the build output
somewhere local:

```powershell
setx CARGO_TARGET_DIR "$env:LOCALAPPDATA\ajo-target"
```

Open a new terminal afterwards. `scripts/deploy-testnet.sh` and
`scripts/gen-bindings.sh` both honour `CARGO_TARGET_DIR`.

### Windows: `cargo test` fails to link with `export ordinal too large`

This affects the `x86_64-pc-windows-gnu` toolchain only. The contracts are
`cdylib` crates, and building a host DLL from one exceeds a limit in the
mingw linker. The contracts themselves are fine — the wasm build and the
tests both work.

Two options:

- Install the MSVC toolchain (Visual Studio Build Tools with the C++
  workload) and use `stable-x86_64-pc-windows-msvc`. This is the better
  long-term fix.
- Or run the library tests only, which is what the failure is blocking:

  ```bash
  cargo test --all --lib
  ```

CI runs on Linux and is unaffected.

### `bun install` seems to install nothing

Bun 1.4 uses isolated installs. Workspace dependencies are linked into
`apps/<name>/node_modules` and the real packages live in
`node_modules/.bun/`, so the root `node_modules` looks almost empty. This is
correct. Check with `bun pm ls`.

### Biome: "couldn't find an ignore file"

`biome.json` sets `vcs.useIgnoreFile`, so it expects `.gitignore` to exist.
If you are working in a copy of the tree without it, restore it.

## Where to go next

- [`CONTRIBUTING.md`](../../CONTRIBUTING.md) — where the work is, and the
  three invariants.
- [`docs/ISSUES.md`](../ISSUES.md) — every `TODO(#n)` written up as a
  claimable task.
- [`contracts/escrow-vault/src/test.rs`](../../contracts/escrow-vault/src/test.rs)
  — start here for contract work. The harness is already written.
