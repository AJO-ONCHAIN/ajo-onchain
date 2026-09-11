#![no_std]
//! # CircleFactory — deploys one `EscrowVault` per circle.
//!
//! Every circle gets its own vault instance. The alternative, one contract
//! holding every circle keyed by id, would put unrelated groups' savings in a
//! single blast radius: one bug in the keying logic and circle A can reach
//! circle B's pot. Separate instances make that class of bug impossible
//! rather than merely absent.
//!
//! The factory holds no funds and has no path to any vault's money. It stores
//! a WASM hash and a list of addresses it has deployed, which is what the
//! indexer subscribes to so it knows which contracts to watch.
//!
//! ## Open question for auditors (threat T4)
//!
//! `set_vault_wasm` lets the admin change the WASM hash used for *future*
//! deployments. Already-deployed vaults are untouched — Soroban does not
//! rewrite a deployed contract because the factory changed a stored hash — so
//! existing members' funds are not at risk from this. The risk is narrower
//! and still real: a compromised admin can make every *new* circle run
//! malicious code, and a member joining a fresh circle has no easy way to
//! tell.
//!
//! Should the hash be frozen at `initialize` instead? Freezing removes the
//! threat and also removes the ability to ship a fix to a vault bug. This is
//! posed openly in `audit/threat-model.md` rather than decided unilaterally.
//! **Currently unmitigated.**

use ajo_shared::errors::Error;
use soroban_sdk::{contract, contractevent, contractimpl, contracttype, Address, BytesN, Env, Vec};

/// A circle was deployed.
///
/// The indexer uses this to discover vaults to watch. It is the only reason
/// the factory keeps a list at all.
#[contractevent(topics = ["deployed"])]
pub struct CircleDeployed {
    pub vault: Address,
    pub founder: Address,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    VaultWasm,
    Circles,
}

#[contract]
pub struct CircleFactory;

#[contractimpl]
impl CircleFactory {
    pub fn initialize(env: Env, admin: Address, vault_wasm: BytesN<32>) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::VaultWasm, &vault_wasm);
        env.storage()
            .instance()
            .set(&DataKey::Circles, &Vec::<Address>::new(&env));
        Ok(())
    }

    /// Deploy a vault for `founder`, salted by `salt`.
    ///
    /// The vault is *not* initialised here. `TODO(#12)`: pass the
    /// `CircleConfig` through as constructor arguments so that deployment and
    /// initialisation are one atomic call — a vault that exists but is
    /// uninitialised can be initialised by whoever calls first, which is a
    /// front-running window we should not leave open.
    pub fn deploy_circle(env: Env, founder: Address, salt: BytesN<32>) -> Result<Address, Error> {
        founder.require_auth();

        let wasm_hash: BytesN<32> = env
            .storage()
            .instance()
            .get(&DataKey::VaultWasm)
            .ok_or(Error::NotInitialized)?;

        let vault = env
            .deployer()
            .with_current_contract(salt)
            .deploy_v2(wasm_hash, ());

        let mut circles: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Circles)
            .unwrap_or_else(|| Vec::new(&env));
        circles.push_back(vault.clone());
        env.storage().instance().set(&DataKey::Circles, &circles);

        CircleDeployed {
            vault: vault.clone(),
            founder,
        }
        .publish(&env);
        Ok(vault)
    }

    /// Every vault this factory has deployed.
    ///
    /// This is the list `scripts/deploy-testnet.sh` tells a stranger to
    /// invoke. An unverifiable trustlessness claim is marketing; a one-line
    /// command anyone can run is evidence.
    pub fn list_circles(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::Circles)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Change the WASM used for future deployments. See threat T4 above.
    pub fn set_vault_wasm(env: Env, vault_wasm: BytesN<32>) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        admin.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::VaultWasm, &vault_wasm);
        Ok(())
    }

    pub fn get_vault_wasm(env: Env) -> Result<BytesN<32>, Error> {
        env.storage()
            .instance()
            .get(&DataKey::VaultWasm)
            .ok_or(Error::NotInitialized)
    }
}
