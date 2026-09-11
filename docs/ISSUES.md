# Contributor issues

Every `TODO(#n)` in the repository, written up ready to open as a GitHub
issue. Find them in the source with:

```bash
grep -rn "TODO(#" --include="*.rs" --include="*.ts" .
```

**When you resolve one, delete both the marker and its entry here, in the
same pull request.** A TODO with no issue and an issue with no TODO are
equally confusing to the next person.

Numbering starts at #2: issue #1 is reserved for the repository tracking
issue. The numbers are stable — they are written into the source — so please
do not renumber.

| # | Title | Area | Label | Effort |
|---|---|---|---|---|
| 2 | A member cannot contribute twice in one round | contracts | good first issue | 1–2h |
| 3 | `advance_phase` is rejected before the deadline | contracts | good first issue | 1–2h |
| 4 | `settle_and_pay` rejects a member who already won | contracts | help wanted | half a day |
| 5 | Fee and remainder land in the right places | contracts | good first issue | half a day |
| 6 | Property test: every member wins exactly once per cycle | contracts | help wanted | 1–2 days |
| 7 | `join` is rejected once `round > 0` | contracts | good first issue | 1–2h |
| 8 | `initialize` rejects out-of-range `member_cap` | contracts | good first issue | 1–2h |
| 9 | `advance_phase` flags a member who missed the deadline | contracts | help wanted | half a day |
| 10 | `settle_and_pay` pays the recomputed recipient | contracts | help wanted | half a day |
| 11 | Implement `pollEvents` against Soroban RPC | indexer | help wanted | 1–2 days |
| 12 | Deploy and initialise a vault atomically | contracts | help wanted | 1–2 days |
| 13 | Persist the indexer cursor in Supabase | indexer | help wanted | half a day |
| 14 | Handle `created`: upsert the circle | indexer | good first issue | 1–2h |
| 15 | Handle `joined`: upsert the member | indexer | good first issue | 1–2h |
| 16 | Handle `contrib`: record the contribution | indexer | help wanted | half a day |
| 17 | Handle `phase`: update phase and deadline | indexer | help wanted | half a day |
| 18 | Handle `payout`: record the payout | indexer | help wanted | half a day |
| 19 | Handle `default`: flag the member | indexer | good first issue | 1–2h |
| 20 | Handle `cycdone`: close the cycle | indexer | help wanted | half a day |
| 21 | Handle `deployed`: watch the new vault | indexer | good first issue | 1–2h |
| 22 | Reminder worker: scan for approaching deadlines | indexer | help wanted | 1–2 days |
| 23 | Reminder worker: send WhatsApp and SMS messages | indexer | help wanted | 1–2 days |

Nine `good first issue`, thirteen `help wanted`.

---

## #2 — A member cannot contribute twice in one round

**Label:** `good first issue` · **Skills:** Rust basics · **Effort:** 1–2h

**What needs doing.** `EscrowVault::contribute` sets a bit in
`contributed_bitmap` and rejects a member whose bit is already set. The check
exists; nothing proves it works, so a refactor could remove it and every test
would still pass. Write the test that would catch that.

**Where.** `contracts/escrow-vault/src/test.rs`, marker
`TODO(good-first-issue #2)`.

**Definition of done.** A member contributes successfully, then a second
`contribute` in the same round fails with `Error::AlreadyContributed`. The
pot increased by exactly one contribution. Deleting the `has_contributed`
check in `lib.rs` makes the test fail. `task check` passes.

**Getting started.** `Harness::new(0)` gives you a deployed vault and token;
`Harness::add_member` returns a funded member. Use
`vault.try_contribute(&who)` to assert on the error rather than panicking.

## #3 — `advance_phase` is rejected before the deadline

**Label:** `good first issue` · **Skills:** Rust basics · **Effort:** 1–2h

**What needs doing.** `advance_phase` is permissionless — anyone may call it
— and the only thing stopping someone rushing a circle through its phases is
the deadline check. Prove the check holds.

**Where.** `contracts/escrow-vault/src/test.rs`, marker
`TODO(good-first-issue #3)`.

**Definition of done.** `advance_phase` before the deadline fails with
`Error::DeadlineNotReached` and the phase is unchanged. After
`Harness::pass_deadline` it succeeds and the phase moves exactly one step.
`task check` passes.

## #4 — `settle_and_pay` rejects a member who already won

**Label:** `help wanted` · **Skills:** Rust, Soroban tests · **Effort:** half
a day

**What needs doing.** This is invariant 3, and threat T1: a member who is
paid twice in one cycle is stealing from whoever is left unpaid at the end.
`winners_bitmap` enforces it. No test covers it.

**Where.** `contracts/escrow-vault/src/test.rs`, marker `TODO(#4)`.

**Definition of done.** Drive a circle through a full round so member 0 is
paid. Get back to `Payout` and show that `settle_and_pay(member_0)` fails —
it should be rejected as `Unauthorized`, because the contract recomputes the
expected recipient and member 0 is no longer it. Then construct the case that
reaches the `has_won` check itself and assert `AlreadyWonThisCycle`. Say in a
comment which guard each assertion is exercising.

## #5 — Fee and remainder land in the right places

**Label:** `good first issue` · **Skills:** Rust basics · **Effort:** half a
day

**What needs doing.** `settle_and_pay` sends `fee_bps` of the pot to
`fee_collector` and the rest to the recipient. Prove both amounts, not just
that a payout happened.

**Where.** `contracts/escrow-vault/src/test.rs`, marker `TODO(#5)`.

**Definition of done.** With a non-zero `fee_bps`, assert the recipient's and
the collector's token balances afterwards. Include a case where the division
truncates and confirm the dust goes to the recipient, not the collector —
that rounding direction is deliberate and should not be able to flip
silently. `task check` passes.

## #6 — Property test: every member wins exactly once per cycle

**Label:** `help wanted` · **Skills:** Rust, property testing · **Effort:**
1–2 days

**What needs doing.** The strongest statement of invariant 3: run a circle of
N members for N rounds and every member has been paid exactly once, in index
order, with the bitmap cleared at the end.

This is the test that makes the other rotation tests hard to break by
accident, because it holds for every N rather than for the one N someone
picked.

**Where.** `contracts/escrow-vault/src/test.rs`, marker `TODO(#6)`.

**Definition of done.** Parameterised over circle size for at least N = 2, 3,
5, 10. Each round: everyone contributes, phases advance, the pot is settled.
Assert each member is paid exactly once, that payouts follow index order,
that `cycdone` is emitted on the final round, and that `winners_bitmap`
resets to 0. Consider `proptest` for the member count if you want to go
further.

## #7 — `join` is rejected once `round > 0`

**Label:** `good first issue` · **Skills:** Rust basics · **Effort:** 1–2h

**What needs doing.** Threat T7. Bitmaps are indexed by position in
`members`, so a mid-cycle join would reassign contribution and payout history
to the wrong person — silently, with nothing failing.

**Where.** `contracts/escrow-vault/src/test.rs`, marker
`TODO(good-first-issue #7)`.

**Definition of done.** Joining during round 0 succeeds and returns an
increasing index. After the first settlement, `join` fails with
`Error::WrongPhase`. Also cover the `AlreadyAMember` and `CircleFull` cases
while you are here. `task check` passes.

## #8 — `initialize` rejects out-of-range `member_cap`

**Label:** `good first issue` · **Skills:** Rust basics · **Effort:** 1–2h

**What needs doing.** All three bitmaps are `u32`, so a circle above 32
members would silently lose everyone past bit 31. `initialize` rejects that
rather than clamping. Cover the boundaries.

**Where.** `contracts/escrow-vault/src/test.rs`, marker
`TODO(good-first-issue #8)`.

**Definition of done.** `member_cap` of 0 and of 33 both fail with
`Error::InvalidAmount`; 1 and 32 both succeed. `contribution <= 0` fails.
`fee_bps` of 10001 fails, 10000 succeeds. `task check` passes.

## #9 — `advance_phase` flags a member who missed the deadline

**Label:** `help wanted` · **Skills:** Rust, Soroban tests · **Effort:** half
a day

**What needs doing.** Leaving `Contributing` is the moment a missed payment
becomes a default. The flag is permanent and will eventually feed
`ReputationRegistry`, so getting it wrong marks a real person as a defaulter
in a public place.

**Where.** `contracts/escrow-vault/src/test.rs`, marker `TODO(#9)`.

**Definition of done.** In a circle where one member does not contribute,
advancing past `Contributing` sets exactly that member's bit in
`default_bitmap` and emits one `default` event. Members who did contribute
are untouched. A member already flagged is not flagged again, and no
duplicate event is emitted. `task check` passes.

## #10 — `settle_and_pay` pays the recomputed recipient

**Label:** `help wanted` · **Skills:** Rust, Soroban tests · **Effort:** half
a day

**What needs doing.** The `recipient` argument is a convenience, never an
authority: the contract recomputes the lowest-indexed member who has not won
and rejects anything else. Prove that a caller cannot pay themselves out of
turn.

**Where.** `contracts/escrow-vault/src/test.rs`, marker `TODO(#10)`.

**Definition of done.** Passing the correct recipient succeeds. Passing a
different member, or a non-member address, fails with `Error::Unauthorized`
and moves no funds. Assert balances are unchanged after the failed call.
`task check` passes.

## #11 — Implement `pollEvents` against Soroban RPC

**Label:** `help wanted` · **Skills:** TypeScript, Stellar SDK · **Effort:**
1–2 days

**What needs doing.** The indexer's core read loop. Everything else in the
indexer is blocked on this.

**Where.** `apps/indexer/src/lib/stellar.ts`, marker `TODO(#11)`.

**Definition of done.** `pollEvents` calls `rpc.Server.getEvents` for the
factory and for every vault it has deployed; pages until the cursor stops
advancing; converts XDR topics and values with `scValToNative` into the
`ContractEvent` shape; and handles the RPC retention window explicitly — if
the stored cursor is older than the node's oldest ledger, fail loudly rather
than skipping the gap. Cold start polls the factory first, or the first
events from a new vault are missed.

**Note.** This stays read-only. No key, no signing, no transaction
construction — see the header comment in `src/index.ts`.

## #12 — Deploy and initialise a vault atomically

**Label:** `help wanted` · **Skills:** Rust, Soroban · **Effort:** 1–2 days

**What needs doing.** `CircleFactory::deploy_circle` deploys a vault but does
not initialise it, which leaves a window in which anyone can call
`initialize` on it first and set the terms. Pass the `CircleConfig` through
as constructor arguments so deployment and initialisation are one call.

**Where.** `contracts/circle-factory/src/lib.rs`, marker `TODO(#12)`.

**Definition of done.** `deploy_circle` takes a `CircleConfig` and passes it
to `deploy_v2` as constructor arguments. The deployed vault is initialised in
the same transaction. A test shows a second `initialize` on that vault fails
with `AlreadyInitialized`. This touches deployment, so tick the
security-sensitive box on your pull request and explain it.

## #13 — Persist the indexer cursor in Supabase

**Label:** `help wanted` · **Skills:** TypeScript, Supabase · **Effort:**
half a day

**What needs doing.** `loadCursor` returns `null` and `saveCursor` does
nothing, so every restart either replays all history or starts at "now" and
leaves a permanent hole in the cache that nothing ever fills.

**Where.** `apps/indexer/src/lib/stellar.ts`, marker `TODO(#13)`.

**Definition of done.** A single-row `indexer_state` table keyed by network.
`loadCursor` reads it, `saveCursor` upserts it. Restarting the indexer
resumes from where it stopped. Include the migration SQL.

## #14 — Handle `created`: upsert the circle

**Label:** `good first issue` · **Skills:** TypeScript, SQL basics ·
**Effort:** 1–2h

**What needs doing.** Turn a `created` event into a `circles` row. The
vault's contract address is the natural primary key.

**Where.** `apps/indexer/src/handlers/index.ts`, marker `TODO(#14)`.

**Definition of done.** Upsert — not insert — keyed on the contract address,
storing `founder`, `token` and `contribution`. Handling the same event twice
produces one row and no error; the indexer replays a batch after a crash, so
this is not hypothetical. Include the migration SQL. See
`docs/contracts/events.md` for the payload.

## #15 — Handle `joined`: upsert the member

**Label:** `good first issue` · **Skills:** TypeScript, SQL basics ·
**Effort:** 1–2h

**What needs doing.** Turn a `joined` event into a `circle_members` row.

**Where.** `apps/indexer/src/handlers/index.ts`, marker `TODO(#15)`.

**Definition of done.** Upsert keyed on `(circle, index)` — **not**
`(circle, member)`. The index is what every contract bitmap refers to, so it
is the value the rest of the cache has to agree with. Replaying is a no-op.
Include the migration SQL.

## #16 — Handle `contrib`: record the contribution

**Label:** `help wanted` · **Skills:** TypeScript, SQL · **Effort:** half a
day

**What needs doing.** Record each contribution and keep the circle's pot in
step.

**Where.** `apps/indexer/src/handlers/index.ts`, marker `TODO(#16)`.

**Definition of done.** Upsert keyed on `(circle, member, round)`. The pot is
recomputed by summing contributions for the round, **not** by
`pot = pot + amount` — a replayed batch would double-count an increment.
Include the migration SQL.

## #17 — Handle `phase`: update phase and deadline

**Label:** `help wanted` · **Skills:** TypeScript · **Effort:** half a day

**What needs doing.** Keep the cached phase and deadline in step with the
contract.

**Where.** `apps/indexer/src/handlers/index.ts`, marker `TODO(#17)`.

**Definition of done.** Map `phase_index` to a phase name using the table in
`docs/contracts/events.md`. Keep it a numeric mapping — the contract
publishes a number precisely so that reordering the Rust enum cannot change
the meaning of old events. An out-of-range index is logged, not silently
coerced. Ignore an event for a round older than the one already stored, so a
replay cannot move the circle backwards.

## #18 — Handle `payout`: record the payout

**Label:** `help wanted` · **Skills:** TypeScript, SQL · **Effort:** half a
day

**What needs doing.** Record who was paid, how much, and in which round. This
is the table Tranche 3's reputation work will read, so store the amounts and
the round, not just the fact of a payout.

**Where.** `apps/indexer/src/handlers/index.ts`, marker `TODO(#18)`.

**Definition of done.** Upsert keyed on `(circle, round)`. Store `recipient`,
`net` and `fee`. Mark the recipient as having won in the current cycle.
Include the migration SQL.

## #19 — Handle `default`: flag the member

**Label:** `good first issue` · **Skills:** TypeScript, SQL basics ·
**Effort:** 1–2h

**What needs doing.** Record that a member missed a contribution deadline.

**Where.** `apps/indexer/src/handlers/index.ts`, marker `TODO(#19)`.

**Definition of done.** Upsert keyed on `(circle, member, round)`. **Never
unflag.** The contract never clears `default_bitmap`, and the cache must not
either, however tempting it is to tidy up after a member who pays late — the
cache would then disagree with the ledger, and the ledger is right. Include
the migration SQL.

## #20 — Handle `cycdone`: close the cycle

**Label:** `help wanted` · **Skills:** TypeScript, SQL · **Effort:** half a
day

**What needs doing.** Every member has been paid once and the contract has
cleared its winners bitmap. The cache should record the completed cycle and
reset per-cycle aggregates.

**Where.** `apps/indexer/src/handlers/index.ts`, marker `TODO(#20)`.

**Definition of done.** The completed cycle is retained as history — it is
exactly the record that makes a member creditworthy — and per-cycle state is
reset for the new cycle. Idempotent on replay.

## #21 — Handle `deployed`: watch the new vault

**Label:** `good first issue` · **Skills:** TypeScript · **Effort:** 1–2h

**What needs doing.** The factory emits `deployed` when a new circle is
created. Until that vault is on the watch list, the indexer never polls it
and the circle is invisible.

**Where.** `apps/indexer/src/handlers/index.ts`, marker `TODO(#21)`.

**Definition of done.** The vault address is persisted and picked up by the
next poll without restarting the process. Idempotent. Pairs naturally with
#11.

## #22 — Reminder worker: scan for approaching deadlines

**Label:** `help wanted` · **Skills:** TypeScript, Supabase · **Effort:**
1–2 days

**What needs doing.** Find circles in `Contributing` whose deadline is within
24 hours and work out who has not paid.

A missed contribution is not a UX annoyance — it is a permanent on-chain
default that leaves the rest of the circle short, and most of them are
forgetfulness rather than inability to pay.

**Where.** `apps/indexer/src/lib/reminders.ts`, marker `TODO(#22)`.

**Definition of done.** An hourly scan queries circles with a deadline inside
24 hours, finds members whose contributed bit is not set, and calls
`sendReminder` for each. **Records that the reminder was sent** so the next
scan does not send it again — without this the worker messages the same
person twenty-four times, which is how a helpful reminder becomes a reason to
block the number.

**Note.** This belongs in the indexer, not in a Next.js route handler. The
file's header comment explains why at length; read it before moving anything.

## #23 — Reminder worker: send WhatsApp and SMS messages

**Label:** `help wanted` · **Skills:** TypeScript, WhatsApp Business API ·
**Effort:** 1–2 days

**What needs doing.** Actually deliver the message. WhatsApp first, SMS as
fallback.

**Where.** `apps/indexer/src/lib/reminders.ts`, marker `TODO(#23)`.

**Definition of done.** Messages send through a WhatsApp Business API
provider using an approved template, with SMS fallback when no WhatsApp
number is on file. Failures are retried with backoff and never crash the
poll loop. **Phone numbers never appear in a log line, in an event, or
on-chain.**

**Start the template approval early.** Business-initiated WhatsApp
conversations require a pre-approved message template, and approval is the
long pole here — not the code.
