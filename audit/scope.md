# Audit scope

Prepared for a [Soroban Audit Bank](https://stellar.org/foundation/audit-bank)
engagement, which is available at no cost to SCF-awarded projects.

This document is written to be handed to an auditor as-is. It says what we
want examined, what we do not, and what we are assuming — because an audit
whose assumptions were never stated is an audit whose conclusions cannot be
relied on.

| Field | Value |
|---|---|
| Audit commit hash | `TBD` — frozen when the engagement is booked |
| Network | Stellar testnet at audit time; mainnet is Tranche 4 |
| Soroban SDK | 27.x (see `contracts/Cargo.toml`) |
| Language | Rust, `no_std`, `wasm32v1-none` |
| Prior audits | None. This is the first. |

## In scope

| Crate | Why it is in scope |
|---|---|
| `contracts/escrow-vault` | **The only contract that holds funds.** It is the entire trust model: custody, the round state machine, defaulter flagging and payout. Every one of the three invariants is enforced here. If exactly one crate can be audited, it is this one. |
| `contracts/circle-factory` | Deploys every vault and controls which WASM they run. It holds no funds, but a compromise here means every future circle runs attacker-chosen code — see threat T4, which we believe is currently unmitigated and want an opinion on. |
| `contracts/shared` | Types, error codes and the three bitmap helpers. In scope because `cycle_complete` and the bit helpers are where an off-by-one becomes a double payout, and because `CircleConfig`/`CircleState` define the stored encoding. |

Specific questions we would like answered, beyond a general review:

1. **T4 — should the vault WASM hash be immutable after `initialize`?**
   Mutability lets us ship a fix to a vault bug; immutability removes a
   trusted party. We have not been able to convince ourselves either way and
   would value an auditor's view. See `threat-model.md`.
2. **`settle_and_pay` ordering.** We know the transfers precede the state
   write and we have filed it ourselves as `known-issues.md` #2. We would
   like confirmation of whether it is exploitable with a SEP-41-conformant
   token, or only with a malicious one.
3. **Permissionless `advance_phase` and `settle_and_pay`.** These take no
   auth by design, so that no member can freeze a circle by refusing to act.
   Is there an economic or griefing attack on that openness that we have
   missed?
4. **Bitmap indexing.** Member index is position in `CircleState.members`.
   `join` is rejected once `round > 0` to keep indices stable. Is there any
   other path that could reorder or shorten that vector?

## Out of scope

| Component | Reason |
|---|---|
| `apps/indexer` | Read-only by construction. Holds no keys, signs nothing, has no path to move funds. If it is compromised the dashboard shows wrong numbers; no circle is affected. |
| `services/scoring` | Advisory, and not yet written. Its only planned write path is a signed attestation stored under a separate key and labelled as model output. |
| `apps/web` | Client-side only, and not yet scaffolded. **The contract must be safe against a fully malicious frontend** — please assume the UI is hostile when reviewing the contracts. Any finding of the form "the frontend prevents this" is a finding, not a mitigation. |
| `contracts/bid-auction` | Tranche 2. Module documentation only, no logic. Its blocking design question is `known-issues.md` #1. |
| `contracts/reputation-registry` | Tranche 3. Module documentation only, no logic. |
| The SEP-24 anchor | A third party outside our trust boundary. See assumption 4. |

## Assumptions

These are the load-bearing assumptions. If an auditor disagrees with one, we
would rather hear that than have the review proceed on a premise we did not
share.

1. **The settlement token is a trusted, standard SEP-41 contract.** We expect
   USDC issued by Circle, or the native Stellar asset wrapper. We assume it
   does not reenter, does not lie about transfer success, and does not have a
   transfer hook. We are aware this assumption is doing real work in
   `settle_and_pay` — see `known-issues.md` #2 — and we would like to know
   what breaks if it is false.

2. **`member_cap <= 32`, enforced at `initialize`.** All three bitmaps are
   `u32`, one bit per member, indexed by position in `members`. The cap is
   rejected at initialisation rather than clamped, and `ajo_shared::types`
   guards the shift-overflow case at 32. Real Ajo circles are 5 to 20 people;
   the limit is accepted rather than engineered around
   (`known-issues.md` #4).

3. **Ledger timestamps are monotonic and not attacker-controlled beyond
   validator drift.** Deadlines use `env.ledger().timestamp()`. We assume an
   attacker cannot move it backwards, nor forwards by more than the drift the
   protocol permits. Every deadline in the system is measured in days, so
   drift measured in seconds is not material — but we assume, rather than
   verify, that this is the right model.

4. **The anchor's SEP-24 flow is outside the trust boundary.** Naira-to-USDC
   conversion is performed by a licensed third-party anchor. A dishonest or
   failed anchor can fail to deliver naira to a member, and that is a real
   product risk we carry. It cannot touch a circle's pot: by the time funds
   reach the vault they are already USDC on-ledger, and the vault has no
   dependency on the anchor.

## What we have not done

Stated plainly, so nobody has to infer it:

- No formal verification.
- No fuzzing campaign. `test.rs` ships with one placeholder test and a
  numbered TODO list of the tests that should exist, including a property
  test for the rotation invariant (`docs/ISSUES.md`, issue #6).
- No mainnet deployment, and therefore no adversarial exposure.
- No economic modelling of the Tranche 2 auction. It is not implemented, and
  its central design question is unresolved.
