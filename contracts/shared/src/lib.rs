#![no_std]
//! Shared vocabulary for the AJO Onchain contracts.
//!
//! Three modules, each of which is a contract with something outside this
//! crate:
//!
//! - [`errors`] is a contract with the frontend and the indexer. Codes are
//!   namespaced by category so an off-chain consumer can map a failure to a
//!   user-facing message without guessing.
//! - [`types`] is a contract with the ledger. These structures are stored, so
//!   changing a field is a storage migration, not a refactor.
//! - [`events`] is a contract with the indexer. Renaming a topic is a breaking
//!   change and both sides must move in the same pull request.

pub mod errors;
pub mod events;
pub mod types;

pub use errors::Error;
pub use events::*;
pub use types::{CircleConfig, CircleState, Phase, Variant};
