//! Contract error codes, namespaced by category.
//!
//! The numeric ranges are deliberate and stable. An indexer or frontend maps a
//! code to a message; grouping by hundreds means a consumer can fall back to a
//! sensible category message for a code it does not recognise yet.
//!
//! | Range | Category          |
//! |-------|-------------------|
//! | `1xx` | generic / auth    |
//! | `2xx` | membership        |
//! | `3xx` | round lifecycle   |
//! | `4xx` | bidding (T2)      |
//!
//! Never renumber an existing variant. Add new ones at the end of their range.

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // --- 1xx: generic and authorisation ---
    AlreadyInitialized = 100,
    NotInitialized = 101,
    Unauthorized = 102,
    InvalidAmount = 103,

    // --- 2xx: membership ---
    CircleFull = 200,
    NotAMember = 201,
    AlreadyAMember = 202,
    MemberInDefault = 203,
    AlreadyWonThisCycle = 204,

    // --- 3xx: round lifecycle ---
    WrongPhase = 300,
    AlreadyContributed = 301,
    ContributionMissing = 302,
    DeadlineNotReached = 303,
    DeadlinePassed = 304,
    CycleComplete = 305,

    // --- 4xx: bidding (Tranche 2, not yet implemented) ---
    BidAlreadyCommitted = 400,
    NoBidCommitted = 401,
    RevealMismatch = 402,
    BidTooHigh = 403,
    NoWinningBid = 404,
}
