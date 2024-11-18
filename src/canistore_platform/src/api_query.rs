use canistore_types::platform::MusicChannel;

use crate::store::{self, State};

#[ic_cdk::query]
fn get_platform_info() -> Result<State, String> {
    Ok(store::state::with(|state| State {
        name: state.name.clone(),
        owner: state.owner,
        ecdsa_key_name: state.ecdsa_key_name.clone(),
        ecdsa_token_public_key: state.ecdsa_token_public_key.clone(),
        token_expiration: state.token_expiration,
        next_channel_id: state.next_channel_id,
        space_count: state.space_count,
    }))
}

#[ic_cdk::query]
fn get_channel_info(id: u64) -> Result<MusicChannel, String> {
    let channel = store::channel::get_channel(id);
    match channel {
        None => Err("Music channel not found".to_string()),
        Some(channel) => Ok(channel.into_inner()),
    }
}

#[ic_cdk::query]
fn get_channel_list() -> Vec<MusicChannel> {
    store::channel::get_channel_list()
}
