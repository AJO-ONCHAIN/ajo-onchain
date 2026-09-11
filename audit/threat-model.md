# Threat model

What we think can go wrong, what stops it, and where we are not yet
protected. Threats marked **unmitigated** are unmitigated in the code as
shipped — they are listed here rather than left for an auditor to find.

## The three invariants

Everything below is in service of these. They are stated identically in
`contracts/escrow-vault/src/lib.rs` and in `CONTRIBUTING.md`, and a pull
request that weakens one is rejected.

1. **No privileged withdrawal.** There is no admin withdrawal function
   anywhere in `EscrowVault`. Funds leave the contract only through
   `settle_and_pay`, and only to a member of that circle. No `pause`, no
   `set_admin`, no `emergency_withdraw`, no upgrade hook that could introduce
   one.
2. **No phase skipping.** Phases advance exactly one step at a time via
   `advance_phase`. No entry point accepts a target phase from the caller.
3. **One win per cycle.** A member may receive at most one payout before
   every member has received one, enforced by a bitmap in contract state.

## Assets

| Asset | Where it lives | What loss looks like |
|---|---|---|
| Circle pot | `EscrowVault` instance balance | Members lose the money they paid in. Unrecoverable. |
| Payout ordering | `winners_bitmap` in `CircleState` | Someone is paid twice and someone is never paid. Financially identical to theft. |
| Member indices | `CircleState.members` | Contribution and default history silently attach to the wrong person. |
| Default history | `default_bitmap`, later `ReputationRegistry` | A member is wrongly recorded as a defaulter, permanently, in a public place. |
| Vault WASM hash | `CircleFactory` instance storage | Every future circle runs attacker-chosen code. |
| Member phone numbers | Supabase, for reminders | Personal data breach. Never on-chain, never in an event, never in a log. |

## Actors

| Actor | Capabilities | Trusted? |
|---|---|---|
| Circle member | Calls `join`, `contribute`; may call the permissionless entry points | **No** |
| Circle founder | A member who called `initialize`. Sets terms once; has no ongoing privilege whatsoever | **No** |
| Any external caller | Can call `advance_phase` and `settle_and_pay` on any circle | **No** |
| Factory admin | Can change the vault WASM hash for *future* deployments | **No** — and this is threat T4 |
| Indexer | Reads events, writes the Supabase cache | **No** — holds no keys, cannot affect funds |
| Scoring service | Reads history, will publish signed attestations | **No** — advisory only, no write path to funds |
| SEP-24 anchor | Converts naira to USDC off-ledger | **No** — outside the trust boundary entirely |
| Settlement token contract | Moves value on `transfer` | **Yes** — the one trusted component. See `scope.md` assumption 1. |
| AJO developers | Can publish new code; cannot touch a deployed vault | **No** |

Nearly every row is "No". That is the point of the project: the only trusted
component is the token contract, and we chose a standard one specifically so
that trust is placed somewhere already widely audited.

---

## T1 — Double payout in one cycle

**Threat.** A member receives the pot twice before some other member has
received it once. Because every member has already paid in for both rounds,
this is theft from whoever is left unpaid at the end of the cycle.

**Mitigation.** `winners_bitmap` in `CircleState`. `settle_and_pay` checks
`has_won(index)` and rejects with `AlreadyWonThisCycle`. For rotational
circles the recipient is not taken from the caller at all: the contract
recomputes the lowest-indexed member who has not yet won, and rejects any
other address with `Unauthorized`. The bitmap is cleared only when
`cycle_complete` reports that every member has been paid.

**Status.** Mitigated in code. **Not yet covered by a test** — this is
`docs/ISSUES.md` issues #4 and #6, and until #6 exists a refactor could
remove the check without anything failing.

## T2 — Phase skipping

**Threat.** A caller jumps a circle straight to `Payout` and takes the pot
before contributions are complete, or replays `Payout` to drain it.

**Mitigation.** `advance_phase` takes no arguments — there is no syntax in
the interface for expressing a target phase, so no caller can supply one. It
moves exactly one step through `next_phase`, a total function over
`(Phase, Variant)`. `Payout` is terminal for `advance_phase`; only
`settle_and_pay` leaves it, and that sets the phase back to `Contributing`
and increments the round in the same write. Advancing also requires
`ledger().timestamp() >= deadline`.

**Status.** Mitigated in code. Test is `docs/ISSUES.md` issue #3.

## T3 — Stalled circle

**Threat.** A circle stops advancing and everyone's funds are stuck. This is
the failure mode a naive design walks straight into: if advancing the round
required the founder's or the recipient's signature, then any member who
stops responding — loses their phone, travels, or simply decides to be
difficult — freezes every other member's money indefinitely.

**Mitigation.** `advance_phase` and `settle_and_pay` are **permissionless**.
Anyone at all may poke a circle whose deadline has passed: another member, a
bot we run, a stranger. Neither function lets the caller choose who benefits,
so opening them up costs nothing. A member who refuses to participate is
flagged as a defaulter and the circle continues without them.

**Status.** Mitigated by design. We would like an auditor's view on whether
this openness creates a griefing or economic attack we have not thought of —
see `scope.md` question 3.

## T4 — Malicious vault WASM set by the factory admin

**Threat.** The factory admin's key is compromised, or the admin turns
malicious, and `set_vault_wasm` is pointed at a contract that drains every
circle deployed after that moment. A member joining a fresh circle has no
practical way to notice.

**Scope of the damage.** Already-deployed vaults are unaffected: Soroban does
not rewrite a deployed contract because the factory changed a stored hash.
Existing members' funds are safe. Every circle created afterwards is not.

**Mitigation.** **Currently unmitigated.**

**Open question for auditors.** Should the WASM hash be immutable after
`initialize`?

- Immutable removes the trusted party entirely, which is consistent with the
  rest of the design. It also removes any ability to ship a fix for a vault
  bug to new circles; the only route would be deploying a new factory and
  persuading everyone to migrate.
- Mutable keeps that escape hatch and keeps a permanent trusted admin.

A middle path exists — a timelock on hash changes, or a published hash
allowlist members can check — and we have not evaluated either properly. We
have deliberately not decided this unilaterally. See `scope.md` question 1.

## T5 — Reentrancy via a malicious token contract

**Threat.** The settlement token calls back into the vault during a
`transfer`, re-entering `contribute` or `settle_and_pay` before the first
call has finished writing state.

**Mitigation, partial.**

- `contribute` follows checks-effects-interactions correctly. It sets the
  contributed bit, adds to the pot, and **writes state to storage before**
  calling `transfer`. A reentrant call sees the updated bitmap and is
  rejected with `AlreadyContributed`.
- `settle_and_pay` **does not.** It performs both transfers and only then
  writes the winner bit, clears the pot and increments the round. A reentrant
  call during the payout transfer would observe the pre-payout state.

We believe this is not exploitable with a SEP-41-conformant token, because
such a token does not call back. But "safe given a trusted dependency" is a
weaker property than "safe regardless", and this is the contract that holds
the money.

**Status.** Filed by us as `known-issues.md` #2 (medium, open). To be
reordered to checks-effects-interactions before audit. We would like
confirmation of exploitability either way — `scope.md` question 2.

## T6 — Bid front-running

**Threat.** In a bid-based circle, a member watches others' bids and
undercuts by the smallest possible margin, winning every auction without ever
bidding competitively.

**Mitigation.** Sealed-bid commit/reveal. During `Bidding` only
`sha256(discount || nonce || member || round)` is submitted; the values
appear during `Revealing`, by which point the bidding window is closed. The
nonce prevents brute-forcing a small preimage space; `member` prevents
copying someone else's commitment; `round` prevents replay.

**Status.** Tranche 2, **not implemented**. Design is documented in
`contracts/bid-auction/src/lib.rs`. Two blocking open questions remain,
including what happens to a member who commits and never reveals.

## T7 — Join-after-start shifting member indices

**Threat.** A new member joins mid-cycle. Every bitmap is indexed by position
in `CircleState.members`, so if the vector were ever reordered — or if
indices were reassigned — one member's contribution and payout history would
silently become another's. Nothing would fail; the numbers would just be
wrong, and wrong in a way that decides who gets paid.

**Mitigation.** `join` is rejected with `WrongPhase` once `round > 0`.
Members can only ever be appended, never inserted or removed, so an index
that has been assigned is never reused or moved. There is no `leave`
function, and a defaulting member keeps their index and their bit.

**Status.** Mitigated in code. Test is `docs/ISSUES.md` issue #7.

---

## Threats we are carrying knowingly

Not every risk here is a contract bug, and pretending otherwise would be
dishonest:

- **Win-then-default** (`known-issues.md` #1, high). A bid winner takes the
  pot and stops contributing. The remaining members are short and the money
  is gone. This blocks Tranche 2.
- **Naira devaluation** (`known-issues.md` #3, product). Members save in
  naira, the pot is denominated in USDC, and the two move apart. No contract
  change fixes this.
- **Social recovery of a lost key.** A member who loses their key loses their
  place in the circle. Self-custody's cost, and we have not solved it.
