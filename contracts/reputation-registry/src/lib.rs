#![no_std]
//! # ReputationRegistry — portable contribution history.
//!
//! **Status: Tranche 3. Not implemented.**
//!
//! ## What it is for
//!
//! Someone who has completed six Ajo circles without missing a contribution
//! has demonstrably good credit behaviour, and today that fact dies with the
//! circle. It lives in a WhatsApp group and in the collector's memory. It
//! cannot be shown to a lender, carried to another group, or used to get
//! better terms anywhere.
//!
//! This registry records contribution and default history so that it travels
//! with the member.
//!
//! ## It is a public primitive, not an AJO database
//!
//! This is the design decision that matters, and it should not be quietly
//! reversed later.
//!
//! The registry is **readable by any Stellar protocol** — SoroSusu included,
//! and any lender, anchor or savings product that wants it. It is not an
//! AJO-private store that happens to live on-chain. One shared credit
//! primitive for African ROSCA users is worth more to those users than three
//! siloed ones, including to us: a registry only AJO reads is only as useful
//! as AJO is large, while a shared one gets more useful as the whole category
//! grows.
//!
//! Read access is therefore unauthenticated and always will be. A PR adding
//! an allowlist, a read fee, or a per-caller gate on the read path is
//! changing the product, not optimising it, and needs a discussion first.
//!
//! ## Two kinds of record, stored separately
//!
//! The separation below is the whole integrity story of this contract.
//!
//! **1. Contract-enforced facts.** Contribution counts, defaults, completed
//! cycles. These are written only by an `EscrowVault` reporting its own
//! history. They are as trustworthy as the vault contract, which is to say
//! they are enforced, not asserted.
//!
//! **2. Model output.** `services/scoring` may publish a signed attestation —
//! a credit score, a default early-warning flag. This is stored **under a
//! separate storage key**, labelled as off-chain model output, and carries
//! the signing key and the model version that produced it.
//!
//! Consumers must be able to tell the two apart without reading our
//! documentation, which is why they are different keys rather than fields of
//! one struct. A score is an opinion. A contribution count is a fact. Storing
//! them together would let the first borrow the authority of the second.
//!
//! The scoring service's attestation is its **only** write path, and it is
//! advisory. It has no route to move funds, and no route to change a
//! contract-enforced fact. A PR giving it one should be rejected on sight.
//!
//! ## Open questions
//!
//! - **Privacy.** A public, permanent default record attached to a Stellar
//!   address is a real harm to a real person, and "it is pseudonymous" is a
//!   thin defence once an address touches a KYC'd anchor. Sharibo has shown
//!   payout privacy is achievable with Stellar's native BLS12-381 pairings;
//!   the same tools may apply here. Unresolved.
//! - **Sybil resistance.** Nothing stops one person running several
//!   addresses, building history on each, and defaulting on all of them at
//!   once. Any answer probably involves the anchor's KYC, which drags an
//!   off-chain trusted party into an on-chain primitive. Unresolved.
//! - **Write authorisation.** How does the registry know a caller really is
//!   an `EscrowVault` and not a contract impersonating one? Most likely a
//!   factory-attested allowlist of deployed vault addresses. Unresolved.

use soroban_sdk::{contract, contractimpl, Env, String};

#[contract]
pub struct ReputationRegistry;

#[contractimpl]
impl ReputationRegistry {
    /// Build marker. This crate stores no history yet.
    pub fn version(env: Env) -> String {
        String::from_str(&env, "0.1.0-tranche3-unimplemented")
    }
}
