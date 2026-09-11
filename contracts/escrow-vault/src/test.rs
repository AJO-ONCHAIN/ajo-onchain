#![cfg(test)]
//! Start here when contributing.
//!
//! Every invariant in lib.rs's header comment should have at least one test
//! that fails if the invariant is broken. Right now most of them do not have
//! one — that is what the TODO list below is, and each entry maps to a
//! numbered issue in `docs/ISSUES.md`. Claim one by commenting on the issue.
//!
//! The harness below is the whole setup you need: a mock SEP-41 token, a
//! deployed vault, and funded members. If you find yourself writing more
//! scaffolding than assertions, the harness is missing something — improve it
//! rather than copying it.

// TODO(good-first-issue #2): a member cannot contribute twice in one round.
// TODO(good-first-issue #3): advance_phase is rejected before the deadline.
// TODO(#4): settle_and_pay rejects a member who already won this cycle.
// TODO(#5): the fee lands with fee_collector, the remainder with the recipient.
// TODO(#6): property test — after N rounds every member has won exactly once.
// TODO(good-first-issue #7): join is rejected once round > 0.
// TODO(good-first-issue #8): initialize rejects member_cap of 0 or above 32.
// TODO(#9): advance_phase flags a member who missed the contribution deadline.
// TODO(#10): settle_and_pay pays the recomputed recipient, not the one passed.

extern crate std;

use ajo_shared::types::{CircleConfig, Phase, Variant};
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token, Address, Env,
};

use crate::{EscrowVault, EscrowVaultClient};

/// Seconds between phase deadlines in tests. Short enough to be readable,
/// long enough that no test accidentally depends on it being zero.
const INTERVAL: u64 = 86_400;
const CONTRIBUTION: i128 = 10_000;

/// A deployed vault plus everything needed to drive it.
///
/// `dead_code` is allowed because this harness is scaffolding for the TODO
/// list above, not for the single placeholder test below. The fields and
/// helpers a contributor will reach for — funded members, the token client,
/// the ledger clock — are here and unused on purpose, so that claiming an
/// issue means writing assertions rather than writing setup.
#[allow(dead_code)]
pub struct Harness<'a> {
    pub env: Env,
    pub vault: EscrowVaultClient<'a>,
    pub token: token::Client<'a>,
    pub token_admin: token::StellarAssetClient<'a>,
    pub founder: Address,
    pub fee_collector: Address,
}

#[allow(dead_code)]
impl Harness<'_> {
    /// Deploy a token and a rotational vault, with the founder as member 0.
    pub fn new(fee_bps: u32) -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let token_admin_address = Address::generate(&env);
        let issued = env.register_stellar_asset_contract_v2(token_admin_address);
        let token_id = issued.address();

        let founder = Address::generate(&env);
        let fee_collector = Address::generate(&env);

        let vault_id = env.register(EscrowVault, ());
        let vault = EscrowVaultClient::new(&env, &vault_id);

        let config = CircleConfig {
            token: token_id.clone(),
            member_cap: 4,
            contribution: CONTRIBUTION,
            interval: INTERVAL,
            variant: Variant::Rotational,
            fee_bps,
            fee_collector: fee_collector.clone(),
        };
        vault.initialize(&config, &founder);

        Self {
            token: token::Client::new(&env, &token_id),
            token_admin: token::StellarAssetClient::new(&env, &token_id),
            env,
            vault,
            founder,
            fee_collector,
        }
    }

    /// Create a funded address and add it to the circle.
    pub fn add_member(&self) -> Address {
        let who = Address::generate(&self.env);
        self.token_admin.mint(&who, &(CONTRIBUTION * 100));
        self.vault.join(&who);
        who
    }

    /// Move the ledger clock past the current deadline.
    pub fn pass_deadline(&self) {
        let now = self.env.ledger().timestamp();
        self.env.ledger().set_timestamp(now + INTERVAL + 1);
    }
}

/// The harness compiles, the vault initialises, and the founder is member 0
/// of a circle that starts in `Contributing` with an empty pot.
///
/// This is deliberately shallow. It is here so that a contributor picking up
/// any TODO above starts from a green `cargo test`, not from a build error.
#[test]
fn initialize_seats_the_founder_and_opens_contributions() {
    let h = Harness::new(100);
    let state = h.vault.get_state();

    assert_eq!(state.members.len(), 1);
    assert_eq!(state.members.get(0).unwrap(), h.founder);
    assert_eq!(state.round, 0);
    assert_eq!(state.phase, Phase::Contributing);
    assert_eq!(state.pot, 0);
    assert_eq!(state.winners_bitmap, 0);
    assert_eq!(state.contributed_bitmap, 0);
    assert_eq!(state.default_bitmap, 0);

    let config = h.vault.get_config();
    assert_eq!(config.contribution, CONTRIBUTION);
    assert_eq!(config.fee_collector, h.fee_collector);
}
