# Contributing to AJO Onchain

**New to Rust, Soroban, or open source? You are exactly who this file is
for.**

This project needs people who understand Ajo circles at least as much as it
needs people who understand smart contracts. If you have run one, saved in
one, or watched one collapse, you know things about this problem that no
amount of Rust does. Say so in an issue — that is a contribution.

## Setup in five minutes

```bash
git clone https://github.com/ajo-onchain/ajo-onchain
cd ajo-onchain

cp .env.example .env      # placeholders are fine for tests
bun install
task check
```

`task check` runs everything CI runs: Rust formatting, clippy, the contract
tests, the wasm build, and the TypeScript lint.

**If `task contracts:test` doesn't pass, open an issue. A broken setup is our
bug, not yours.** You are almost certainly not the only person hitting it,
and the others just closed the tab.

You need Rust 1.84+, the `wasm32v1-none` target, the Stellar CLI, Bun and
Task. [`docs/development/SETUP.md`](./docs/development/SETUP.md) has install
commands and a common-problems section covering the failures we have actually
hit — missing wasm target, clippy passing locally but failing in CI, and two
Windows-specific ones.

Run `task` with no arguments to see every available task.

## Where the work is

| Area | Skills | Path |
|---|---|---|
| Contract tests | Rust basics. No blockchain experience needed — the harness is written. | `contracts/escrow-vault/src/test.rs` |
| Contract logic | Rust, Soroban | `contracts/escrow-vault/`, `contracts/circle-factory/` |
| Indexer handlers | TypeScript, SQL basics | `apps/indexer/src/handlers/` |
| Event polling | TypeScript, Stellar SDK | `apps/indexer/src/lib/stellar.ts` |
| Reminders | TypeScript, WhatsApp Business API | `apps/indexer/src/lib/reminders.ts` |
| Web app | Next.js, Tailwind, mobile-first | `apps/web/` (not scaffolded yet) |
| Documentation | Writing clearly | `docs/`, and any comment that confused you |
| Ajo domain knowledge | Having lived it | Open an issue |

### Finding something to do

Every stub in the repository carries a numbered marker:

```bash
grep -rn "TODO(#" --include="*.rs" --include="*.ts" .
```

Each one has a matching write-up in [`docs/ISSUES.md`](./docs/ISSUES.md) with
the exact file, a definition of done, the skills needed and an effort
estimate. Nine are marked **good first issue**.

Good places to start:

- **[#2](./docs/ISSUES.md) — a member cannot contribute twice in one round.**
  Rust basics, 1–2 hours. The harness deploys the vault and funds the members
  for you.
- **[#14](./docs/ISSUES.md) — handle the `created` event.** TypeScript and
  basic SQL, 1–2 hours.
- **[#7](./docs/ISSUES.md) — `join` is rejected once the circle has started.**
  Rust basics, 1–2 hours, and it covers a real threat (T7).

**Comment on the issue to claim it** so two people don't do the same work. If
you get stuck, ask in the issue. A question is not a failure; a silently
abandoned branch helps nobody.

## Branches and commits

Branch from `main`:

```
feat/short-description
fix/short-description
docs/short-description
test/short-description
```

Commits follow [Conventional Commits](https://www.conventionalcommits.org/),
enforced by commitlint on a git hook:

```
test(vault): reject a second contribution in one round

Closes #2.
```

Valid scopes are listed in `commitlint.config.js`. They exist because
`git log --oneline -- contracts/` is how a reviewer or an auditor
reconstructs what happened, and a scope makes that readable without opening
every diff.

## Before you open a pull request

- [ ] `task check` passes.
- [ ] Your change has a test that fails if the change is reverted.
- [ ] You removed the `TODO(#n)` marker and its entry in `docs/ISSUES.md`.
- [ ] If you changed an event topic **or a field name** in
      `contracts/shared/src/events.rs`, you also updated
      `apps/indexer/src/handlers/index.ts` and `docs/contracts/events.md`.
      These must move together — otherwise the indexer stops seeing the
      event, nothing fails, and the dashboard quietly goes stale.

The pull request template asks whether your change touches fund movement,
phase transitions, or the winners bitmap. Answer honestly. Ticking the box
is not a problem — those changes are expected and welcome. Ticking it
wrongly, or leaving it unticked when it applies, wastes a reviewer's
attention on rediscovering what you already knew.

## The three invariants

These are the entire value proposition of the project. Everything else is
implementation detail; these are the product.

1. **No privileged withdrawal.** There is no admin withdrawal function
   anywhere in `EscrowVault`. Funds leave the contract only through
   `settle_and_pay`, and only to a member of that circle. No `pause`, no
   `set_admin`, no `emergency_withdraw`, no upgrade hook that could introduce
   one.
2. **No phase skipping.** Phases advance exactly one step at a time via
   `advance_phase`. No entry point accepts a target phase from the caller.
3. **One win per cycle.** A member may receive at most one payout before
   every member has received one, enforced by a bitmap in contract state.

**A pull request that weakens any of these will be rejected, however good the
reason.**

That is deliberately absolute, and it is not about your code. A member of an
Ajo circle in Lagos cannot audit a Rust contract. What they can be told is a
short sentence — *nobody can take this money except you, when it is your
turn* — and that sentence has to stay true without qualification. An
invariant with a good exception is not an invariant; it is a default, and a
default cannot be explained to someone deciding whether to trust the thing
with their savings.

The pragmatic version of the same point: these three properties are what an
auditor will be asked to confirm, and what the project's credibility rests
on. Weakening one costs far more than any feature it buys.

**If you believe an invariant must change,** open a discussion first and add
your reasoning to [`audit/known-issues.md`](./audit/known-issues.md) before
writing any code. The discussion comes before the pull request, not after it.
That is how issue #2 in that file got there — we found an ordering flaw in
our own `settle_and_pay`, wrote it down with its severity, and left it in the
code with a pointer, rather than quietly patching it and hoping nobody
noticed.

### The corollary: off-chain components cannot move funds

The contracts are the backend. Everything off-chain is a read cache or an
advisory service:

- **`apps/indexer`** holds no keys and signs nothing. If it dies, nobody
  loses money — the dashboard goes stale.
- **`services/scoring`** produces advisory output. A pull request giving it a
  path to move funds should be rejected on sight.
- **`apps/web`** route handlers are a thin BFF only: session, the SEP-24
  anchor callback proxy, and calls needing a secret. No business logic, no
  long-lived connections, no scheduled work.

A rule enforced in a route handler is a rule a malicious client skips by
calling the contract directly. If the frontend and the contract ever
disagree, the contract wins and the UI has been lying.

## Code of conduct

By participating you agree to the
[Code of Conduct](./CODE_OF_CONDUCT.md).

## Questions

Open an issue. There is no such thing as a question that is too basic here —
if the setup guide, a comment, or an error message confused you, that is a
documentation bug and reporting it is a real contribution.
