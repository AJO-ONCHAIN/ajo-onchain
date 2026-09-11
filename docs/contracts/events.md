# Contract events

> ## Renaming a topic is a breaking change
>
> `apps/indexer/src/handlers/index.ts` branches on these topic names, and
> `contracts/shared/src/events.rs` is the source of truth for them.
>
> If you rename a topic on one side only, nothing fails. The contract keeps
> emitting, the indexer's `default` branch logs an unknown topic, and the
> dashboard quietly stops updating — which someone notices days later when
> the numbers look wrong.
>
> **Both sides must move in the same pull request.** The pull request
> template has a checkbox for this.

The same applies to field names. The `#[contractevent]` macro publishes
non-topic fields as a map keyed by the Rust struct's field name, so renaming
`amount` to `value` breaks the indexer exactly as thoroughly as renaming the
topic does.

Topics must fit `symbol_short!` — at most 9 characters. That is why the topic
is `cycdone` and not `cycle_done`.

## Topic table

| Topic | Emitter | Emitted when | Data fields | Indexer action |
|---|---|---|---|---|
| `created` | `EscrowVault::initialize` | A circle is created | `founder: Address`, `token: Address`, `contribution: i128` | Upsert a `circles` row keyed by the emitting contract address. `TODO(#14)` |
| `joined` | `EscrowVault::initialize`, `EscrowVault::join` | A member is seated, including the founder at index 0 | `member: Address`, `index: u32` | Upsert a `circle_members` row keyed by `(circle, index)`. `TODO(#15)` |
| `contrib` | `EscrowVault::contribute` | A member pays in | `member: Address`, `amount: i128`, `round: u32` | Record the contribution, update the circle's pot. Key on `(circle, member, round)`. `TODO(#16)` |
| `phase` | `EscrowVault::advance_phase` | The round advances one phase | `round: u32`, `phase_index: u32` | Update phase and deadline. `TODO(#17)` |
| `payout` | `EscrowVault::settle_and_pay` | The pot is paid out | `recipient: Address`, `net: i128`, `fee: i128`, `round: u32` | Record the payout, mark the recipient as having won. `TODO(#18)` |
| `default` | `EscrowVault::advance_phase` | A member missed the contribution deadline | `member: Address`, `round: u32` | Flag the member. **Never unflag.** `TODO(#19)` |
| `cycdone` | `EscrowVault::settle_and_pay` | Every member has been paid once | `round: u32` | Close the cycle, reset per-cycle aggregates. `TODO(#20)` |
| `deployed` | `CircleFactory::deploy_circle` | A new vault is deployed | `vault: Address`, `founder: Address` | Add the vault to the watch list. `TODO(#21)` |

## `phase_index` values

`phase` publishes a number, not a name.

| Value | Phase |
|---|---|
| 0 | `Contributing` |
| 1 | `Bidding` |
| 2 | `Revealing` |
| 3 | `Settling` |
| 4 | `Payout` |

The mapping is written out explicitly in `phase_index` in
`contracts/escrow-vault/src/lib.rs` rather than derived from the enum's
discriminant. That is deliberate: reordering the `Phase` enum is an ordinary
refactor, and it must not be able to change the meaning of events already
written to the ledger. Do not "improve" this by publishing the enum name.

## Ordering guarantees

- Events within one transaction are emitted in the order the contract makes
  them. `initialize` emits `created` then `joined`.
- Events across transactions follow ledger order.
- **A round produces `contrib` events in arbitrary order** — members pay
  whenever they pay. Do not infer member index from event order; use the
  `index` from `joined`.

## Handlers must be idempotent

The indexer advances its cursor only after a whole batch has been handled, so
a crash mid-batch replays that batch.

Every write must therefore be an upsert keyed by something stable — the
contract address plus round, or the event's ledger and index. A blind
`insert` produces duplicates, and `count = count + 1` produces wrong totals.
Neither will fail loudly.

## Events are not a payment path

The indexer reads these events and writes a cache. That is all it does. It
holds no keys and signs nothing, so a missed, duplicated, or malicious event
cannot move anyone's money — it can only make the dashboard wrong.

If you find yourself designing something where a contract event triggers a
transfer, stop: that is a trusted off-chain component, and this project does
not have one.
