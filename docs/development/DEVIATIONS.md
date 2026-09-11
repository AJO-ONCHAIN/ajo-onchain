# Deviations from the implementation brief

Every place the scaffold departs from the original build brief, and why.
Recorded so that a reviewer comparing the two does not have to guess whether
a difference was a decision or an accident.

---

## 1. Soroban SDK 27, not 23

**Brief:** `soroban-sdk = "23"`.
**Built:** `soroban-sdk = "27"` (resolved 27.0.6).

SDK 23 is four major versions behind. The brief's own instruction is to fix
the code to match the current SDK rather than silently pin an old one, and
only the two most recent majors receive security fixes. Built and verified
against 27.0.6 with Rust 1.98.1 and Stellar CLI 28.0.0.

## 2. Events use `#[contractevent]`, not `env.events().publish`

**Brief:** "topic constants and thin publish helpers".
**Built:** seven `#[contractevent]` structs in `contracts/shared/src/events.rs`,
still wrapped in thin `emit_*` helpers.

`Events::publish` is deprecated in SDK 27 and emits a warning, which fails
CI's `cargo clippy -D warnings`. The macro is the current API, and it also
registers the events in the contract spec so generated bindings know about
them.

The `emit_*` helper functions are kept, so the brief's intent — one call site
per event, payload shape defined in one place — is unchanged.

## 3. Event payloads are named maps, not tuples

A consequence of deviation 2. `#[contractevent]` publishes non-topic fields
as a map keyed by field name, so the indexer reads `amount` rather than "the
second element of the tuple".

This is an improvement for the indexer contract, but it widens what counts as
a breaking change: **renaming a field is now as breaking as renaming a
topic.** Called out in `docs/contracts/events.md` and in the pull request
template.

## 4. No parallel table of `Symbol` topic constants

**Brief:** "topic constants and thin publish helpers".
**Built:** topic strings are declared once, in each struct's `topics = [...]`
argument.

Keeping `pub const TOPIC_CREATED: Symbol` alongside
`#[contractevent(topics = ["created"])]` would be two sources of truth for a
wire format, and the one that drifts is the one that silently breaks the
indexer. The topic table in the module doc comment and in
`docs/contracts/events.md` serves the documentation purpose the constants
would have served.

## 5. `fee_bps` is validated at `initialize`

**Brief:** `initialize` rejects `member_cap == 0 || > 32` and
`contribution <= 0`.
**Built:** those, plus `fee_bps > 10_000` (100%).

Without the check, a misconfigured circle computes a negative `net` in
`settle_and_pay` and the payout transfer panics mid-settlement, stranding the
pot in a vault with no way to release it — and invariant 1 means there is no
admin function to rescue it. The brief invites improvements outside the three
invariants and the repository layout; this is one.

## 6. `cycle_complete(0)` returns `false`

**Brief:** compare `winners_bitmap` against a full mask, guarding the
`member_count >= 32` shift overflow.
**Built:** that, plus an explicit `member_count == 0` case returning `false`.

The mask for zero members is `0`, and `0 & 0 == 0` would report a circle with
no members as complete — letting a degenerate circle emit `cycdone` and reset
its winners bitmap. "Vacuously true" is the wrong answer to hand a state
machine that acts on it. Unreachable in practice, because `initialize` seats
the founder as member 0.

## 7. `settle_and_pay` rejects `BidBased` circles

**Brief:** for rotational circles, recompute the expected recipient. Silent
on what a bid-based circle does before Tranche 2.
**Built:** `Variant::BidBased` returns `Error::NoWinningBid`.

The alternative — falling through to rotational order — would pay a
plausible-looking but wrong member, and would do it silently. An error is the
honest behaviour for a mechanism that does not exist yet.

## 8. The indexer has a `src/lib/config.ts`

**Brief:** lists four indexer files.
**Built:** those four plus `src/lib/config.ts`.

Environment loading was going to be duplicated between `index.ts` and
`reminders.ts`. Centralising it also makes the absence of any signing key
visible in one place — the interface is the complete list of secrets the
process holds.

## 9. `test.rs` has nine TODOs, not five

**Brief:** five numbered TODOs.
**Built:** those five verbatim, plus `#7`–`#10` covering `join` after round
0, `initialize` bounds checking, defaulter flagging, and recipient
recomputation.

Each maps to a threat in `audit/threat-model.md` that otherwise had no test
attached. The brief asks for roughly 15 contributor issues overall; the
repository has 22.

## 10. `test.rs` ships a test harness

**Brief:** "one compiling placeholder test".
**Built:** one placeholder test plus a `Harness` struct that deploys a mock
SEP-41 token and a vault, funds members, and advances the ledger clock.

The harness is annotated `#[allow(dead_code)]` because most of it is unused
until someone claims an issue. That is deliberate: a contributor claiming
issue #2 should write assertions, not setup. Without it, the first four
issues each involve writing the same forty lines.

## 11. The scaffold brief itself is gitignored

`AJO_ONCHAIN_SCAFFOLD_BRIEF.md` is in the working tree but excluded from git.

It is internal planning material — it discusses grant strategy and the
community vote — and this repository is public-facing. Remove the entry from
`.gitignore` if you want it committed.

## 12. `scripts/create-issues.sh`

**Brief:** §3 lists two files in `scripts/` — `deploy-testnet.sh` and
`gen-bindings.sh`. §9 asks for `docs/ISSUES.md` "ready to bulk-create" but
does not say how.

**Built:** a third script that opens all 23 issues from `docs/ISSUES.md`
through `gh`, in order, with labels.

The numbers in `docs/ISSUES.md` are load-bearing: they are written into the
source as `TODO(#n)` markers, and GitHub assigns issue numbers from a counter
that pull requests also consume. Creating the issues by hand, or in the wrong
order, or after opening a single pull request, silently points every marker at
the wrong issue. The script checks the tracker is empty before it starts and
verifies each returned number against the expected one, stopping on the first
mismatch rather than producing 22 wrong links.

Run `scripts/create-issues.sh --dry-run` first; a dry run needs neither `gh`
nor a remote, so the parsed titles and labels can be checked while the
repository is still local.

---

## Not deviations, but worth recording

**Verification gate 1 on Windows.** `cargo test --all` fails to link on the
`x86_64-pc-windows-gnu` toolchain with `export ordinal too large`. This is a
mingw limitation on `cdylib` crates, not a repository problem:
`cargo test --all --lib` passes, `stellar contract build` emits all four
wasm artifacts, and CI runs on Linux. Documented in
[`SETUP.md`](./SETUP.md#windows-cargo-test-fails-to-link-with-export-ordinal-too-large).

**`CARGO_TARGET_DIR` on Windows.** Cargo could not create `contracts/target`
inside a OneDrive-synced folder. Also documented in `SETUP.md`. No repository
change was needed — both shell scripts already honour `CARGO_TARGET_DIR`.

**Dependency versions.** `@stellar/stellar-sdk` is pinned to `^17.0.1` and
`@supabase/supabase-js` to `^2.116.0`, the current releases at scaffold time.
