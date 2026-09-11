//! Stored types for a savings circle.
//!
//! These structures live in contract instance storage. Adding, removing or
//! reordering a field changes the on-ledger encoding and requires a migration
//! plan, so treat this file as a schema rather than as ordinary code.

use soroban_sdk::{contracttype, Address, Vec};

/// Upper bound on circle membership.
///
/// The three bitmaps in [`CircleState`] are `u32`, one bit per member, indexed
/// by position in `members`. That caps a circle at 32 people. This is a
/// documented, accepted limitation rather than a bug — see
/// `audit/known-issues.md` #4. Real Ajo circles are 5 to 20 people, and a
/// wider bitmap would cost storage and complexity for a case that does not
/// occur in the field.
pub const MAX_MEMBERS: u32 = 32;

/// How a circle chooses who is paid in each round.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Variant {
    /// Fixed order. The lowest-indexed member who has not yet won is paid.
    /// This is the traditional Ajo rotation and is Tranche 1 scope.
    Rotational,
    /// Sealed-bid discount auction decides the recipient. Tranche 2; the
    /// contract recognises the variant but the auction is not implemented.
    BidBased,
}

/// Where a circle is in the current round.
///
/// Phases advance exactly one step at a time. No entry point anywhere in the
/// system accepts a target phase from a caller — see invariant 2 in
/// `escrow-vault`.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Phase {
    /// Members pay in for this round.
    Contributing,
    /// Bids are committed as hashes. `BidBased` circles only.
    Bidding,
    /// Commitments are revealed and checked. `BidBased` circles only.
    Revealing,
    /// The recipient for this round is determined.
    Settling,
    /// The pot is payable. Only `settle_and_pay` leaves this phase.
    Payout,
}

/// The terms of a circle, fixed at initialisation.
///
/// Every field here is immutable for the life of the contract. There is no
/// setter for any of them. A circle whose terms could change mid-cycle would
/// not be meaningfully trustless, because the founder could raise the
/// contribution or redirect the fee after members had committed funds.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CircleConfig {
    /// SEP-41 token used for contributions and payouts. Assumed trusted; see
    /// `audit/scope.md` assumption 1.
    pub token: Address,
    /// Maximum members. Must be in `1..=MAX_MEMBERS`.
    pub member_cap: u32,
    /// Amount each member pays per round, in the token's smallest unit.
    pub contribution: i128,
    /// Seconds between phase deadlines.
    pub interval: u64,
    pub variant: Variant,
    /// Protocol fee on each payout, in basis points.
    pub fee_bps: u32,
    pub fee_collector: Address,
}

/// Everything about a circle that changes.
///
/// The three bitmaps are indexed by a member's position in `members`. That
/// index is the reason `join` is rejected once `round > 0`: appending is safe,
/// but any reordering would silently reassign another member's contribution
/// and payout history.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CircleState {
    pub members: Vec<Address>,
    /// Completed rounds. Round 0 is the joining window.
    pub round: u32,
    pub phase: Phase,
    /// Ledger timestamp at which the current phase may be advanced.
    pub deadline: u64,
    /// Members who have been paid in the current cycle. Invariant 3.
    pub winners_bitmap: u32,
    /// Members who have paid into the current round. Cleared each payout.
    pub contributed_bitmap: u32,
    /// Members who missed a contribution deadline. Never cleared.
    pub default_bitmap: u32,
    /// Contributions held for the current round.
    pub pot: i128,
}

/// Bit mask covering `member_count` members.
///
/// `1u32 << 32` is a shift overflow, which panics rather than producing the
/// mask you wanted, so the full-width case is handled separately. This is the
/// single place that arithmetic appears, so every bitmap helper inherits the
/// guard.
fn full_mask(member_count: u32) -> u32 {
    if member_count >= MAX_MEMBERS {
        u32::MAX
    } else {
        (1u32 << member_count) - 1
    }
}

/// Bit for a member index, or `0` for an out-of-range index.
///
/// Returning `0` makes every read report "not set" and every write a no-op for
/// an index that cannot exist, instead of panicking on the shift.
fn bit(index: u32) -> u32 {
    if index >= MAX_MEMBERS {
        0
    } else {
        1u32 << index
    }
}

impl CircleState {
    pub fn has_won(&self, index: u32) -> bool {
        self.winners_bitmap & bit(index) != 0
    }

    pub fn mark_won(&mut self, index: u32) {
        self.winners_bitmap |= bit(index);
    }

    pub fn has_contributed(&self, index: u32) -> bool {
        self.contributed_bitmap & bit(index) != 0
    }

    pub fn mark_contributed(&mut self, index: u32) {
        self.contributed_bitmap |= bit(index);
    }

    pub fn in_default(&self, index: u32) -> bool {
        self.default_bitmap & bit(index) != 0
    }

    pub fn mark_default(&mut self, index: u32) {
        self.default_bitmap |= bit(index);
    }

    /// Has every member been paid exactly once?
    ///
    /// A circle with no members is not complete. Treating the empty case as
    /// complete would let a degenerate circle emit `cycdone` and reset its
    /// winners bitmap, and "vacuously true" is the wrong answer to give a
    /// state machine that acts on it.
    pub fn cycle_complete(&self, member_count: u32) -> bool {
        if member_count == 0 {
            return false;
        }
        let mask = full_mask(member_count);
        self.winners_bitmap & mask == mask
    }
}
