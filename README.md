# AJO Onchain

**Rotating savings circles, with a market for early access to the pot.**

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Contributions welcome](https://img.shields.io/badge/contributions-welcome-brightgreen.svg)](./CONTRIBUTING.md)
[![Good first issues](https://img.shields.io/badge/good%20first%20issues-9-orange.svg)](./docs/ISSUES.md)

---

## What this is

Across Nigeria, millions of people save through **Ajo** — also called Esusu
or Adashe. A group of neighbours, market traders or colleagues agrees to
contribute a fixed amount every week or month. Each round, one member takes
the entire pot. The turn rotates until everyone has had one, and then the
circle starts again or disbands.

It works because the group knows each other. There is no credit check and no
bank; there is a collector who holds the money and social pressure that makes
people pay. For a trader who needs ₦200,000 to restock and has no access to
credit, it is the difference between growing and standing still.

It also breaks in a specific, predictable way. **Members need the money
before their turn comes.** A medical bill, school fees, an unexpected
opportunity. Today that gets resolved socially: begging the collector to move
you up, quietly swapping slots with someone, or defaulting and taking the
circle down with you. And the collector holds everyone's savings, which
requires trusting one person with a great deal of money.

AJO Onchain replaces the collector with a smart contract on Soroban, and then
**prices** the early-access problem instead of leaving it to social
negotiation. A member bids a discount for early access to the pot, and that
discount is distributed as a dividend to the members who wait. Waiting stops
being a favour and starts being a yield.

## Status

**Pre-development. Tranche 1 in progress.**

The contracts compile, build to wasm, and have one placeholder test plus a
test harness. Nothing is deployed. The indexer runs but its handlers are
stubs. The web app is a README. The auction is documentation only, blocked on
an unresolved design question we have written down rather than guessed at.

We would rather say that plainly than imply more. Every stub is a numbered
`TODO(#n)` with a matching entry in [`docs/ISSUES.md`](./docs/ISSUES.md).

| Tranche | Scope | Status |
|---|---|---|
| 1 — Activation | Circle creation, joining, fixed-order contribution and payout, default flagging, SEP-24 NGN↔USDC ramp, testnet deployment, mobile-first UI | **In progress** |
| 2 — Liquidity | Sealed-bid commit/reveal, dividend distribution, once-per-cycle-winner enforcement | Scaffolded only |
| 3 — Trust & credit | On-chain contribution history, governance to replace a defaulter, AI scoring and default early warning, external audit | Scaffolded only |
| 4 — Mainnet | Pilot cohort, metrics reporting | Not started |

## How it differs

Rotating savings on Stellar is not an empty category. Three other teams work
in it, and two of them have shipped things we have not.

| | Custodial fintechs | SoroSusu | Sharibo | AJO Onchain |
|---|---|---|---|---|
| Custody | Company / partner bank | Smart contract | Smart contract | Smart contract |
| Core unlock | Record-keeping UX | Base rotation | Payout privacy (ZK) | **Early liquidity via auction** |
| Fiat access (NGN) | Native | None | None | **SEP-24 anchor** |
| Portable credit history | Locked to one company | None | None | **On-chain registry** |

**[SoroSusu](https://github.com/sorosusu)** proved the base rotation contract
works on mainnet. That is further than we are.
**[Sharibo](https://github.com/sharibo)** proved on-chain payout privacy is
achievable using Stellar's native BLS12-381 pairings, which is genuinely
hard. Neither addresses early liquidity or fiat access, and that gap is what
this project is for.

We are also not trying to own the credit layer. The `ReputationRegistry` is
designed to be **readable by any Stellar protocol, SoroSusu included**,
because one shared credit primitive for African ROSCA users is worth more to
those users than three siloed ones — and more to us too, since a registry
only we read is only as useful as we are large.

## Verify it yourself

Nothing is deployed yet, so these commands carry placeholders. They will be
filled in the day the testnet deployment happens, and you should hold us to
it — an unverifiable claim about trustlessness is marketing.

```bash
# Every circle the factory has deployed, read straight from the ledger.
stellar contract invoke --id <FACTORY_ID> --network testnet -- list_circles

# A circle's full state: members, round, phase, pot, and all three bitmaps.
stellar contract invoke --id <VAULT_ID> --network testnet -- get_state

# Its terms, fixed at creation and with no setter anywhere.
stellar contract invoke --id <VAULT_ID> --network testnet -- get_config
```

These need no key, no account, and no trust in us.

**Note what is not there: no admin withdrawal function exists anywhere in
`EscrowVault`. Read the source and check.** Funds leave the contract only
through `settle_and_pay`, only to a member of that circle, and only to the
recipient the contract recomputes for itself. There is no `pause`, no
`set_admin`, no `emergency_withdraw`, and no upgrade hook that could add one.
The complete interface is documented in
[`docs/contracts/escrow-vault.md`](./docs/contracts/escrow-vault.md); compare
it against [the source](./contracts/escrow-vault/src/lib.rs).

## Architecture

```
                         TRUSTLESS  (enforced by contract)
        ┌──────────────────────────────────────────────────────────┐
        │                                                          │
        │   CircleFactory ──deploy──> EscrowVault <──transfer──> USDC
        │                                  │        (SEP-41)       │
        │                                  │                       │
        │                                  └──history──> ReputationRegistry
        │                                                     ▲    │
        └─────────────────┬────────────────────────────────┬──┼────┘
                          │ events                  read   │  │ signed
                          ▼ (read)                         │  │ attestation
        ┌─────────────────────────────────────────────────────┼────┐
        │                                                     │    │
        │   apps/indexer ──> Supabase ──read──> apps/web      │    │
        │   (no keys)         (cache)           (thin BFF)    │    │
        │                                                     │    │
        │   services/scoring ─────────────────────────────────┘    │
        │   (advisory, Tranche 3)                                  │
        │                                                          │
        └──────────────────────────────────────────────────────────┘
                         ADVISORY  (cannot move funds)
```

**The contracts are the backend. Everything off-chain is a read cache or an
advisory service.**

The indexer holds no keys and signs nothing. If it dies, nobody loses money —
the dashboard goes stale, and circles keep running, because `advance_phase`
and `settle_and_pay` are permissionless and anyone can poke a circle whose
deadline has passed.

Mermaid versions, including the round state machine, are in
[`docs/architecture/diagrams.md`](./docs/architecture/diagrams.md).

## Repository layout

```
contracts/            Cargo workspace — the crown jewels
  shared/             types, errors, event schemas
  circle-factory/     deploys one vault per circle
  escrow-vault/       custody + round state machine        [Tranche 1]
  bid-auction/        sealed-bid commit/reveal             [Tranche 2 stub]
  reputation-registry/ portable credit history             [Tranche 3 stub]
apps/
  web/                Next.js UI + thin BFF                [README only]
  indexer/            Soroban event subscriber + reminders
services/
  scoring/            FastAPI credit scoring               [README only]
packages/
  contract-bindings/  generated TS bindings (gitignored)
docs/                 architecture, contracts, setup, issues
audit/                scope, threat model, known issues
scripts/              deploy and binding generation
```

## Quick start

```bash
git clone https://github.com/<ORG>/ajo-onchain
cd ajo-onchain

cp .env.example .env
bun install
task check        # fmt, clippy, tests, wasm build, lint
```

You need Rust 1.84+, the `wasm32v1-none` target, the Stellar CLI, Bun and
Task. Full instructions, including the common failure modes, are in
[`docs/development/SETUP.md`](./docs/development/SETUP.md).

If `task check` does not pass on a clean clone, that is our bug. Open an
issue.

## Roadmap

**Tranche 1 — Activation.** Circle creation, joining, contribution and
payout, default flagging, the SEP-24 naira ramp, testnet deployment, and a
mobile-first UI. The contract entry points exist; the tests and the UI do
not.

**Tranche 2 — Liquidity.** The sealed-bid discount auction. Blocked, on
purpose, by [`known-issues.md` #1](./audit/known-issues.md): a bid winner can
take the pot and stop contributing, and we have not chosen between a bond, a
reputation-weighted auction, or a group clawback vote. Each implies a
different contract, so building before deciding would waste the work.

**Tranche 3 — Trust and credit.** On-chain contribution history, governance
to replace a defaulter without breaking bitmap indices, advisory credit
scoring, and an external audit.

**Tranche 4 — Mainnet.** A pilot cohort and public metrics.

## Security

The [`audit/`](./audit/) directory is not a placeholder:

- [`scope.md`](./audit/scope.md) — what we want audited, what we do not, and
  the four assumptions the review rests on.
- [`threat-model.md`](./audit/threat-model.md) — assets, actors, and seven
  threats with their current status. T4 is marked unmitigated.
- [`known-issues.md`](./audit/known-issues.md) — four issues we filed against
  ourselves, including a real ordering flaw in `settle_and_pay` that is still
  in the code, with the fix and its severity written down.

We would rather a reviewer find our bugs already documented than discover we
had not looked. If you find something we have missed, please open a private
security advisory rather than a public issue.

### The three invariants

1. **No privileged withdrawal.** No admin withdrawal function exists anywhere
   in `EscrowVault`. Funds leave only through `settle_and_pay`, and only to a
   member of that circle.
2. **No phase skipping.** Phases advance exactly one step at a time via
   `advance_phase`. No entry point accepts a target phase from the caller.
3. **One win per cycle.** A member may receive at most one payout before
   every member has received one.

A pull request that weakens any of these will be rejected, however good the
reason.

## Contributing

**New to Rust, Soroban, or open source? You are exactly who
[`CONTRIBUTING.md`](./CONTRIBUTING.md) is for.**

There are 22 scoped, claimable tasks in
[`docs/ISSUES.md`](./docs/ISSUES.md) — nine of them marked good first issue,
each with the exact file, the marker to grep for, and a definition of done.
The contract test harness is already written, so claiming a test issue means
writing assertions rather than setup.

```bash
grep -rn "TODO(#" --include="*.rs" --include="*.ts" .
```

## License

[MIT](./LICENSE)
