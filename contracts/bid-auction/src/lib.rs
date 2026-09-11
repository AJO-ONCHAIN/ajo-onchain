#![no_std]
//! # BidAuction — sealed-bid discount auction for early access to the pot.
//!
//! **Status: Tranche 2. Not implemented. Do not implement it yet** — see the
//! blocking open questions at the bottom of this comment.
//!
//! This crate exists in the workspace on purpose. It shows where the work is
//! going without pretending it is done.
//!
//! ## The problem it solves
//!
//! A rotating savings circle pays members in a fixed order. The failure mode
//! that actually breaks circles in practice is that **members need the money
//! before their turn comes** — a medical bill, school fees, restocking
//! inventory. Today that gets resolved socially: begging the collector,
//! swapping slots, or defaulting and blowing up the circle.
//!
//! AJO Onchain prices it instead. A member bids a discount for early access
//! to the pot, and the discount is distributed as a dividend to the members
//! who wait. Waiting stops being a favour and becomes a yield.
//!
//! ## Mechanism
//!
//! ### Commit
//!
//! During `Phase::Bidding` a member submits only a hash:
//!
//! ```text
//! commitment = sha256(discount || nonce || member || round)
//! ```
//!
//! - `discount` — the amount, in the settlement token's smallest unit, the
//!   bidder gives up from the pot.
//! - `nonce` — bidder-chosen randomness. Without it the preimage space is
//!   small enough to brute-force: an observer who knows `member` and `round`
//!   could enumerate plausible discounts and read the sealed bid.
//! - `member` — binds the bid to one address, so a commitment cannot be
//!   copied and replayed by someone else.
//! - `round` — binds it to one round, so a commitment from an earlier round
//!   cannot be replayed later.
//!
//! Sealing the bid is what defeats front-running (threat T6). If bids were
//! public on submission, the last bidder could always undercut by one unit.
//!
//! ### Reveal
//!
//! During `Phase::Revealing` the bidder submits `(discount, nonce)`. The
//! contract recomputes the hash and rejects a mismatch with
//! `Error::RevealMismatch`. A bid that is never revealed is not counted.
//!
//! ### Winner
//!
//! **Lowest net request wins.** The bidder asking for the least money —
//! equivalently, offering the largest discount — takes the pot this round.
//! Ties break toward the lower member index, which is deterministic and
//! cheap; the alternative, on-chain randomness, buys fairness nobody has
//! asked for at a cost in complexity.
//!
//! ### Split
//!
//! The discount is divided **10% protocol fee / 90% dividend** to the members
//! who did not win this round. The protocol fee is the same `fee_bps`
//! mechanism the vault already applies to a rotational payout, so there is
//! one fee path rather than two.
//!
//! ## Interaction with the vault
//!
//! This contract never holds funds. It determines a recipient and a discount
//! and hands them to `EscrowVault::settle_and_pay`, which remains the only
//! code that moves money. Invariant 1 survives Tranche 2 intact, and a PR
//! that gives this crate custody of anything should be rejected on sight.
//!
//! Invariant 3 still binds: a bid winner is marked in `winners_bitmap` and
//! cannot win again until every member has been paid once. The auction
//! decides the *order* within a cycle, never the *number* of payouts.
//!
//! ## Open questions — BLOCKING
//!
//! Neither of these has an answer yet, and implementation must not proceed
//! past this line. Building an auction on an unresolved settlement rule
//! wastes the work.
//!
//! **1. Commit-without-reveal.** A member commits a bid and never reveals it.
//! They have cost the round a phase of delay at no price. Is there a bond? Is
//! it a default? Is it simply free? A free option is exploitable: commit to
//! every round, reveal only when the pot is unusually large. Any bond has to
//! be small enough not to price out the members the product exists for.
//!
//! **2. Win-then-default.** A member wins the auction, takes the pot, and
//! stops contributing. Default flagging excludes them from future *bidding*,
//! but the money is already gone and the remaining members are short. This is
//! `audit/known-issues.md` #1, severity high, and it is the single reason
//! Tranche 2 is not being written yet. Options under consideration — a bond
//! from bid winners, a reputation-weighted auction, a group clawback vote —
//! are recorded there. **No option has been chosen.**

use soroban_sdk::{contract, contractimpl, Env, String};

#[contract]
pub struct BidAuction;

#[contractimpl]
impl BidAuction {
    /// Build marker. The presence of this crate in a deployment is not a
    /// claim that the auction works — it does not exist yet.
    pub fn version(env: Env) -> String {
        String::from_str(&env, "0.1.0-tranche2-unimplemented")
    }
}
