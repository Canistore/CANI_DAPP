use candid::{CandidType, Principal};
use canistore_types::constant::Environment;
use ic_cdk::{init, post_upgrade, pre_upgrade, trap};
use serde::Deserialize;

use crate::store;

#[derive(CandidType, Clone, Debug, Deserialize)]
pub enum CanisterArgs {
    Init(StateInitArgs),
    Upgrade(StateUpgradeArgs),
}

#[derive(CandidType, Clone, Debug, Deserialize)]
pub struct StateInitArgs {
    name: String,
    owner: Principal,
    env: Environment,
    dao_canister_id: Principal,
    indexer_canister_id: Option<Principal>,
}

#[derive(CandidType, Clone, Debug, Deserialize)]
pub struct StateUpgradeArgs {
    name: Option<String>,
    owner: Option<Principal>,
    env: Option<Environment>,
    dao_canister_id: Option<Principal>,
    indexer_canister_id: Option<Principal>,
}

#[init]
fn init(args: Option<CanisterArgs>) {
    match args {
        Some(CanisterArgs::Init(init_args)) => {
            // Initialize the state with provided values or defaults where not provided
            store::state::with_mut(|state_ref| {
                state_ref.name = init_args.name;
                state_ref.owner = init_args.owner;
                state_ref.dao_canister_id = init_args.dao_canister_id;
                state_ref.env = init_args.env;

                if let Some(indexer_canister_id) = init_args.indexer_canister_id {
                    state_ref.indexer_canister_id = indexer_canister_id;
                } else {
                    state_ref.indexer_canister_id = Principal::anonymous()
                }

                state_ref.user_count = 0;
            });
            store::state::save();
        }
        Some(CanisterArgs::Upgrade(_)) => {
            ic_cdk::trap(
                "Cannot initialize the canister with an Upgrade args. Please provide an Init args.",
            );
        }
        None => {
            trap("No initialization arguments provided. Use default initialization.");
        }
    }
}

#[pre_upgrade]
fn pre_upgrade() {
    store::state::save();
}

#[post_upgrade]
fn post_upgrade(args: Option<CanisterArgs>) {
    store::state::load();

    match args {
        Some(CanisterArgs::Upgrade(upgrade_args)) => {
            store::state::with_mut(|state_ref| {
                if let Some(name) = upgrade_args.name {
                    state_ref.name = name;
                }
                if let Some(owner) = upgrade_args.owner {
                    state_ref.owner = owner;
                }
                if let Some(dao_canister_id) = upgrade_args.dao_canister_id {
                    state_ref.dao_canister_id = dao_canister_id;
                }
                if let Some(indexer_canister_id) = upgrade_args.indexer_canister_id {
                    state_ref.indexer_canister_id = indexer_canister_id;
                }
                if let Some(env) = upgrade_args.env {
                    state_ref.env = env;
                }
            });
        }
        Some(CanisterArgs::Init(_)) => {
            ic_cdk::trap(
                "Cannot upgrade the canister with Init args. Please provide Upgrade args.",
            );
        }
        None => {
            // No arguments provided; continue with the existing state
        }
    }
}
