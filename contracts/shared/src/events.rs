//! Event definitions and publish helpers.
//!
//! **These topics are a contract with the indexer.** `apps/indexer` branches
//! on them by name. Renaming one is a breaking change: the indexer stops
//! seeing the event, the dashboard silently goes stale, and nothing fails
//! loudly. If you must rename a topic, change both sides in the same pull
//! request and say so in the description.
//!
//! | Topic     | Struct              | Emitted by                     |
//! |-----------|---------------------|--------------------------------|
//! | `created` | [`CircleCreated`]   | `EscrowVault::initialize`      |
//! | `joined`  | [`MemberJoined`]    | `initialize`, `join`           |
//! | `contrib` | [`ContributionMade`]| `EscrowVault::contribute`      |
//! | `phase`   | [`PhaseAdvanced`]   | `EscrowVault::advance_phase`   |
//! | `payout`  | [`PayoutSettled`]   | `EscrowVault::settle_and_pay`  |
//! | `default` | [`MemberDefaulted`] | `EscrowVault::advance_phase`   |
//! | `cycdone` | [`CycleCompleted`]  | `EscrowVault::settle_and_pay`  |
//!
//! Every topic must fit `symbol_short!`, which is at most 9 characters. That
//! is why the topic is `cycdone` and not `cycle_done`.
//!
//! The topic strings are declared once, in the `topics = [...]` argument on
//! each struct. There is deliberately no parallel table of `Symbol` constants
//! — two sources of truth for a wire format drift apart, and the one that
//! drifts silently is the one that breaks the indexer.
//!
//! Non-topic fields are published as a map keyed by field name, so a consumer
//! reads `amount` rather than "the second element of the tuple". Renaming a
//! field is therefore also a breaking change for the indexer.
//!
//! The `emit_*` helpers exist so no contract constructs an event by hand. One
//! call site per event means the payload shape is defined once, here, and
//! `docs/contracts/events.md` has a single thing to describe.

use soroban_sdk::{contractevent, Address, Env};

/// A vault was initialised.
#[contractevent(topics = ["created"])]
pub struct CircleCreated {
    pub founder: Address,
    pub token: Address,
    pub contribution: i128,
}

/// A member was seated. Emitted for the founder at index 0 as well as for
/// every later `join`, so the indexer can rebuild the member list from
/// events alone without reading contract state.
#[contractevent(topics = ["joined"])]
pub struct MemberJoined {
    pub member: Address,
    pub index: u32,
}

/// A member paid into the current round.
#[contractevent(topics = ["contrib"])]
pub struct ContributionMade {
    pub member: Address,
    pub amount: i128,
    pub round: u32,
}

/// The round advanced exactly one phase.
///
/// `phase_index` is a stable number rather than the `Phase` enum, so that
/// reordering the enum cannot change the meaning of events already on the
/// ledger. The mapping lives in `escrow_vault::phase_index`.
#[contractevent(topics = ["phase"])]
pub struct PhaseAdvanced {
    pub round: u32,
    pub phase_index: u32,
}

/// The pot was paid out. `net` is what the recipient received; `fee` went to
/// the configured fee collector. `round` is the round that just closed.
#[contractevent(topics = ["payout"])]
pub struct PayoutSettled {
    pub recipient: Address,
    pub net: i128,
    pub fee: i128,
    pub round: u32,
}

/// A member missed a contribution deadline. Permanent: the default bitmap is
/// never cleared.
#[contractevent(topics = ["default"])]
pub struct MemberDefaulted {
    pub member: Address,
    pub round: u32,
}

/// Every member has now been paid once and the winners bitmap has reset.
#[contractevent(topics = ["cycdone"])]
pub struct CycleCompleted {
    pub round: u32,
}

pub fn emit_created(env: &Env, founder: &Address, token: &Address, contribution: i128) {
    CircleCreated {
        founder: founder.clone(),
        token: token.clone(),
        contribution,
    }
    .publish(env);
}

pub fn emit_joined(env: &Env, member: &Address, index: u32) {
    MemberJoined {
        member: member.clone(),
        index,
    }
    .publish(env);
}

pub fn emit_contrib(env: &Env, member: &Address, amount: i128, round: u32) {
    ContributionMade {
        member: member.clone(),
        amount,
        round,
    }
    .publish(env);
}

pub fn emit_phase(env: &Env, round: u32, phase_index: u32) {
    PhaseAdvanced { round, phase_index }.publish(env);
}

pub fn emit_payout(env: &Env, recipient: &Address, net: i128, fee: i128, round: u32) {
    PayoutSettled {
        recipient: recipient.clone(),
        net,
        fee,
        round,
    }
    .publish(env);
}

pub fn emit_default(env: &Env, member: &Address, round: u32) {
    MemberDefaulted {
        member: member.clone(),
        round,
    }
    .publish(env);
}

pub fn emit_cycdone(env: &Env, round: u32) {
    CycleCompleted { round }.publish(env);
}
