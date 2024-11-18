use candid::{self, Principal};
use canistore_types::{
    certificate::{MusicCertificate, MusicCertificateResp},
    constant::{CanisterType, Environment},
    dao::DaoStateInfo,
    message::Message,
    platform::TrackInfo,
};
use ic_cdk::api::call::CallResult;

// Generic CanisterService for any CanisterType
pub struct CanisterService {
    pub principal: Principal,
}

impl CanisterService {
    // Initialization method to create a service instance by retrieving the canister PID from the environment
    pub fn init(env: &Environment, canister_type: &CanisterType) -> Result<Self, String> {
        let pid = Principal::from_text(env.get_canister_pid(canister_type.clone()))
            .map_err(|_| format!("Invalid {:?} PID", canister_type))?;
        Ok(Self { principal: pid })
    }
}

// DAO-specific methods
#[allow(dead_code)]
impl CanisterService {
    pub async fn get_dao_info(&self) -> CallResult<(DaoStateInfo,)> {
        ic_cdk::call(self.principal, "get_dao_info", ()).await
    }

    pub async fn store_certificate(
        &self,
        music_certificate: MusicCertificate,
    ) -> CallResult<(Result<MusicCertificateResp, String>,)> {
        ic_cdk::call(self.principal, "store_certificate", (music_certificate,)).await
    }
}

// Platform-specific methods
#[allow(dead_code)]
impl CanisterService {
    pub async fn add_track_to_channel(
        &self,
        channel_id: u64,
        track_info: TrackInfo,
    ) -> CallResult<(Result<(), String>,)> {
        ic_cdk::call(
            self.principal,
            "add_track_to_channel",
            (channel_id, track_info),
        )
        .await
    }

    pub async fn delete_track_from_channel(
        &self,
        channel_id: u64,
        track_id: u64,
    ) -> CallResult<(Result<(), String>,)> {
        ic_cdk::call(
            self.principal,
            "delete_track_from_channel",
            (channel_id, track_id),
        )
        .await
    }

    pub async fn delete_track_from_channel_by_share(
        &self,
        channel_id: u64,
        space_canister_id: Principal,
        track_id: u64,
    ) -> CallResult<(Result<(), String>,)> {
        ic_cdk::call(
            self.principal,
            "delete_track_from_channel_by_share",
            (channel_id, space_canister_id, track_id),
        )
        .await
    }

    pub async fn batch_add_tracks_to_channel(
        &self,
        channel_id: u64,
        tracks: Vec<TrackInfo>,
    ) -> CallResult<(Result<(), String>,)> {
        ic_cdk::call(
            self.principal,
            "batch_add_tracks_to_channel",
            (channel_id, tracks),
        )
        .await
    }
}

// Indexer-specific methods
impl CanisterService {
    pub async fn receive_message(&self, msg: Message) -> CallResult<(Result<String, String>,)> {
        ic_cdk::call(self.principal, "receive_message", (msg,)).await
    }

    pub async fn receive_batch_messages(
        &self,
        messages: Vec<Message>,
    ) -> CallResult<(Result<u64, String>,)> {
        ic_cdk::call(self.principal, "receive_batch_messages", (messages,)).await
    }
}
