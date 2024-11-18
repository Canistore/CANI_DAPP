use candid::{CandidType, Decode, Encode, Principal};

use canistore_types::{
    constant::Environment,
    error::{CustomError, ErrorCode},
    license::{License, LicenseRecord},
    message::{Message, MessageSource, MessageType},
    payment::{PaymentOrder, PaymentType, SubscriberInfo},
    space::{Album, Category, SharedTrack, Track, UserPost},
};
use ciborium::{from_reader, into_writer};
use ic_cdk_timers::TimerId;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    storable::Bound,
    DefaultMemoryImpl, StableBTreeMap, StableCell, Storable,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
};

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct Space {
    pub name: String,
    pub owner: Principal,
    pub max_track_files: u32,
    pub max_albums: u32,
    pub max_custom_data_size: u16,
    pub max_oss_data_size: u128,
    pub next_album_id: u64,
    pub next_track_id: u64,
    pub next_license_id: u64,
    pub next_order_id: u64,
    pub next_post_id: u64,
    pub next_message_id: u64,
    pub enable_search_index: bool,
    pub status: u8,                    // -1: archived; 0: active; 1: readonly
    pub visibility: u8,                // 0: private; 1: public
    pub managers: BTreeSet<Principal>, // Managers can read and write
    pub oss_canister: BTreeSet<Principal>,
    pub dao_canister: Principal,

    pub avatar: String,
    pub cover: String,
    pub desc: String,
    pub lang: String,
    pub created: u64,
    pub sub_prices: Vec<u64>,
    pub subscribers: Vec<SubscriberInfo>,
    pub followers: Vec<Principal>,
    pub total_subscribers: u64,
    pub total_albums: u64,
    pub total_tracks: u64,
    pub total_licenses: u64,
    pub total_income: u64,
    pub total_orders: u64,
    pub total_followers: u64,
    pub total_view: u64,
    pub total_post: u64,
    pub sended_message_id: u64,
    pub canister: Principal,
    pub categories: Vec<Category>,
    pub custom_url: String,
    pub services: Vec<String>,
    pub store_track_ids: Vec<u64>,
    pub env: Environment,
}

impl Default for Space {
    fn default() -> Self {
        Self {
            name: String::from("default_space"),
            owner: Principal::anonymous(), // Placeholder for a default owner
            max_track_files: 100_000,
            max_albums: 1_000,
            max_oss_data_size: 1024 * 1024 * 1024 * 1024, // 1TB
            max_custom_data_size: 1024 * 4,               // 4KB
            next_album_id: 0,
            next_track_id: 0,
            next_license_id: 0,
            next_order_id: 0,
            next_post_id: 0,
            next_message_id: 0,
            enable_search_index: false,
            status: 0,     // Active
            visibility: 0, // Private
            managers: BTreeSet::new(),
            oss_canister: BTreeSet::new(),
            dao_canister: Principal::anonymous(),

            avatar: String::from("default_avatar"),
            cover: String::from("default_cover"),
            desc: String::from("This is a default space description."),
            lang: String::from("en"),
            created: 0, // Or current timestamp if you want
            sub_prices: vec![],
            subscribers: vec![],
            followers: vec![],
            total_subscribers: 0,
            total_albums: 0,
            total_tracks: 0,
            total_licenses: 0,
            total_income: 0,
            total_orders: 0,
            total_followers: 0,
            total_view: 0,
            total_post: 0,
            sended_message_id: 0,
            canister: Principal::anonymous(),
            categories: vec![],
            custom_url: String::from("default_url"),
            services: vec![],
            store_track_ids: vec![],
            env: Environment::Test,
        }
    }
}

impl Space {
    pub fn write_permission(&self, caller: Principal) -> Result<(), String> {
        if caller == self.owner || self.managers.contains(&caller) {
            Ok(())
        } else {
            Err("Unauthorized".to_string())
        }
    }

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

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct SpaceInfo {
    pub name: String,
    pub owner: Principal,
    pub max_track_files: u32,
    pub max_albums: u32,
    pub max_custom_data_size: u16,
    pub max_oss_data_size: u128,
    pub status: u8,                    // -1: archived; 0: active; 1: readonly
    pub visibility: u8,                // 0: private; 1: public
    pub managers: BTreeSet<Principal>, // Managers can read and write
    pub oss_canister: BTreeSet<Principal>,
    pub dao_canister: Principal,

    pub avatar: String,
    pub cover: String,
    pub desc: String,
    pub lang: String,
    pub created: u64,
    pub sub_prices: Vec<u64>,
    pub subscribers: Vec<SubscriberInfo>,
    pub followers: Vec<Principal>,
    pub total_subscribers: u64,
    pub total_albums: u64,
    pub total_tracks: u64,
    pub total_licenses: u64,
    pub total_income: u64,
    pub total_orders: u64,
    pub total_followers: u64,
    pub total_view: u64,
    pub total_post: u64,
    pub canister: Principal,
    pub categories: Vec<Category>,
    pub custom_url: String,
    pub services: Vec<String>,
    pub store_track_ids: Vec<u64>,
    pub env: Environment,

    // show fields
    pub total_shares_album: u64,
    pub total_shares: u64,
}

impl SpaceInfo {
    pub fn from_space(space: &Space, total_shares_album: u64, total_shares: u64) -> Self {
        SpaceInfo {
            name: space.name.clone(),
            owner: space.owner,
            max_track_files: space.max_track_files,
            max_albums: space.max_albums,
            max_custom_data_size: space.max_custom_data_size,
            max_oss_data_size: space.max_oss_data_size,
            status: space.status,
            visibility: space.visibility,
            managers: space.managers.clone(),
            oss_canister: space.oss_canister.clone(),
            dao_canister: space.dao_canister,
            avatar: space.avatar.clone(),
            cover: space.cover.clone(),
            desc: space.desc.clone(),
            lang: space.lang.clone(),
            created: space.created,
            sub_prices: space.sub_prices.clone(),
            subscribers: space.subscribers.clone(),
            followers: space.followers.clone(),
            total_subscribers: space.total_subscribers,
            total_albums: space.total_albums,
            total_tracks: space.total_tracks,
            total_licenses: space.total_licenses,
            total_income: space.total_income,
            total_orders: space.total_orders,
            total_followers: space.total_followers,
            total_view: space.total_view,
            total_post: space.total_post,
            canister: space.canister,
            categories: space.categories.clone(),
            custom_url: space.custom_url.clone(),
            services: space.services.clone(),
            store_track_ids: space.store_track_ids.clone(),
            env: space.env.clone(),
            total_shares_album,
            total_shares,
        }
    }
}

impl Storable for Space {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> Cow<[u8]> {
        let mut buf = vec![];
        into_writer(self, &mut buf).expect("failed to encode Space data");
        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        from_reader(&bytes[..]).expect("failed to decode Space data")
    }
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct AlbumWrapper(pub Album);

impl Storable for AlbumWrapper {
    const BOUND: Bound = Bound::Unbounded;

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }
}

impl AlbumWrapper {
    pub fn into_inner(self) -> Album {
        self.0
    }
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct TrackWrapper(pub Track);

impl Storable for TrackWrapper {
    const BOUND: Bound = Bound::Unbounded;

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }
}

impl TrackWrapper {
    pub fn into_inner(self) -> Track {
        self.0
    }
}

#[derive(CandidType, Clone, Default, Deserialize, Serialize)]
pub struct LicenseMap(BTreeMap<Principal, License>);

impl LicenseMap {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn insert(&mut self, principal: Principal, license: License) {
        self.0.insert(principal, license);
    }

    pub fn get(&self, principal: &Principal) -> Option<&License> {
        self.0.get(principal)
    }
}

impl Storable for LicenseMap {
    const BOUND: Bound = Bound::Unbounded;

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }
}

#[derive(CandidType, Clone, Default, Deserialize, Serialize)]
pub struct LicenseRecordMap(BTreeMap<u128, LicenseRecord>);

impl Storable for LicenseRecordMap {
    const BOUND: Bound = Bound::Unbounded;

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct PaymentOrderWrapper(pub PaymentOrder);

impl Storable for PaymentOrderWrapper {
    const BOUND: Bound = Bound::Unbounded;

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }
}

impl PaymentOrderWrapper {
    pub fn into_inner(self) -> PaymentOrder {
        self.0
    }
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct SharedTrackWrapper(pub SharedTrack);

impl Storable for SharedTrackWrapper {
    const BOUND: Bound = Bound::Unbounded;

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }
}

impl SharedTrackWrapper {
    pub fn into_inner(self) -> SharedTrack {
        self.0
    }
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct UserPostWrapper(pub UserPost);

impl Storable for UserPostWrapper {
    const BOUND: Bound = Bound::Unbounded;

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }
}

impl UserPostWrapper {
    pub fn into_inner(self) -> UserPost {
        self.0
    }
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct MessageWrapper(pub Message);

impl Storable for MessageWrapper {
    const BOUND: Bound = Bound::Unbounded;

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }
}

const SPACE_MEMORY_ID: MemoryId = MemoryId::new(0);
const ALBUM_MEMORY_ID: MemoryId = MemoryId::new(1);
const TRACK_MEMORY_ID: MemoryId = MemoryId::new(2);
const LICENSE_MEMORY_ID: MemoryId = MemoryId::new(3);
const RECORD_MEMORY_ID: MemoryId = MemoryId::new(4);
const PAYMENT_MEMORY_ID: MemoryId = MemoryId::new(5);
const SHARE_MEMORY_ID: MemoryId = MemoryId::new(6);
const POST_MEMORY_ID: MemoryId = MemoryId::new(7);
const MESSAGE_MEMORY_ID: MemoryId = MemoryId::new(8);

thread_local! {
    static SPACE: RefCell<Space> = RefCell::new(Space::default());

    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static SPACE_STORE: RefCell<StableCell<Space, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(SPACE_MEMORY_ID)),
            Space::default()
        ).expect("failed to init SPACE_STORE store")
    );

    static ALBUM_STORE: RefCell<StableBTreeMap<u64, AlbumWrapper, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(ALBUM_MEMORY_ID)),
        )
    );

    static TRACK_STORE: RefCell<StableBTreeMap<u64, TrackWrapper, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(TRACK_MEMORY_ID)),
        )
    );

    static LICENSE_STORE: RefCell<StableBTreeMap<(u64, u64), LicenseMap, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(LICENSE_MEMORY_ID)),
        )
    );

    static RECORD_STORE: RefCell<StableBTreeMap<u64, LicenseRecordMap, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(RECORD_MEMORY_ID)),
        )
    );

    static PAYMENT_STORE: RefCell<StableBTreeMap<u64, PaymentOrderWrapper, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(PAYMENT_MEMORY_ID)),
        )
    );

    static SHARE_STORE: RefCell<StableBTreeMap<u64, SharedTrackWrapper, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(SHARE_MEMORY_ID)),
        )
    );

    static POST_STORE: RefCell<StableBTreeMap<u64, UserPostWrapper, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(POST_MEMORY_ID)),
        )
    );

    static MESSAGE_STORE: RefCell<StableBTreeMap<String, MessageWrapper, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(MESSAGE_MEMORY_ID)),
        )
    );

    pub static TIMER_IDS: RefCell<Vec<TimerId>> = RefCell::new(Vec::new());
}

pub mod state {
    use super::*;

    pub fn with<R>(f: impl FnOnce(&Space) -> R) -> R {
        SPACE.with(|r| f(&r.borrow()))
    }

    pub fn with_mut<R>(f: impl FnOnce(&mut Space) -> R) -> R {
        SPACE.with(|r| f(&mut r.borrow_mut()))
    }

    pub fn load() {
        SPACE_STORE.with(|r| {
            let s = r.borrow().get().clone();
            SPACE.with(|h| {
                *h.borrow_mut() = s;
            });
        });
    }

    pub fn save() {
        SPACE.with(|h| {
            SPACE_STORE.with(|r| {
                r.borrow_mut()
                    .set(h.borrow().clone())
                    .expect("failed to set SPACE_STORE data");
            });
        });
    }

    pub fn add_managers(new_managers: Vec<Principal>) {
        SPACE.with(|r| {
            let mut space = r.borrow_mut();
            for manager in new_managers {
                space.managers.insert(manager);
            }
        });
    }

    pub fn get_is_share_store(track_id: u64) -> bool {
        SPACE.with(|r| r.borrow().store_track_ids.contains(&track_id))
    }

    pub fn get_share_store_list(limit: usize, offset: usize) -> Vec<Track> {
        let store_track_ids: Vec<u64> = SPACE.with(|r| r.borrow().store_track_ids.clone());

        let paginated_track_ids: Vec<u64> = store_track_ids
            .iter()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect();

        track::get_tracks_by_ids(paginated_track_ids)
    }

    pub fn get_env() -> Environment {
        SPACE.with(|r| r.borrow().env.clone())
    }
}

pub mod album {
    use canistore_types::space::AlbumListEntry;

    use super::*;

    // pub fn total_albums() -> u64 {
    //     ALBUM_STORE.with(|r| r.borrow().len())
    // }

    pub fn get_album(id: u64) -> Option<AlbumWrapper> {
        ALBUM_STORE.with(|r| r.borrow().get(&id))
    }

    pub fn add_album(id: u64, album: Album) {
        ALBUM_STORE.with(|r| r.borrow_mut().insert(id, AlbumWrapper(album)));
    }

    pub fn delete_album(id: u64) -> bool {
        ALBUM_STORE.with(|r| r.borrow_mut().remove(&id)).is_some()
    }

    pub fn edit_album(id: u64, updated_album: Album) -> Result<(), String> {
        ALBUM_STORE.with(|r| {
            let mut store = r.borrow_mut();

            if store.contains_key(&id) {
                store.remove(&id);
                store.insert(id, AlbumWrapper(updated_album));
                Ok(())
            } else {
                Err(format!("Album with ID {} not found", id))
            }
        })
    }

    pub fn get_albums_list(limit: usize, offset: usize) -> Vec<AlbumListEntry> {
        ALBUM_STORE.with(|r| {
            let mut albums: Vec<AlbumListEntry> = r
                .borrow()
                .iter()
                .map(|(_, wrapper)| AlbumListEntry::from(&wrapper.0))
                .collect();

            // Sort albums by `created` in descending order
            albums.sort_by_key(|a| std::cmp::Reverse(a.created));

            // Return the albums with pagination
            albums.into_iter().skip(offset).take(limit).collect()
        })
    }

    pub fn add_track_ids_to_album(album_id: u64, track_ids: Vec<u64>) -> Result<(), String> {
        ALBUM_STORE.with(|r| {
            let mut store = r.borrow_mut();

            if let Some(album_wrapper) = store.get(&album_id) {
                let mut album = album_wrapper.clone();

                let new_track_ids: Vec<u64> = track_ids
                    .into_iter()
                    .filter(|id| !album.0.track_ids.contains(id))
                    .collect();

                album.0.track_ids.extend(new_track_ids);
                store.insert(album_id, album);
                Ok(())
            } else {
                Err(format!("Album with ID {} not found", album_id))
            }
        })
    }

    pub fn remove_track_ids_from_album(album_id: u64, track_ids: Vec<u64>) -> Result<(), String> {
        ALBUM_STORE.with(|r| {
            let mut store = r.borrow_mut();

            if let Some(album_wrapper) = store.get(&album_id) {
                let mut album = album_wrapper.clone();

                album.0.track_ids.retain(|id| !track_ids.contains(id));
                store.insert(album_id, album);
                Ok(())
            } else {
                Err(format!("Album with ID {} not found", album_id))
            }
        })
    }
}

pub mod track {
    use super::*;

    pub fn total_tracks() -> u64 {
        TRACK_STORE.with(|r| r.borrow().len())
    }

    pub fn get_track(id: u64) -> Option<TrackWrapper> {
        TRACK_STORE.with(|r| r.borrow().get(&id))
    }

    pub fn get_tracks_by_ids(ids: Vec<u64>) -> Vec<Track> {
        TRACK_STORE.with(|r| {
            let store = r.borrow();
            ids.iter()
                .filter_map(|id| store.get(id).map(|track_wrapper| track_wrapper.0.clone()))
                .collect()
        })
    }

    pub fn add_track(id: u64, track: Track) {
        TRACK_STORE.with(|r| r.borrow_mut().insert(id, TrackWrapper(track)));
    }

    pub fn edit_track(id: u64, updated_track: Track) -> Result<(), String> {
        TRACK_STORE.with(|r| {
            let mut store = r.borrow_mut();

            if store.contains_key(&id) {
                store.remove(&id);
                store.insert(id, TrackWrapper(updated_track));
                Ok(())
            } else {
                Err(format!("Track with ID {} not found", id))
            }
        })
    }

    pub fn delete_track(id: u64) -> Result<(), String> {
        TRACK_STORE.with(|r| {
            let mut store = r.borrow_mut();

            if let Some(track_wrapper) = store.get(&id) {
                let track = &track_wrapper.0;
                if track.album_id.is_some() && track.public_at > 0 {
                    return Err(
                        "Cannot delete a track that is part of an album and is public.".to_string(),
                    );
                }
                store.remove(&id);
                Ok(())
            } else {
                Err(format!("Track with ID {} not found.", id))
            }
        })
    }

    pub fn check_track_ids(ids: Vec<u64>) -> bool {
        TRACK_STORE.with(|r| {
            let store = r.borrow();
            for id in ids {
                if !store.contains_key(&id) {
                    return false;
                }
            }
            true
        })
    }

    pub fn update_track_field<F>(track_id: u64, update_fn: F) -> Result<(), String>
    where
        F: FnOnce(&mut Track),
    {
        TRACK_STORE.with(|r| {
            let mut store = r.borrow_mut();

            if let Some(track_wrapper) = store.get(&track_id) {
                let mut track = track_wrapper.clone();

                // Apply the update function to modify the track
                update_fn(&mut track.0);

                // Save the updated track back to the store
                store.insert(track_id, track);

                Ok(())
            } else {
                Err(format!("Track with ID {} not found", track_id))
            }
        })
    }

    pub fn set_track_album_id(track_id: u64, album_id: Option<u64>) -> Result<(), String> {
        update_track_field(track_id, |track| {
            track.album_id = album_id;
        })
    }

    pub fn set_track_cert(
        track_id: u64,
        cert_key: Option<String>,
        cert_hex: Option<String>,
    ) -> Result<(), String> {
        update_track_field(track_id, |track| {
            track.cert_key = cert_key;
            track.cert_hex = cert_hex;
        })
    }

    pub fn get_public_track_ids() -> Vec<u64> {
        TRACK_STORE.with(|r| {
            let store = r.borrow();
            store
                .iter()
                .filter_map(|(id, track_wrapper)| {
                    if track_wrapper.0.public_at > 0 {
                        Some(id)
                    } else {
                        None
                    }
                })
                .collect()
        })
    }

    pub fn set_track_public_status(track_id: u64, public_at: u64) -> Result<(), String> {
        TRACK_STORE.with(|r| {
            let mut store = r.borrow_mut();
            if let Some(track_wrapper) = store.get(&track_id) {
                let mut track = track_wrapper.clone();
                track.0.public_at = public_at;
                store.insert(track_id, track);
                Ok(())
            } else {
                Err(format!("Track with ID {} not found", track_id))
            }
        })
    }

    pub fn get_track_and_album(track_id: u64) -> Result<(Track, Album), String> {
        let track = get_track(track_id)
            .map(|track| track.into_inner())
            .ok_or_else(|| CustomError::new(ErrorCode::NoDataFound, Some("Track")).to_string())?;

        let album_id = track.album_id.ok_or_else(|| {
            CustomError::new(ErrorCode::DataNoAssociated, Some("Track and Album")).to_string()
        })?;

        let album = album::get_album(album_id)
            .map(|album| album.into_inner())
            .ok_or_else(|| CustomError::new(ErrorCode::NoDataFound, Some("Album")).to_string())?;

        Ok((track, album))
    }
}

pub mod license {
    use canistore_types::license::{
        AssetType, ChannelType, LicenseKey, LicenseListEntry, LicensedMedia, LicensedTerritory,
        RightPeriod, UsageRights,
    };
    use ic_cdk::api::time;

    use crate::SYSTEM_LICENSE_USER;

    use super::*;

    #[allow(dead_code)]
    pub fn total_licenses() -> u64 {
        LICENSE_STORE.with(|r| r.borrow().len())
    }

    pub fn get_license(id: (u64, u64)) -> Option<LicenseMap> {
        LICENSE_STORE.with(|r| r.borrow().get(&id))
    }

    pub fn get_license_by_track(track_id: u64) -> Option<LicenseMap> {
        let license_key = LicenseKey::new(None, Some(track_id));
        get_license(license_key.to_tuple())
    }

    pub fn get_track_genesis_license(track_id: u64) -> Option<License> {
        if let Some(license_map) = get_license_by_track(track_id) {
            license_map.0.get(&ic_cdk::id()).cloned()
        } else {
            None
        }
    }

    pub fn get_license_list(
        limit: usize,
        offset: usize,
        filter: Option<fn(&License) -> bool>,
    ) -> Vec<LicenseListEntry> {
        LICENSE_STORE.with(|r| {
            let store = r.borrow();
            let mut licenses: Vec<LicenseListEntry> = store
                .iter()
                .flat_map(|(_, license_map)| {
                    license_map
                        .0
                        .values()
                        // Apply the optional filter if provided, otherwise include all licenses
                        .filter(|license| match filter {
                            Some(filter_fn) => filter_fn(license),
                            None => true,
                        })
                        .map(LicenseListEntry::from)
                        .collect::<Vec<_>>()
                })
                .collect();

            // Sort licenses by `created` in descending order
            licenses.sort_by_key(|l| std::cmp::Reverse(l.created));

            // Return the licenses with pagination
            licenses.into_iter().skip(offset).take(limit).collect()
        })
    }

    pub fn get_all_licenses(limit: usize, offset: usize) -> Vec<LicenseListEntry> {
        get_license_list(limit, offset, None)
    }

    pub fn get_track_license_list(limit: usize, offset: usize) -> Vec<LicenseListEntry> {
        get_license_list(
            limit,
            offset,
            Some(|license| license.resource_key.album_id.unwrap_or(0) == 0),
        )
    }

    #[allow(dead_code)]
    pub fn get_album_license_list(limit: usize, offset: usize) -> Vec<LicenseListEntry> {
        get_license_list(
            limit,
            offset,
            Some(|license| license.resource_key.track_id.unwrap_or(0) == 0),
        )
    }

    pub fn delete_license(album_id: Option<u64>, track_id: Option<u64>) -> Result<(), String> {
        LICENSE_STORE.with(|r| {
            let mut store = r.borrow_mut();

            let license_key = LicenseKey::new(album_id, track_id);

            // Check if the license exists for the given resource
            if let Some(license_map) = store.get(&license_key.to_tuple()) {
                if !license_map.0.is_empty() {
                    store.remove(&license_key.to_tuple());
                    Ok(())
                } else {
                    Err(format!(
                        "No active licenses found for {}.",
                        match (album_id, track_id) {
                            (Some(album_id), None) => format!("album ID {}", album_id),
                            (None, Some(track_id)) => format!("track ID {}", track_id),
                            _ => "invalid resource".to_string(),
                        }
                    ))
                }
            } else {
                Err(format!(
                    "License for {} not found.",
                    match (album_id, track_id) {
                        (Some(album_id), None) => format!("album ID {}", album_id),
                        (None, Some(track_id)) => format!("track ID {}", track_id),
                        _ => "invalid resource".to_string(),
                    }
                ))
            }
        })
    }

    pub fn add_license(
        id: u64,
        album_id: Option<u64>,
        track_id: Option<u64>,
        user_pid: Principal,
        channel: ChannelType,
        asset_type: Vec<AssetType>,
        usage_rights: Vec<UsageRights>,
        licensed_media: Vec<LicensedMedia>,
        licensed_territory: Vec<LicensedTerritory>,
        right_period: Vec<RightPeriod>,
        fee: Option<u128>,
    ) {
        // Create a license key based on whether it's a track or an album
        let license_key = LicenseKey::new(album_id, track_id);

        // Load existing license map or create a new one if it doesn't exist
        let mut license_map =
            if let Some(existing_license_map) = get_license(license_key.to_tuple()) {
                existing_license_map
            } else {
                LicenseMap::new()
            };

        // Only add a new license if the user does not already have one for this key
        if !license_map.0.contains_key(&user_pid) {
            let new_license = License {
                id,
                user: user_pid,
                resource_key: license_key.clone(),
                start_time: time(),
                valid_duration: None, // None
                revoke_time: None,
                channel,
                asset_type,
                usage_rights,
                licensed_media,
                licensed_territory,
                right_period,
                fee,
                created: time(),
            };

            // Insert the new license into the map
            license_map.insert(user_pid, new_license);

            // Store the updated license map in the LICENSE_STORE
            LICENSE_STORE.with(|r| r.borrow_mut().insert(license_key.to_tuple(), license_map));
        } else {
            ic_cdk::println!("License already exists for user: {:?}", user_pid);
        }
    }

    pub fn remove_track_license(track_id: u64, user_pid: Principal) {
        let license_key = LicenseKey::new(None, Some(track_id));

        if let Some(mut license_map) = get_license(license_key.to_tuple()) {
            if license_map.0.contains_key(&user_pid) {
                license_map.0.remove(&user_pid);
                if license_map.0.is_empty() {
                    LICENSE_STORE.with(|r| r.borrow_mut().remove(&license_key.to_tuple()));
                } else {
                    LICENSE_STORE
                        .with(|r| r.borrow_mut().insert(license_key.to_tuple(), license_map));
                }

                ic_cdk::println!("License removed for user: {:?}", user_pid);
            } else {
                ic_cdk::println!("No license found for user: {:?}", user_pid);
            }
        } else {
            ic_cdk::println!("No licenses found for track_id: {:?}", track_id);
        }
    }

    pub fn add_track_license_for_platform(id: u64, track_id: u64) {
        let user_pid = SYSTEM_LICENSE_USER;
        add_license(
            id,
            None,
            Some(track_id),
            user_pid,
            ChannelType::Platform,
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            None,
        );
    }

    pub fn remove_track_license_for_platform(track_id: u64) {
        let user_pid = SYSTEM_LICENSE_USER;
        remove_track_license(track_id, user_pid);
    }
}

pub mod payment {
    use std::time::Duration;

    use crate::{
        pay::{token_balance, token_fee, token_transfer},
        utils::generate_order_subaccount,
    };

    use super::*;
    use crate::utils::check_page_size;
    use canistore_types::payment::{
        PaymentInfo, PaymentStatus, QueryCommonReq, QueryOrder, QuerySort,
    };
    use ic_cdk::api::time;
    use icrc_ledger_types::icrc1::account::Account;

    pub fn get_payment_order(id: u64) -> Option<PaymentOrder> {
        PAYMENT_STORE.with(|r| {
            let order_wrapper = r.borrow().get(&id);
            match order_wrapper {
                Some(order) => Some(order.into_inner()),
                None => None,
            }
        })
    }

    pub fn create_award_order(
        order_id: u64,
        payer: Principal,
        source: String,
        token: String,
        amount: u64,
        payment_type: PaymentType,
    ) -> PaymentInfo {
        let payment_order = PaymentOrder {
            id: order_id,
            payer,
            amount,
            payment_type: payment_type.clone(),
            source: source.clone(),
            token: token.clone(),
            amount_paid: 0,
            status: PaymentStatus::Unpaid,
            verified_time: None,
            shared_time: None,
            created_time: time(),
        };

        PAYMENT_STORE.with(|store| {
            store
                .borrow_mut()
                .insert(order_id, PaymentOrderWrapper(payment_order.clone()));
        });

        let recipient_subaccount = generate_order_subaccount(payer, order_id);

        PaymentInfo {
            id: order_id,
            recipient: recipient_subaccount,
            token,
            amount,
            payment_type,
            created_time: time(),
        }
    }

    pub fn limit_orders(caller: Principal, req: &QueryCommonReq) -> (usize, bool, Vec<QueryOrder>) {
        let mut data = Vec::new();
        let (page, size) = check_page_size(req.page, req.size);
        let start = (page - 1) * size;
        let mut has_more = false;
        let mut total = 0;

        // Collect all entries in a Vec
        let mut orders: Vec<(u64, PaymentOrderWrapper)> = PAYMENT_STORE
            .with(|store| store.borrow().iter().map(|(k, v)| (k, v.clone())).collect());

        // Sort or reverse based on the sort order
        match req.sort {
            QuerySort::TimeAsc => {}
            QuerySort::TimeDesc => {
                orders.reverse();
            }
        }

        // Iterate through the orders
        for (_idx, (_key, order_wrapper)) in orders.iter().enumerate() {
            let order = &order_wrapper.0;

            if order.payer != caller {
                continue;
            }

            if total >= start && total < start + size {
                data.push(QueryOrder::from_payment_order(
                    order.clone(),
                    generate_order_subaccount(order.payer, order.id),
                ));
            }
            total += 1;
        }

        if total > start + size {
            has_more = true;
        }

        (total, has_more, data)
    }

    pub async fn confirm_payment_order(order_id: u64) -> Result<bool, String> {
        let order = get_payment_order(order_id);

        let mut check_order = match order {
            Some(order) => order,
            None => return Err(format!("Order with id {} not found", order_id)),
        };

        // Verify the payment order
        let is_verified = verify_payment_order(&mut check_order).await;

        PAYMENT_STORE.with(|store| {
            let mut store = store.borrow_mut();
            // store.remove(&order_id);
            store.insert(order_id, PaymentOrderWrapper(check_order));
        });

        if is_verified {
            Ok(true)
        } else {
            Err("Order verification failed".to_string())
        }
    }

    pub async fn refund_payment_order(
        order_id: u64,
        caller: Principal,
        to: Vec<u8>,
    ) -> Result<bool, String> {
        // Fetch the payment order
        let order = get_payment_order(order_id);
        let mut check_order = match order {
            Some(order) => order,
            None => return Err(format!("Order with id {} not found", order_id)),
        };

        // Check ownership
        if check_order.payer != caller {
            return Err("Caller is not the owner of the order".to_string());
        }

        // Perform the refund
        let is_refunded = process_refund_payment_order(&mut check_order, to).await;

        // Update the payment store
        PAYMENT_STORE.with(|store| {
            let mut store = store.borrow_mut();
            store.insert(order_id, PaymentOrderWrapper(check_order));
        });

        if is_refunded {
            Ok(true)
        } else {
            Err("Refund process failed".to_string())
        }
    }

    async fn verify_payment_order(order: &mut PaymentOrder) -> bool {
        // If order is already paid, return true
        if order.status == PaymentStatus::Paid {
            return true;
        }

        // If order is neither unpaid nor verifying, return false
        if order.status != PaymentStatus::Unpaid {
            return false;
        }

        // Update the status to verifying
        order.status = PaymentStatus::Verifying;

        // Set timeout for 15 minutes (converted to nanoseconds)
        let fifteen_minutes = Duration::from_secs(15 * 60).as_nanos() as u64;
        if order.created_time + fifteen_minutes < time() {
            order.status = PaymentStatus::TimedOut;
            return false;
        }

        // Check the account balance based on the generated subaccount
        let subaccount: Option<[u8; 32]> = generate_order_subaccount(order.payer, order.id)
            .try_into()
            .ok();
        let payment_account = Account {
            owner: ic_cdk::id(),
            subaccount,
        };
        let amount_paid = token_balance(order.token.as_str(), payment_account).await;

        // Update the order with the paid amount
        order.amount_paid = amount_paid;

        // If the amount paid is less than the required amount, mark it as unpaid and return false
        if amount_paid == 0 || amount_paid < order.amount {
            order.status = PaymentStatus::Unpaid;
            return false;
        }

        // Mark the order as paid
        order.status = PaymentStatus::Paid;
        order.verified_time = Some(time());

        true
    }

    async fn process_refund_payment_order(order: &mut PaymentOrder, refund_to: Vec<u8>) -> bool {
        // If the order is already paid or refunded, return false
        if order.status == PaymentStatus::Paid || order.status == PaymentStatus::Refunded {
            return false;
        }

        if order.amount == 0 {
            return false;
        }

        // Generate subaccount for the payer
        let subaccount: Option<[u8; 32]> = generate_order_subaccount(order.payer, order.id)
            .try_into()
            .ok();

        let payer_account = Account {
            owner: order.payer,
            subaccount,
        };

        let balance = token_balance(&order.token, payer_account).await;
        order.amount_paid = balance;

        if balance < token_fee(&order.token) {
            return false;
        }

        // Calculate the refundable amount (balance minus fee)
        let amount_to_refund = balance - token_fee(&order.token);

        // Attempt to transfer the refund
        match token_transfer(&order.token, subaccount, refund_to, amount_to_refund).await {
            Ok(_) => {
                // Update order status to refunded
                order.status = PaymentStatus::Refunded;
                order.verified_time = Some(time());
                true
            }
            Err(_) => false, // If the transfer fails, return false
        }
    }
}

pub mod share {
    use canistore_types::space::SharedTrackListResp;

    use super::*;

    pub fn total_shares() -> u64 {
        SHARE_STORE.with(|r| r.borrow().len())
    }

    pub fn total_shares_album() -> u64 {
        let track_ids: Vec<u64> = SHARE_STORE.with(|r| {
            let store = r.borrow();
            store.iter().map(|(track_id, _)| track_id).collect()
        });

        let tracks = track::get_tracks_by_ids(track_ids);

        // Collect unique album IDs
        let unique_album_ids: std::collections::HashSet<u64> = tracks
            .into_iter()
            .filter_map(|track| track.album_id)
            .collect();

        unique_album_ids.len() as u64
    }

    pub fn get_share(track_id: u64) -> Option<SharedTrackWrapper> {
        SHARE_STORE.with(|r| r.borrow().get(&track_id))
    }

    // pub fn get_share_list() -> Vec<SharedTrack> {
    //     SHARE_STORE.with(|r| {
    //         let store = r.borrow();
    //         store
    //             .iter()
    //             .map(|(_, wrapper)| wrapper.into_inner().clone())
    //             .collect()
    //     })
    // }

    pub fn get_share_list(limit: usize, offset: usize) -> Vec<SharedTrackListResp> {
        let shared_tracks: Vec<SharedTrack> = SHARE_STORE.with(|r| {
            let store = r.borrow();
            store
                .iter()
                .skip(offset)
                .take(limit)
                .map(|(_, wrapper)| wrapper.into_inner().clone())
                .collect()
        });

        // Convert Vec<SharedTrack> to Vec<SharedTrackListResp>
        shared_tracks
            .into_iter()
            .filter_map(|shared_track| {
                if let Some(track_wrapper) = track::get_track(shared_track.track_id) {
                    let track = track_wrapper.into_inner().clone();
                    Some(SharedTrackListResp::new(shared_track, track))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn create_share(track_id: u64, shared_track: SharedTrack) -> Result<(), String> {
        SHARE_STORE.with(|r| {
            let mut store = r.borrow_mut();
            if store.contains_key(&track_id) {
                return Err(format!("Share for track ID {} already exists", track_id));
            }
            store.insert(track_id, SharedTrackWrapper(shared_track));
            Ok(())
        })
    }

    pub fn batch_create_share(
        track_ids: Vec<u64>,
        channel_id: u64,
        created_at: u64,
    ) -> Result<(), String> {
        SHARE_STORE.with(|r| {
            let mut store = r.borrow_mut();

            for track_id in track_ids {
                let shared_track = SharedTrack {
                    track_id,
                    channel_id,
                    created_at,
                };
                store.insert(track_id, SharedTrackWrapper(shared_track));
            }

            Ok(())
        })
    }

    pub fn delete_share(track_id: u64) -> Result<(), String> {
        SHARE_STORE.with(|r| {
            let mut store = r.borrow_mut();
            if store.remove(&track_id).is_none() {
                return Err(format!("Share for track ID {} does not exist", track_id));
            }
            Ok(())
        })
    }
}

pub mod post {
    use super::*;

    pub fn get_post(post_id: u64) -> Option<UserPostWrapper> {
        POST_STORE.with(|r| r.borrow().get(&post_id))
    }

    pub fn create_post(post_id: u64, user_post: UserPost) {
        POST_STORE.with(|r| {
            let mut store = r.borrow_mut();
            store.insert(post_id, UserPostWrapper(user_post));
        })
    }

    pub fn get_post_list(limit: usize, offset: usize, descending: bool) -> Vec<UserPost> {
        let mut user_posts: Vec<UserPost> = POST_STORE.with(|r| {
            let store = r.borrow();
            store
                .iter()
                .map(|(_, wrapper)| wrapper.into_inner().clone())
                .collect::<Vec<_>>()
        });
        if descending {
            user_posts.reverse();
        }

        user_posts.into_iter().skip(offset).take(limit).collect()
    }

    pub fn delete_post(post_id: u64) -> Result<(), String> {
        POST_STORE.with(|r| {
            let mut store = r.borrow_mut();
            if store.remove(&post_id).is_none() {
                return Err(format!("Post with ID {} does not exist", post_id));
            }
            Ok(())
        })
    }
}

pub mod message {
    use canistore_types::constant::CanisterType;

    use crate::canister_service::CanisterService;

    use super::*;

    pub fn create_message(message_id: &String, message: &Message) {
        MESSAGE_STORE.with(|r| {
            let mut store = r.borrow_mut();
            store.insert(message_id.clone(), MessageWrapper(message.clone()));
        })
    }

    pub async fn send_message<T: CandidType + Clone + Serialize>(
        message_type: MessageType,
        message_label: &str,
        message_id: u64,
        message_data: T,
        msg_resource: Option<MessageSource>,
    ) -> Result<String, String> {
        // Step 1: Create the message
        let msg = match Message::new(
            message_id,
            message_type,
            message_label.to_string(),
            message_data,
            msg_resource,
        ) {
            Ok(message) => message,
            Err(err) => return Err(format!("Failed to create message: {}", err)),
        };

        // Step 2: Define the indexer service
        let env = state::get_env();
        let indexer_service = CanisterService::init(&env, &CanisterType::Indexer)?;

        let result = indexer_service.receive_message(msg.clone()).await;

        match result {
            Ok((Ok(id),)) => {
                ic_cdk::print(format!("Message with ID {} was sent successfully.", id));
                Ok(id) // Return the message ID if sent successfully
            }
            Ok((Err(err_msg),)) => {
                ic_cdk::print(format!("Failed to send message: {}", err_msg));

                // Save the message locally if the send fails
                self::create_message(&msg.msg_id, &msg);
                Err(format!(
                    "Failed to send message: {}. Message saved locally.",
                    err_msg
                ))
            }
            Err(call_err) => {
                ic_cdk::print(format!(
                    "An error occurred during send_message: {:?}",
                    call_err
                ));

                // Save the message locally if the send fails
                self::create_message(&msg.msg_id, &msg);
                self::set_send_batch_messages_timer();
                Err(format!(
                    "Failed to call receive_message: {:?}. Message saved locally.",
                    call_err
                ))
            }
        }
    }

    // Get up to 50 messages from MESSAGE_STORE and send them, then delete the sent messages
    pub async fn send_batch_messages() -> Result<u64, String> {
        // Step 1: Retrieve up to 50 messages from the MESSAGE_STORE
        let messages_to_send = MESSAGE_STORE.with(|store| {
            let store_ref = store.borrow_mut();
            let mut messages_batch = Vec::new();
            let mut keys_to_delete = Vec::new();

            for (key, wrapper) in store_ref.iter().take(50) {
                messages_batch.push(wrapper.0.clone()); // MessageWrapper contains the message
                keys_to_delete.push(key.clone()); // Store the keys of messages to be deleted
            }

            (messages_batch, keys_to_delete)
        });

        let (messages, keys_to_delete) = messages_to_send;

        if messages.is_empty() {
            return Ok(0);
        }

        let env = state::get_env();
        let indexer_service = CanisterService::init(&env, &CanisterType::Indexer)?;

        // Step 2: Send the batch of messages using `receive_batch_messages`
        match indexer_service.receive_batch_messages(messages).await {
            Ok((Ok(sent_count),)) => {
                // Step 3: Delete successfully sent messages from MESSAGE_STORE
                MESSAGE_STORE.with(|store| {
                    let mut store_ref = store.borrow_mut();
                    for key in keys_to_delete {
                        store_ref.remove(&key);
                    }
                });
                Ok(sent_count)
            }
            Ok((Err(error),)) => Err(format!("Failed to send messages: {}", error)),
            Err((code, msg)) => Err(format!(
                "Failed to call canister method. Code: {:?}, Message: {:?}",
                code, msg
            )),
        }
    }

    pub fn set_send_batch_messages_timer() {
        let secs: Duration = Duration::from_secs(10);

        // Set the timer to trigger every 10 seconds
        let timer_id = ic_cdk_timers::set_timer(secs, move || {
            let send_task = async {
                match send_batch_messages().await {
                    Ok(sent_count) => {
                        ic_cdk::print(format!("Successfully sent {} messages", sent_count));
                    }
                    Err(err) => {
                        ic_cdk::print(format!(
                            "Failed to send batch messages: {}. Retrying...",
                            err
                        ));
                        // Re-schedule the timer to retry
                        set_send_batch_messages_timer();
                    }
                }
            };
            ic_cdk::spawn(send_task);
        });

        // Store the timer ID for potential future use (like canceling)
        TIMER_IDS.with(|timer_ids| timer_ids.borrow_mut().push(timer_id));
    }
}
