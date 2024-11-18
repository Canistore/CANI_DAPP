use canistore_types::indexer::CanisterArgs;

use crate::store;

#[ic_cdk::init]
fn init(args: Option<CanisterArgs>) {
    match args {
        Some(CanisterArgs::Init(args)) => {
            store::state::with_mut(|indexer| {
                if !args.name.is_empty() {
                    indexer.name = args.name;
                }
                indexer.owner = args.owner;
                indexer.user_count = 0;
            });
            store::state::save();
        }
        Some(CanisterArgs::Upgrade(_)) => {
            ic_cdk::trap(
                "Cannot initialize the canister with an Upgrade args. Please provide an Init args.",
            );
        }
        None => {
            ic_cdk::trap("No initialization arguments provided");
        }
    }
}

#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    store::state::save();
}

#[ic_cdk::post_upgrade]
fn post_upgrade(args: Option<CanisterArgs>) {
    store::state::load();
    match args {
        Some(CanisterArgs::Upgrade(args)) => {
            store::state::with_mut(|indexer| {
                if let Some(owner) = args.owner {
                    indexer.owner = owner;
                }
                if let Some(user_count) = args.user_count {
                    indexer.user_count = user_count;
                }
            });
        }
        Some(CanisterArgs::Init(_)) => {
            ic_cdk::trap(
                "Cannot upgrade the canister with an Init args. Please provide an Upgrade args.",
            );
        }
        _ => {}
    }
}
