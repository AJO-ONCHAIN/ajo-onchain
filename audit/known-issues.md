# Known issues

Filed by us, before any external review.

This file exists because a repository with no known issues has either been
examined and is perfect, or has never been examined. Reviewers can tell the
difference, and the second reads much worse than an honest list. Issue #2
below is a real ordering flaw that we found, wrote down, and left in the code
with a pointer to this file rather than quietly patching it — the fix should
arrive with the regression test that proves it.

| # | Issue | Severity | Threat | Tranche | Status |
|---|---|---|---|---|---|
| 1 | Win-then-default | High | T2 | 2 | **Open — blocking** |
| 2 | Transfer-before-write in `settle_and_pay` | Medium | T1 | 1 | Open |
| 3 | Naira devaluation vs USDC-denominated savings | Product | — | 1 | Open |
| 4 | 32-member cap | Low | — | 1 | **Accepted** |

---

## 1. Win-then-default

**Severity:** High · **Threat:** T2 · **Tranche:** 2 · **Status: open, and
blocking.**

### The problem

A member wins the sealed-bid auction, receives the pot, and then stops
contributing for the rest of the cycle.

Default flagging catches them — `advance_phase` sets their bit when they miss
the contribution deadline, and they are excluded from future *bidding*. But
that is a punishment applied after the fact to someone who already has the
money. The remaining members are short by exactly the amount the defaulter
would have paid for the rest of the cycle, and nothing in the contract gets
it back.

Rotational circles have a weaker version of the same problem, and it is worth
being clear that this is not a new risk we invented: a traditional Ajo circle
has it too. What makes the auction variant worse is that it lets a member
choose to be paid early. Someone who intends to default is exactly the person
most motivated to bid aggressively for the first pot. **The mechanism
selects for the attacker.**

### Options under consideration

- **Bond from bid winners.** The winner locks collateral covering their
  remaining contributions, released as they pay. Sound, and it prices out
  precisely the liquidity-constrained members the product exists to serve. If
  you could post the bond you might not need the early payout.
- **Reputation-weighted auction.** Bidding power scales with completed
  circles from `ReputationRegistry`. Helps eventually; does nothing for a new
  circle of new members, which is every circle at launch.
- **Group clawback vote.** Members vote to reassign a defaulter's future
  payout slot. Reintroduces governance, and therefore a majority that can
  coordinate against a minority — which is the social failure mode the
  contract was supposed to remove.

### Decision

**No option has been chosen, and implementation must not proceed past this
line.** Building the auction on an unresolved settlement rule wastes the
work: each option above implies a different contract shape, not a different
parameter.

This is why `contracts/bid-auction` is documentation and a `version()` stub.

---

## 2. Transfer-before-write in `settle_and_pay`

**Severity:** Medium · **Threat:** T1, T5 · **Tranche:** 1 · **Status:
open.**

### The problem

`EscrowVault::settle_and_pay` performs the token transfers — fee to the
collector, remainder to the recipient — and only afterwards writes the winner
bit, clears the pot, increments the round and saves state.

That is the wrong order. Checks-effects-interactions says the state write
comes first, so that any reentrant call during the external transfer sees the
post-payout world rather than the pre-payout one.

`contribute` in the same file does it correctly, which makes the
inconsistency easy to miss on a read: one function establishes the pattern
and the other quietly departs from it.

### Is it exploitable?

Probably not, with the tokens we intend to support. A SEP-41-conformant token
does not call back into its caller during `transfer`, so there is no
reentrant call to observe the stale state.

That defence depends entirely on the token being well-behaved. `scope.md`
assumption 1 states that dependency explicitly, and this issue is the place
where it is doing the most work. "Safe because our dependency behaves" is
weaker than "safe regardless", and this is the function that moves the money.

### Fix

Reorder to checks-effects-interactions: compute `fee` and `net`, write the
full post-payout state to storage, then transfer. The `pot` value is already
captured in locals before the transfers, so the reorder is mechanical.

**Before the audit.** The code comment at the call site points here, so
whoever touches that function finds this first. It ships with a regression
test that fails against the current ordering.

---

## 3. Naira devaluation versus USDC-denominated savings

**Severity:** Product · **Tranche:** 1 · **Status: open. Needs a product
answer, not a contract one.**

### The problem

Members earn, think, and spend in naira. Contributions are denominated in
USDC, because that is what exists on-ledger and what the SEP-24 anchor
bridges to.

When the naira falls against the dollar — which it has, repeatedly and
sharply — a member's contribution costs more naira each round than it did at
the start. The pot they eventually receive is worth more naira than they
expected, but they have to survive the intervening months of rising
contributions to get there, and a member on a fixed naira income may simply
be unable to. **A currency move can cause a default in a system that is
working exactly as designed.**

The reverse case is milder but real: if the naira strengthens, members who
have already been paid have done better than those still waiting.

### Why this is not a contract bug

`EscrowVault` moves a fixed number of token units per round. It is doing
precisely what it was told. No change to the state machine addresses this.

### Options, none chosen

- **Naira-denominated contributions**, converted at contribution time. Moves
  the FX risk to whoever holds the float, and the contract would need an
  oracle — a new trusted component, which is a significant cost to the trust
  model.
- **Show naira prominently, USDC secondarily,** and state the risk plainly at
  signup. Honest, cheap, and does not actually reduce the risk.
- **Shorter cycles.** Less FX exposure per circle. Also smaller pots, which
  makes the product less useful for the lump-sum purchases people run Ajo
  circles for in the first place.
- **A naira-backed stablecoin,** if a credible one becomes available on
  Stellar with real liquidity. Not currently something we can rely on.

Recorded here so that nobody discovers it during the pilot and treats it as a
surprise. It is not a surprise; it is a known, unresolved product risk.

---

## 4. 32-member cap

**Severity:** Low · **Tranche:** 1 · **Status: accepted. Will not be
fixed.**

`CircleState` tracks membership with three `u32` bitmaps — `winners_bitmap`,
`contributed_bitmap`, `default_bitmap` — one bit per member, indexed by
position in `members`. That caps a circle at 32 people.
`EscrowVault::initialize` rejects a `member_cap` above 32 rather than
clamping it, and `ajo_shared::types` guards the shift-overflow case at the
boundary.

**Why this is accepted rather than fixed.** Real Ajo circles are 5 to 20
people. The size is not an implementation detail — it is bounded by how many
people can hold each other socially accountable, which is what makes the
model work at all. A 100-member circle is not a bigger Ajo circle; it is a
different, worse product with no social enforcement.

Widening to `u64` or a `Vec<bool>` would cost storage and complexity on every
single operation, to support circles we do not believe should exist.

**If this ever needs to change,** the honest approach is a new bitmap type in
`ajo-shared` with a storage migration, not a silent widening. Every index in
the system refers to a bit position, and changing the width changes the
stored encoding of every live circle.
