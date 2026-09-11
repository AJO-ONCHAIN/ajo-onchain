#![no_std]
//! # EscrowVault — custody and the round state machine for one savings circle.
//!
//! One vault instance holds the funds of exactly one circle. It is the whole
//! trust model of AJO Onchain: if this contract is correct, no member has to
//! trust the founder, the developers, the indexer, or each other.
//!
//! ## The three invariants
//!
//! 1. **No privileged withdrawal.** There is no admin withdrawal function
//!    anywhere in `EscrowVault`. Funds leave the contract only through
//!    `settle_and_pay`, and only to a member of that circle. No `pause`, no
//!    `set_admin`, no `emergency_withdraw`, no upgrade hook that could
//!    introduce one.
//! 2. **No phase skipping.** Phases advance exactly one step at a time via
//!    `advance_phase`. No entry point accepts a target phase from the caller.
//! 3. **One win per cycle.** A member may receive at most one payout before
//!    every member has received one, enforced by a bitmap in contract state.
//!
//! Any change that weakens one of these belongs in audit/known-issues.md
//! BEFORE it is merged.
//!
//! ## Reading the invariants in the source
//!
//! Invariant 1 is a claim about absence, which is the hard kind to verify.
//! The way to check it is to read every `#[contractimpl]` function below and
//! confirm that the only `transfer` calls moving funds out of
//! `env.current_contract_address()` are the two in `settle_and_pay`, and that
//! their destinations are the recomputed recipient and the configured fee
//! collector. There is no third path, and there is no setter for
//! `fee_collector`.
//!
//! Invariant 2 is enforced by `next_phase`, a total function from
//! `(Phase, Variant)` to the next phase. No caller supplies a target.
//!
//! Invariant 3 is enforced by `winners_bitmap` in `CircleState`, checked in
//! `settle_and_pay`.
//!
//! ## Permissionless pokes
//!
//! `advance_phase` and `settle_and_pay` require no authorisation. This is
//! deliberate and it is threat T3 in `audit/threat-model.md`: if advancing a
//! round needed a specific member's signature, that member could stall the
//! circle and freeze everyone else's funds by doing nothing. Anyone may poke a
//! circle whose deadline has passed. Neither function lets the caller choose
//! who benefits.

use ajo_shared::{
    errors::Error,
    events,
    types::{CircleConfig, CircleState, Phase, Variant, MAX_MEMBERS},
};
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Vec};

/// Basis-point denominator. `fee_bps` is a fraction of this.
const BPS_DENOMINATOR: i128 = 10_000;

/// Instance storage keys. Two entries, both always present after
/// `initialize`, both rewritten in place. There is no per-member storage: the
/// member list and all three bitmaps live inside `CircleState`, so a round
/// costs a fixed number of ledger entries regardless of circle size.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Config,
    State,
}

#[contract]
pub struct EscrowVault;

#[contractimpl]
impl EscrowVault {
    /// Create the circle. Callable once.
    ///
    /// The founder becomes member 0 and the first contribution deadline is set
    /// immediately, so the circle is live the moment it exists rather than
    /// waiting for a separate "start" call that someone could forget to make.
    pub fn initialize(env: Env, config: CircleConfig, founder: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Config) {
            return Err(Error::AlreadyInitialized);
        }
        founder.require_auth();

        // A `member_cap` above MAX_MEMBERS would silently lose every member
        // past bit 31 of all three bitmaps, so it is rejected rather than
        // clamped.
        if config.member_cap == 0 || config.member_cap > MAX_MEMBERS {
            return Err(Error::InvalidAmount);
        }
        if config.contribution <= 0 {
            return Err(Error::InvalidAmount);
        }
        // A fee above 100% would make `net` negative and the payout transfer
        // would panic mid-settlement, stranding the pot.
        if i128::from(config.fee_bps) > BPS_DENOMINATOR {
            return Err(Error::InvalidAmount);
        }

        let mut members = Vec::new(&env);
        members.push_back(founder.clone());

        let state = CircleState {
            members,
            round: 0,
            phase: Phase::Contributing,
            deadline: env.ledger().timestamp() + config.interval,
            winners_bitmap: 0,
            contributed_bitmap: 0,
            default_bitmap: 0,
            pot: 0,
        };

        events::emit_created(&env, &founder, &config.token, config.contribution);
        events::emit_joined(&env, &founder, 0);

        env.storage().instance().set(&DataKey::Config, &config);
        env.storage().instance().set(&DataKey::State, &state);
        Ok(())
    }

    /// Join the circle. Only while `round == 0`.
    ///
    /// Joining closes once the first round settles because every bitmap is
    /// indexed by position in `members`. Appending is safe; anything that
    /// could reorder the list would silently reassign one member's
    /// contribution and payout history to someone else. See threat T7.
    pub fn join(env: Env, who: Address) -> Result<u32, Error> {
        let config = load_config(&env)?;
        let mut state = load_state(&env)?;
        who.require_auth();

        if state.round > 0 {
            return Err(Error::WrongPhase);
        }
        if state.members.len() >= config.member_cap {
            return Err(Error::CircleFull);
        }
        if member_index(&state, &who).is_some() {
            return Err(Error::AlreadyAMember);
        }

        state.members.push_back(who.clone());
        let index = state.members.len() - 1;

        events::emit_joined(&env, &who, index);
        env.storage().instance().set(&DataKey::State, &state);
        Ok(index)
    }

    /// Pay this round's contribution into the pot.
    ///
    /// State is written before the token transfer
    /// (checks-effects-interactions), so a token contract that calls back into
    /// this vault cannot observe a stale `contributed_bitmap` and contribute
    /// twice. `settle_and_pay` does not currently follow this ordering — that
    /// is filed as known-issues #2 rather than quietly fixed, because the fix
    /// should land together with the test that proves it.
    pub fn contribute(env: Env, who: Address) -> Result<(), Error> {
        let config = load_config(&env)?;
        let mut state = load_state(&env)?;
        who.require_auth();

        if state.phase != Phase::Contributing {
            return Err(Error::WrongPhase);
        }
        let index = member_index(&state, &who).ok_or(Error::NotAMember)?;
        if state.has_contributed(index) {
            return Err(Error::AlreadyContributed);
        }

        state.mark_contributed(index);
        state.pot += config.contribution;
        let round = state.round;
        env.storage().instance().set(&DataKey::State, &state);

        let vault = env.current_contract_address();
        token::Client::new(&env, &config.token).transfer(&who, &vault, &config.contribution);

        events::emit_contrib(&env, &who, config.contribution, round);
        Ok(())
    }

    /// Advance the round exactly one phase. Permissionless, after the deadline.
    ///
    /// Takes no arguments at all. That is invariant 2: there is no way to
    /// express "skip to Payout" through this interface, so no caller can.
    pub fn advance_phase(env: Env) -> Result<Phase, Error> {
        let config = load_config(&env)?;
        let mut state = load_state(&env)?;

        if env.ledger().timestamp() < state.deadline {
            return Err(Error::DeadlineNotReached);
        }

        // Leaving Contributing is the moment a missed payment becomes a
        // default. Flagging here rather than at payout means the record is
        // written even if the round never settles.
        if state.phase == Phase::Contributing {
            flag_defaulters(&env, &mut state);
        }

        let next = next_phase(state.phase, config.variant)?;
        state.phase = next;
        state.deadline = env.ledger().timestamp() + config.interval;

        events::emit_phase(&env, state.round, phase_index(next));
        env.storage().instance().set(&DataKey::State, &state);
        Ok(next)
    }

    /// Pay the pot out and open the next round. Permissionless.
    ///
    /// `recipient` is supplied by the caller for convenience, never for
    /// authority. For a rotational circle the contract recomputes who is owed
    /// the pot — the lowest-indexed member who has not yet won — and rejects
    /// any other address. A caller who passes their own address out of turn
    /// gets an error, not the money.
    pub fn settle_and_pay(env: Env, recipient: Address) -> Result<i128, Error> {
        let config = load_config(&env)?;
        let mut state = load_state(&env)?;

        if state.phase != Phase::Payout {
            return Err(Error::WrongPhase);
        }

        let index = match config.variant {
            Variant::Rotational => {
                let (expected_index, expected) =
                    next_in_rotation(&state).ok_or(Error::CycleComplete)?;
                if expected != recipient {
                    return Err(Error::Unauthorized);
                }
                expected_index
            }
            // Tranche 2. The recipient of a bid-based round is the winner of a
            // sealed-bid auction that does not exist yet. An error is the
            // honest behaviour here: the alternative is to fall back to
            // rotation silently and pay the wrong member.
            Variant::BidBased => return Err(Error::NoWinningBid),
        };

        if state.has_won(index) {
            return Err(Error::AlreadyWonThisCycle);
        }

        let fee = state.pot * i128::from(config.fee_bps) / BPS_DENOMINATOR;
        let net = state.pot - fee;

        // KNOWN ISSUE (audit/known-issues.md #2, medium, open):
        // these transfers run before the state write below, which is the wrong
        // order. It is safe against a well-behaved SEP-41 token, and every
        // token we intend to support is one, but "safe given a trusted
        // dependency" is a weaker property than "safe regardless", and this is
        // the contract holding the money. Reorder to
        // checks-effects-interactions before the audit. Left in place and
        // filed openly so the fix arrives with a regression test.
        let client = token::Client::new(&env, &config.token);
        let vault = env.current_contract_address();
        if fee > 0 {
            client.transfer(&vault, &config.fee_collector, &fee);
        }
        if net > 0 {
            client.transfer(&vault, &recipient, &net);
        }

        let paid_round = state.round;
        state.mark_won(index);
        state.pot = 0;
        state.contributed_bitmap = 0;
        state.round += 1;
        state.phase = Phase::Contributing;
        state.deadline = env.ledger().timestamp() + config.interval;

        events::emit_payout(&env, &recipient, net, fee, paid_round);

        // Everyone has been paid once: the cycle closes and a new one opens
        // with a cleared winners bitmap. Until this point invariant 3 makes a
        // second payout to the same member impossible.
        if state.cycle_complete(state.members.len()) {
            state.winners_bitmap = 0;
            events::emit_cycdone(&env, paid_round);
        }

        env.storage().instance().set(&DataKey::State, &state);
        Ok(net)
    }

    pub fn get_config(env: Env) -> Result<CircleConfig, Error> {
        load_config(&env)
    }

    pub fn get_state(env: Env) -> Result<CircleState, Error> {
        load_state(&env)
    }
}

/// The phase transition table. Total over `(Phase, Variant)`.
///
/// Rotational circles skip `Bidding` and `Revealing` through an explicit match
/// arm rather than by omission. Writing the skip out means adding a phase
/// later produces a compile error here instead of a silently unreachable
/// state.
fn next_phase(current: Phase, variant: Variant) -> Result<Phase, Error> {
    match (current, variant) {
        (Phase::Contributing, Variant::Rotational) => Ok(Phase::Settling),
        (Phase::Contributing, Variant::BidBased) => Ok(Phase::Bidding),
        (Phase::Bidding, _) => Ok(Phase::Revealing),
        (Phase::Revealing, _) => Ok(Phase::Settling),
        (Phase::Settling, _) => Ok(Phase::Payout),
        // Only `settle_and_pay` leaves Payout. If `advance_phase` could, the
        // pot would roll into the next round unpaid.
        (Phase::Payout, _) => Err(Error::WrongPhase),
    }
}

/// Stable numeric form of a phase, for the `phase` event.
///
/// Defined here rather than derived from the enum so that reordering `Phase`
/// cannot silently change the meaning of already-emitted events.
fn phase_index(phase: Phase) -> u32 {
    match phase {
        Phase::Contributing => 0,
        Phase::Bidding => 1,
        Phase::Revealing => 2,
        Phase::Settling => 3,
        Phase::Payout => 4,
    }
}

/// Position of `who` in the member list, if they are a member.
fn member_index(state: &CircleState, who: &Address) -> Option<u32> {
    state.members.first_index_of(who)
}

/// The member owed the pot in a rotational circle: lowest index not yet paid.
fn next_in_rotation(state: &CircleState) -> Option<(u32, Address)> {
    for index in 0..state.members.len() {
        if !state.has_won(index) {
            return Some((index, state.members.get(index)?));
        }
    }
    None
}

/// Mark every member who did not pay into this round.
///
/// The default bitmap is never cleared. A default is a permanent part of a
/// member's record, and that record is what `reputation-registry` will
/// eventually export as portable credit history.
fn flag_defaulters(env: &Env, state: &mut CircleState) {
    let round = state.round;
    for index in 0..state.members.len() {
        if state.has_contributed(index) || state.in_default(index) {
            continue;
        }
        state.mark_default(index);
        if let Some(member) = state.members.get(index) {
            events::emit_default(env, &member, round);
        }
    }
}

fn load_config(env: &Env) -> Result<CircleConfig, Error> {
    env.storage()
        .instance()
        .get(&DataKey::Config)
        .ok_or(Error::NotInitialized)
}

fn load_state(env: &Env) -> Result<CircleState, Error> {
    env.storage()
        .instance()
        .get(&DataKey::State)
        .ok_or(Error::NotInitialized)
}

mod test;
