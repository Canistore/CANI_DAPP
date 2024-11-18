use crate::ecdsa;
use candid::{CandidType, Decode, Encode, Principal};
use canistore_types::{cose::PLATFORM_TOKEN_AAD, platform::MusicChannel};
use ciborium::{from_reader, into_writer};
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    storable::Bound,
    DefaultMemoryImpl, StableBTreeMap, StableCell, Storable,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct State {
    pub name: String,
    pub owner: Principal,
    pub space_count: u128,
    pub next_channel_id: u64,
    pub ecdsa_key_name: String,
    pub ecdsa_token_public_key: String,
    pub token_expiration: u64, // in seconds
}

impl Default for State {
    fn default() -> Self {
        Self {
            name: String::from("Canistore Platform"),
            owner: Principal::anonymous(),
            ecdsa_key_name: String::from("canistore_test_key"),
            ecdsa_token_public_key: String::from(""),
            token_expiration: 0,
            space_count: 0,
            next_channel_id: 0,
        }
    }
}

impl State {
    pub fn owner_permission(&self, caller: Principal) -> Result<(), String> {
        if caller == self.owner {
            Ok(())
        } else {
            Err("Unauthorized".to_string())
        }
    }

    pub fn controller_or_owner_permission(&self, caller: Principal) -> Result<(), String> {
        if caller == self.owner || ic_cdk::api::is_controller(&caller) {
            Ok(())
        } else {
            Err("Unauthorized".to_string())
        }
    }
}

impl Storable for State {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> Cow<[u8]> {
        let mut buf = vec![];
        into_writer(self, &mut buf).expect("failed to encode User Center data");
        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        from_reader(&bytes[..]).expect("failed to decode User Center data")
    }
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct MusicChannelWrapper(pub MusicChannel);

impl Storable for MusicChannelWrapper {
    const BOUND: Bound = Bound::Unbounded;

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }
}

impl MusicChannelWrapper {
    pub fn into_inner(self) -> MusicChannel {
        self.0
    }
}

const STATE_MEMORY_ID: MemoryId = MemoryId::new(0);
const CHANNEL_MEMORY_ID: MemoryId = MemoryId::new(1);

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());

    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static STATE_STORE: RefCell<StableCell<State, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(STATE_MEMORY_ID)),
            State::default()
        ).expect("failed to init STATE_STORE")
    );

    static CHANNEL_STORE: RefCell<StableBTreeMap<u64, MusicChannelWrapper, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(CHANNEL_MEMORY_ID)),
        )
    );
}

pub mod state {
    use super::*;

    pub fn with<R>(f: impl FnOnce(&State) -> R) -> R {
        STATE.with(|r| f(&r.borrow()))
    }

    pub fn with_mut<R>(f: impl FnOnce(&mut State) -> R) -> R {
        STATE.with(|r| f(&mut r.borrow_mut()))
    }

    pub fn load() {
        STATE_STORE.with(|r| {
            let s = r.borrow().get().clone();
            STATE.with(|h| {
                *h.borrow_mut() = s;
            });
        });
    }

    pub fn save() {
        STATE.with(|h| {
            STATE_STORE.with(|r| {
                r.borrow_mut()
                    .set(h.borrow().clone())
                    .expect("failed to save User Center data");
            });
        });
    }

    pub async fn init_ecdsa_public_key() -> Result<(), String> {
        let ecdsa_key_name = with(|r| {
            if r.ecdsa_token_public_key.is_empty() && !r.ecdsa_key_name.is_empty() {
                Some(r.ecdsa_key_name.clone())
            } else {
                None
            }
        });

        if let Some(ecdsa_key_name) = ecdsa_key_name {
            let pk = ecdsa::public_key_with(&ecdsa_key_name, vec![PLATFORM_TOKEN_AAD.to_vec()])
                .await
                .map_err(|err| format!("failed to retrieve ECDSA public key: {err}"))?;

            with_mut(|r| {
                r.ecdsa_token_public_key = hex::encode(pk.public_key);
            });
        }

        Ok(())
    }
}

pub mod channel {
    use canistore_types::platform::TrackInfo;

    use super::*;

    pub fn get_channel(channel_id: u64) -> Option<MusicChannelWrapper> {
        CHANNEL_STORE.with(|r| r.borrow().get(&channel_id))
    }

    pub fn add_channel(id: u64, channel: MusicChannel) {
        CHANNEL_STORE.with(|r| r.borrow_mut().insert(id, MusicChannelWrapper(channel)));
    }

    pub fn get_channel_list() -> Vec<MusicChannel> {
        CHANNEL_STORE.with(|r| {
            let store = r.borrow();
            store
                .iter()
                .map(|(_, wrapper)| wrapper.into_inner().clone())
                .collect()
        })
    }

    // Add a track to the specified channel
    pub fn add_track_to_channel(channel_id: u64, track: TrackInfo) -> Result<(), String> {
        CHANNEL_STORE.with(|r| {
            let mut store = r.borrow_mut();
            if let Some(channel_wrapper) = store.get(&channel_id) {
                let mut channel = channel_wrapper.0.clone();
                channel.add_track(track);
                store.insert(channel_id, MusicChannelWrapper(channel));
                Ok(())
            } else {
                Err("Channel not found".to_string())
            }
        })
    }

    // Delete a track from the specified channel by its position
    pub fn delete_track_from_channel(channel_id: u64, position: u64) -> Result<(), String> {
        CHANNEL_STORE.with(|r| {
            let mut store = r.borrow_mut();
            if let Some(channel_wrapper) = store.get(&channel_id) {
                let mut channel = channel_wrapper.0.clone();
                channel.delete_track(position)?;
                store.insert(channel_id, MusicChannelWrapper(channel));
                Ok(())
            } else {
                Err("Channel not found".to_string())
            }
        })
    }

    // Delete a track from the specified channel by its OssFileInfo (space_canister_id and track_id)
    pub fn delete_track_from_channel_by_share(
        channel_id: u64,
        space_canister_id: Principal,
        track_id: u64,
    ) -> Result<(), String> {
        CHANNEL_STORE.with(|r| {
            let mut store = r.borrow_mut();
            if let Some(channel_wrapper) = store.get(&channel_id) {
                let mut channel = channel_wrapper.0.clone();
                channel
                    .delete_track_oss_file(space_canister_id, track_id)
                    .map_err(|err| format!("Failed to delete track: {}", err))?;
                store.insert(channel_id, MusicChannelWrapper(channel));
                Ok(())
            } else {
                Err("Channel not found".to_string())
            }
        })
    }
}
