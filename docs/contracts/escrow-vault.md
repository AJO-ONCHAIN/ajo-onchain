# `EscrowVault`

One instance per circle. Holds the funds and runs the round state machine.

Source: [`contracts/escrow-vault/src/lib.rs`](../../contracts/escrow-vault/src/lib.rs)

## Entry points

| Function | Auth | Phase required | Behaviour |
|---|---|---|---|
| `initialize(config, founder)` | `founder` | — | Callable once. Rejects `member_cap == 0` or `> 32`, `contribution <= 0`, and `fee_bps > 10000`. Founder becomes member 0. Sets the first deadline. |
| `join(who)` | `who` | `round == 0` only | Appends to `members`. Rejects mid-cycle joins. Rejects duplicates and a full circle. |
| `contribute(who)` | `who` | `Contributing` | Transfers `config.contribution` in, sets the contributed bit, adds to `pot`. Rejects a second contribution in one round. |
| `advance_phase()` | **none** | any | Moves exactly one phase forward, only once `ledger().timestamp() >= deadline`. Resets the deadline. Flags defaulters when leaving `Contributing`. |
| `settle_and_pay(recipient)` | **none** | `Payout` | Pays `fee_bps` of the pot to `fee_collector`, remainder to `recipient`. Marks the winner, clears the pot and contributed bitmap, increments the round, returns to `Contributing`. Emits `cycdone` when the cycle closes. |
| `get_config()` | none | any | View. |
| `get_state()` | none | any | View. |

### What is not in this table

There is no `withdraw`, no `pause`, no `set_admin`, no `emergency_withdraw`,
no `upgrade`, and no setter for any field of `CircleConfig`. That absence is
invariant 1 and it is the entire product. The table above is the complete
interface — check it against the source.

### Why two functions take no auth

`advance_phase` and `settle_and_pay` are callable by anyone.

This looks wrong at first glance and is deliberate. If advancing a round
required a particular member's signature, that member could freeze every
other member's funds by doing nothing at all — losing a phone is enough. So
anyone may poke a circle whose deadline has passed.

Opening them costs nothing because neither lets the caller choose who
benefits. `advance_phase` takes no arguments. `settle_and_pay` takes a
recipient, but for a rotational circle the contract recomputes who is owed
the pot and rejects any other address with `Unauthorized` — the argument is
a convenience for the caller, never a grant of authority.

This is threat T3 in [`audit/threat-model.md`](../../audit/threat-model.md).

## Storage layout

Instance storage, two keys.

| Key | Type | Mutability |
|---|---|---|
| `DataKey::Config` | `CircleConfig` | Written once at `initialize`. Never updated. |
| `DataKey::State` | `CircleState` | Rewritten in place on every state change. |

There is no per-member storage entry. The member list and all three bitmaps
live inside `CircleState`, so a round costs a fixed number of ledger entries
regardless of how many people are in the circle — which matters, because the
member paying that cost is on a metered connection in Lagos.

### `CircleConfig` — immutable

| Field | Type | Notes |
|---|---|---|
| `token` | `Address` | SEP-41 settlement token. Trusted; see `audit/scope.md` assumption 1. |
| `member_cap` | `u32` | `1..=32`. See known-issues #4. |
| `contribution` | `i128` | Per member, per round, in the token's smallest unit. |
| `interval` | `u64` | Seconds between phase deadlines. |
| `variant` | `Variant` | `Rotational` or `BidBased`. |
| `fee_bps` | `u32` | Protocol fee in basis points. |
| `fee_collector` | `Address` | Where the fee goes. **No setter exists.** |

Every field is fixed for the life of the circle. A founder who could raise
the contribution or redirect the fee after members had committed funds would
be a trusted party, and the point is that there isn't one.

### `CircleState` — mutable

| Field | Type | Notes |
|---|---|---|
| `members` | `Vec<Address>` | Append-only. Position is the bitmap index. |
| `round` | `u32` | Completed rounds. Round 0 is the joining window. |
| `phase` | `Phase` | Current phase. |
| `deadline` | `u64` | Ledger timestamp at which the phase may advance. |
| `winners_bitmap` | `u32` | Paid this cycle. Cleared when the cycle completes. Invariant 3. |
| `contributed_bitmap` | `u32` | Paid in this round. Cleared at each payout. |
| `default_bitmap` | `u32` | Missed a deadline. **Never cleared.** |
| `pot` | `i128` | Contributions held for the current round. |

## Why `join` locks after round 0

Every bitmap is a `u32` indexed by a member's position in `members`. Bit 3 of
`winners_bitmap` means "the member at index 3 has been paid".

That only works while indices are stable. Appending is safe: a new member
takes the next free index and every existing bit still refers to the same
person. But any operation that reordered or removed an entry would silently
reassign history — one member's contribution record and payout status would
become another's, and nothing would fail. The numbers would just quietly
decide the wrong thing about who gets paid.

So `join` is rejected with `WrongPhase` once `round > 0`, there is no `leave`
function, and a member who defaults keeps their index and their bit.

The cost is real: a circle cannot replace a defaulter mid-cycle. Governance
to do that safely is Tranche 3 work, and it will need to preserve indices —
most likely by substituting the address at a fixed index rather than removing
anything.

This is threat T7.

## Fee model

```
fee = pot * fee_bps / 10_000
net = pot - fee
```

The fee goes to `config.fee_collector`, the remainder to the recipient. Both
are transferred in `settle_and_pay` and nowhere else.

`fee_bps` is validated at `initialize` to be at most `10_000` (100%). Without
that check a misconfigured circle would compute a negative `net`, and the
payout transfer would panic mid-settlement, leaving the pot stranded in a
vault with no way to release it.

Integer division truncates, so the fee rounds down and the recipient
receives the remainder. The dust favours the member rather than the
protocol, which is the right direction for a rounding error to go.

## Events

See [`events.md`](./events.md). Every state change emits one; the indexer
depends on them, and nothing else does.

## Known issues affecting this contract

- [`known-issues.md` #2](../../audit/known-issues.md) — `settle_and_pay`
  transfers before writing state. Medium, open, to be fixed before audit.
- [`known-issues.md` #4](../../audit/known-issues.md) — the 32-member cap.
  Low, accepted, will not be fixed.
