use std::cmp::Ordering;

use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::time;
use serde::{Deserialize as SerdeDeserialize, Serialize};
use serde_json;

use crate::{
    space::AudioFile,
    user::{Attribute, MusicContentType, UserSpaceInfo},
};

#[derive(Debug, Clone, Eq)]
struct MessageId {
    timestamp: u64,
    internal_id: u64,
    canister_pid: String,
}

impl MessageId {
    fn new(internal_id: u64) -> Self {
        let timestamp = time();
        let canister_pid = ic_cdk::id().to_text();

        MessageId {
            timestamp,
            internal_id,
            canister_pid,
        }
    }

    fn to_string(&self) -> String {
        format!(
            "{}-{}-{}",
            self.timestamp, self.internal_id, self.canister_pid
        )
    }
}

impl PartialEq for MessageId {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp && self.internal_id == other.internal_id
    }
}

impl Ord for MessageId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp
            .cmp(&other.timestamp)
            .then_with(|| self.internal_id.cmp(&other.internal_id))
    }
}

impl PartialOrd for MessageId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub enum MessageType {
    Create,
    Update,
    Delete,
    Replace,
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct MessageSource {
    pub canister_id: Principal,
    pub resource_type: String,
    pub resource_id: u64,
}

#[derive(CandidType, Clone, Serialize, Debug, SerdeDeserialize)]
pub struct Message {
    pub msg_id: String,
    pub msg_type: MessageType,
    pub payload_type: String,
    pub payload: Vec<u8>,
    pub caller: Principal,
    pub msg_resource: Option<MessageSource>,
    pub timestamp: u64,
}

impl Message {
    pub fn new<T: Serialize>(
        internal_id: u64,
        msg_type: MessageType,
        payload_type: String,
        payload_data: T,
        msg_resource: Option<MessageSource>,
    ) -> Result<Self, String> {
        let message_id = MessageId::new(internal_id);
        let msg_id = message_id.to_string();

        let payload_json = serde_json::to_vec(&payload_data)
            .map_err(|e| format!("Failed to serialize payload: {:?}", e))?;

        Ok(Self {
            msg_id,
            msg_type,
            payload_type,
            payload: payload_json,
            caller: ic_cdk::caller(),
            msg_resource,
            timestamp: time(),
        })
    }

    pub fn decode_payload<T: for<'de> SerdeDeserialize<'de>>(&self) -> Result<T, String> {
        serde_json::from_slice::<T>(&self.payload)
            .map_err(|e| format!("Failed to deserialize payload: {:?}", e))
    }
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp
    }
}

impl Eq for Message {}

impl Ord for Message {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

impl PartialOrd for Message {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct MsgUserPost {
    pub space_id: Principal,
    pub user_pid: Principal,
    pub user_handler: String,
    pub content: String,
    pub created_at: u64,
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct MsgUserInfo {
    pub user_pid: Principal,
    pub avatar: String,
    pub artist_name: String,
    pub location: String,
    pub genre: String,
    pub website: String,
    pub bio: String,
    pub handler: String,
    pub music_content_type: Option<MusicContentType>,
    pub born: Option<u64>,
    pub email: String,
    pub spaces: Vec<UserSpaceInfo>,
    pub attributes: Vec<Attribute>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct MsgShareTrack {
    pub user_pid: Principal,
    pub name: String,
    pub audio_file: AudioFile,
    pub audio_url: String,
    pub cover_image: String,
    pub duration: Option<u64>,
    pub created: u64,
}
