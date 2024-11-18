use candid::{CandidType, Principal};
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
    user_canister_id: Principal,
    platform_canister_id: Option<Principal>,
}

#[derive(CandidType, Clone, Debug, Deserialize)]
pub struct StateUpgradeArgs {
    name: Option<String>,
    owner: Option<Principal>,
    user_canister_id: Option<Principal>,
    platform_canister_id: Option<Principal>,
}

#[init]
fn init(args: Option<CanisterArgs>) {
    match args {
        Some(CanisterArgs::Init(init_args)) => {
            // Initialize the state with provided values or defaults where not provided
            store::state::with_mut(|state_ref| {
                state_ref.name = init_args.name;
                state_ref.owner = init_args.owner;
                state_ref.user_canister_id = init_args.user_canister_id;
                if let Some(platform_canister_id) = init_args.platform_canister_id {
                    state_ref.platform_canister_id = platform_canister_id;
                }
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
    store::certified::save();
}

#[post_upgrade]
fn post_upgrade(args: Option<CanisterArgs>) {
    store::certified::load();

    match args {
        Some(CanisterArgs::Upgrade(upgrade_args)) => {
            store::state::with_mut(|state_ref| {
                if let Some(name) = upgrade_args.name {
                    state_ref.name = name;
                }
                if let Some(owner) = upgrade_args.owner {
                    state_ref.owner = owner;
                }
                if let Some(user_canister_id) = upgrade_args.user_canister_id {
                    state_ref.user_canister_id = user_canister_id;
                }
                if let Some(platform_canister_id) = upgrade_args.platform_canister_id {
                    state_ref.platform_canister_id = platform_canister_id;
                }
            });
            store::state::save();
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
