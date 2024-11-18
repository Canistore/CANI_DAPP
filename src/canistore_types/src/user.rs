use crate::ByteN;
use candid::{CandidType, Nat, Principal};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct NFT {
    pub canister_id: Principal,
    pub standard: String,
    pub token_index: String,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct Collection {
    pub canister_id: Principal,
    pub article_id: String,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct Attribute {
    pub key: String,
    pub value: String,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct SpaceArgs {
    pub name: String,
    pub desc: String,
    pub avatar: String,
    pub code: String,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum SpaceMsgType {
    Subscribe,
    Unsubscribe,
    Add,
    Remove,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct SpaceMsg {
    pub msg_type: SpaceMsgType,
    pub user: Principal,
    pub data: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct QueryCommonReq {
    pub page: Nat,
    pub size: Nat,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct QueryCollectionResp {
    pub page: Nat,
    pub total: u64,
    pub has_more: bool,
    pub data: Vec<Collection>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum MusicContentType {
    Singer,
    Band,
    GroupPerformer,
    SessionMusician,
    OrchestraMusician,
    Songwriter,
    MusicComposer,
    MusicProducer,
    StudioEngineer,
    RecordingMixingEngineer,
    MasteringEngineer,
    DJ,
}

#[allow(dead_code)]
impl MusicContentType {
    fn to_string(&self) -> &'static str {
        match self {
            MusicContentType::Singer => "Singer",
            MusicContentType::Band => "Band",
            MusicContentType::GroupPerformer => "Group Performer",
            MusicContentType::SessionMusician => "Session Musician",
            MusicContentType::OrchestraMusician => "Orchestra Musician",
            MusicContentType::Songwriter => "Songwriter",
            MusicContentType::MusicComposer => "Music Composer",
            MusicContentType::MusicProducer => "Music Producer",
            MusicContentType::StudioEngineer => "Studio Engineer",
            MusicContentType::RecordingMixingEngineer => "Music Recording / Mixing Engineer",
            MusicContentType::MasteringEngineer => "Mastering Engineer",
            MusicContentType::DJ => "DJ",
        }
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct User {
    pub avatar: String,
    pub artist_name: String,
    pub location: String,
    pub genre: String,
    pub website: String,
    pub bio: String,
    pub handler: String,
    pub music_content_type: Option<MusicContentType>,
    pub born: Option<u64>,
    pub nft: Option<NFT>,
    pub email: String,
    pub spaces: Vec<UserSpaceInfo>,
    pub subscribes: Vec<Principal>,
    pub collections: Vec<Collection>,
    pub attributes: Vec<Attribute>,
    pub confirm_agreement: bool,
    pub trusted_ecdsa_pub_key: Option<ByteBuf>,
    pub trusted_eddsa_pub_key: Option<ByteN<32>>,
    pub created: u64,
    pub updated_at: u64,
}

impl User {
    pub fn to_user_info(&self, pid: Principal) -> UserInfo {
        UserInfo {
            pid,
            avatar: self.avatar.clone(),
            nft: self.nft.clone(),
            email: self.email.clone(),
            spaces: self.spaces.clone(),
            artist_name: self.artist_name.clone(),
            location: self.location.clone(),
            genre: self.genre.clone(),
            website: self.website.clone(),
            bio: self.bio.clone(),
            handler: self.handler.clone(),
            music_content_type: self.music_content_type.clone(),
            born: self.born,
            confirm_agreement: self.confirm_agreement.clone(),
            trusted_ecdsa_pub_key: self.trusted_ecdsa_pub_key.clone(),
            trusted_eddsa_pub_key: self.trusted_eddsa_pub_key.clone(),
            created: self.created,
            updated_at: self.updated_at,
        }
    }

    pub fn new() -> Self {
        Self {
            avatar: String::from(""),
            artist_name: String::from(""),
            location: String::from(""),
            genre: String::from(""),
            website: String::from(""),
            bio: String::from(""),
            handler: String::from(""),
            music_content_type: None,
            born: None,
            nft: None,
            email: String::from(""),
            spaces: vec![],
            subscribes: vec![],
            collections: vec![],
            attributes: vec![],
            confirm_agreement: false,
            trusted_ecdsa_pub_key: None,
            trusted_eddsa_pub_key: None,
            created: ic_cdk::api::time(),
            updated_at: ic_cdk::api::time(),
        }
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct UserInfo {
    pub pid: Principal,
    pub avatar: String,
    pub nft: Option<NFT>,
    pub email: String,
    pub artist_name: String,
    pub location: String,
    pub genre: String,
    pub website: String,
    pub bio: String,
    pub spaces: Vec<UserSpaceInfo>,
    pub handler: String,
    pub music_content_type: Option<MusicContentType>,
    pub born: Option<u64>,
    pub confirm_agreement: bool,
    pub trusted_ecdsa_pub_key: Option<ByteBuf>,
    pub trusted_eddsa_pub_key: Option<ByteN<32>>,
    pub created: u64,
    pub updated_at: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct UpdateUserInfo {
    pub avatar: Option<String>,
    pub artist_name: Option<String>,
    pub location: Option<String>,
    pub genre: Option<String>,
    pub website: Option<String>,
    pub bio: Option<String>,
    pub handler: Option<String>,
    pub music_content_type: Option<MusicContentType>,
    pub born: Option<u64>,
    pub confirm_agreement: Option<bool>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct UserSpaceInfo {
    pub space_id: Principal,
    pub oss_id: Vec<Principal>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct UserInfoData {
    pub pid: Principal,
    pub avatar: String,
    pub nft: Option<NFT>,
    pub email: String,
    pub created: u64,
    pub spaces: UserSpaceInfo,
    pub subscribes: Vec<Principal>,
    pub collections: Vec<Collection>,
}
