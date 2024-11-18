use canistore_types::space::CanisterArgs;

use crate::store;

#[ic_cdk::init]
fn init(args: Option<CanisterArgs>) {
    match args {
        Some(CanisterArgs::Init(args)) => {
            store::state::with_mut(|space| {
                if !args.name.is_empty() {
                    space.name = args.name;
                }
                space.owner = args.owner;
                space.dao_canister = args.dao_canister;
                space.max_track_files = args.max_tracks;
                space.max_albums = args.max_albums;
                space.max_oss_data_size = args.max_oss_data_size;
                space.max_custom_data_size = args.max_custom_data_size;
                space.enable_search_index = args.enable_search_index;
                space.status = args.status;
                space.visibility = args.visibility;
                space.env = args.env;
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
            store::state::with_mut(|space| {
                if let Some(dao_canister) = args.dao_canister {
                    space.dao_canister = dao_canister;
                }
                if let Some(max_tracks) = args.max_tracks {
                    space.max_track_files = max_tracks;
                }
                if let Some(max_albums) = args.max_albums {
                    space.max_albums = max_albums;
                }
                if let Some(max_oss_data_size) = args.max_oss_data_size {
                    space.max_oss_data_size = max_oss_data_size;
                }
                if let Some(max_custom_data_size) = args.max_custom_data_size {
                    space.max_custom_data_size = max_custom_data_size;
                }
                if let Some(enable_search_index) = args.enable_search_index {
                    space.enable_search_index = enable_search_index;
                }
                if let Some(status) = args.status {
                    space.status = status;
                }
                if let Some(visibility) = args.visibility {
                    space.visibility = visibility;
                }
                if let Some(env) = args.env {
                    space.env = env;
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
