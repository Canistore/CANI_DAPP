use ic_cdk::api::time;
use std::cmp::Ordering;

use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

use crate::{
    constant::Environment,
    message::{MessageSource, MsgShareTrack, MsgUserPost},
    payment::{SubscriberInfo, SubscriptionPrice},
    user::Attribute,
};

const DEFAULT_OSS_MAX_FILE_SIZE: u64 = 300 * 1024 * 1024 * 1024;
pub const SPACE_FEE: u64 = 10_000;

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum CanisterArgs {
    Init(SpaceInitArgs),
    Upgrade(SpaceUpgradeArgs),
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum OssCanisterArgs {
    Init(OssInitArgs),
    Upgrade(OssUpgradeArgs),
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct SpaceOssCanisterArgs {
    pub space_arg: Option<CanisterArgs>,
    pub oss_arg: OssCanisterArgs,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct SpaceInitArgs {
    pub name: String,              // Space name
    pub owner: Principal,          // Owner
    pub dao_canister: Principal,   // DAO_CANISTER
    pub max_tracks: u32,           // Maximum number of music files
    pub max_albums: u32,           // Maximum number of albums
    pub max_oss_data_size: u128,   // Maximum oss data size in bytes
    pub max_custom_data_size: u16, // Maximum custom data size in bytes
    pub enable_search_index: bool, // Enable search indexing
    pub status: u8,                // -1: archived; 0: active; 1: readonly
    pub visibility: u8,            // 0: private; 1: public
    pub env: Environment,
}

impl Default for SpaceInitArgs {
    fn default() -> Self {
        SpaceInitArgs {
            name: String::from("Canistore Space"), // Default space name
            owner: Principal::anonymous(),         // Default to anonymous principal
            dao_canister: Principal::anonymous(),  // Default to anonymous DAO canister
            max_tracks: 50000,                     // Default maximum tracks
            max_albums: 3000,                      // Default maximum albums
            max_oss_data_size: 1024 * 1024 * 1024 * 1024, // Maximum oss data size in bytes
            max_custom_data_size: 4096,            // Default custom data size
            enable_search_index: true,             // Enable search indexing by default
            status: 1,                             // Default status is active
            visibility: 1,                         // Default visibility is public
            env: Environment::Test,
        }
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct SpaceUpgradeArgs {
    pub dao_canister: Option<Principal>,
    pub max_tracks: Option<u32>,
    pub max_albums: Option<u32>,
    pub max_oss_data_size: Option<u128>,
    pub max_custom_data_size: Option<u16>,
    pub enable_search_index: Option<bool>,
    pub status: Option<u8>,
    pub visibility: Option<u8>,
    pub env: Option<Environment>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum PermissionType {
    Owner,
    Creator,
    None,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum MusicCategory {
    Rock,
    PopMusic,
    CountryMusic,
    ClassicalMusic,
    PopularMusic,
    Blues,
    ElectronicMusic,
    RhythmAndBlues,
    FolkMusic,
    HeavyMetal,
    Jazz,
    HipHopMusic,
    AlternativeRock,
    SynthPop,
    WorldMusic,
    Funk,
    PunkRock,
    Reggae,
    ProgressiveRock,
    DanceMusic,
    Disco,
    NewAgeMusic,
    IndieRock,
    ExperimentalMusic,
    NewWave,
    SoulMusic,
    Ska,
    Singing,
    ChristianMusic,
    Modernism,
    ElectronicDanceMusic,
    EasyListening,
    HardRock,
    Techno,
    Grunge,
    Emo,
    HouseMusic,
    Dubstep,
    LatinMusic,
    KPop,
    Metal,
    Britpop,
    MusicOfLatinAmerica,
    TranceMusic,
    Flamenco,
    SwingMusic,
    IndianClassicalMusic,
    Bachata,
    VocalMusic,
    MusicOfAfrica,
    Other,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct Category {
    pub id: u32,
    pub category_type: MusicCategory,
    pub desc: String,
    pub parent: Option<u32>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, PartialOrd)]
pub enum AlbumStatus {
    Draft,
    Public,
    Subscription,
    Private,
    Deleted,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, PartialOrd)]
pub enum AlbumType {
    Album,
    Single,
    EP,
    Playlist,
    Channel,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct Album {
    pub id: u64,
    pub album_type: AlbumType,
    pub title: String,
    pub artist: String,
    pub cover_image: String,
    pub producer: Principal,
    pub description: String,
    pub category: MusicCategory,
    pub sub_category: Option<MusicCategory>,
    pub status: AlbumStatus,
    pub allow_comments: bool,
    pub likes: u32,
    pub dislikes: u32,
    pub plays: u64,
    pub comments: u32,
    pub is_original: bool,
    pub external_link: String,
    pub tags: Vec<String>,
    pub version: u32,
    pub language: String,
    pub release_at: Option<u64>,
    pub copyright: Option<String>,
    pub track_ids: Vec<u64>,
    pub memory_usage: u64,

    pub subscription_prices: Vec<SubscriptionPrice>,
    pub subscriber_count: u32,
    pub subscribers: Vec<SubscriberInfo>,
    pub album_stat: AlbumStat,

    pub created: u64,
    pub updated: u64,
    pub toped: u64,
}

impl Album {
    pub fn new(
        id: u64,
        album_type: AlbumType,
        title: String,
        artist: String,
        cover_image: String,
        producer: Principal,
        description: String,
        category: MusicCategory,
        sub_category: Option<MusicCategory>,
        is_original: bool,
        external_link: String,
        tags: Vec<String>,
        language: String,
        release_at: Option<u64>,
        copyright: Option<String>,
        subscription_prices: Vec<SubscriptionPrice>,
    ) -> Self {
        let current_time = time();

        Album {
            id,
            album_type,
            title,
            artist,
            cover_image,
            producer,
            description,
            category,
            sub_category,
            status: AlbumStatus::Public, // default status
            allow_comments: true,        // always true
            likes: 0,                    // default value
            dislikes: 0,                 // default value
            plays: 0,                    // default value
            comments: 0,                 // default value
            version: 1,                  // initial version
            is_original,
            external_link,
            tags,
            language,
            release_at,
            copyright,
            track_ids: vec![],
            memory_usage: 0,
            subscription_prices,
            subscriber_count: 0,
            subscribers: vec![],
            album_stat: AlbumStat::default(),
            created: current_time,
            updated: current_time,
            toped: 0,
        }
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct CreateAlbumArg {
    pub artist: String,
    pub album_type: AlbumType,
    pub title: String,
    pub cover_image: String,
    pub producer: Option<Principal>,
    pub description: String,
    pub category: MusicCategory,
    pub sub_category: Option<MusicCategory>,
    pub is_original: bool,
    pub external_link: String,
    pub tags: Vec<String>,
    pub language: String,
    pub release_at: Option<u64>,
    pub copyright: Option<String>,
    pub subscription_prices: Vec<SubscriptionPrice>,
}

#[derive(CandidType, Deserialize, Default)]
pub struct EditAlbumArg {
    pub artist: Option<String>,
    pub album_type: Option<AlbumType>,
    pub title: Option<String>,
    pub cover_image: Option<String>,
    pub producer: Option<Principal>,
    pub description: Option<String>,
    pub category: Option<MusicCategory>,
    pub sub_category: Option<MusicCategory>,
    pub is_original: Option<bool>,
    pub external_link: Option<String>,
    pub tags: Option<Vec<String>>,
    pub language: Option<String>,
    pub release_at: Option<u64>,
    pub copyright: Option<String>,
    pub subscription_prices: Option<Vec<SubscriptionPrice>>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct AlbumListEntry {
    pub id: u64,
    pub title: String,
    pub cover_image: String,
    pub producer: Principal,
    pub description: String,
    pub category: MusicCategory,
    pub sub_category: Option<MusicCategory>,
    pub release_at: Option<u64>,
    pub track_count: usize,
    pub created: u64,
    pub updated: u64,
}

impl From<&Album> for AlbumListEntry {
    fn from(album: &Album) -> Self {
        AlbumListEntry {
            id: album.id,
            title: album.title.clone(),
            cover_image: album.cover_image.clone(),
            producer: album.producer,
            description: album.description.clone(),
            category: album.category.clone(),
            sub_category: album.sub_category.clone(),
            release_at: album.release_at,
            track_count: album.track_ids.len(),
            created: album.created,
            updated: album.updated,
        }
    }
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct AudioFile {
    pub canister_id: Principal,
    pub file_id: u32,
}

impl Default for AudioFile {
    fn default() -> Self {
        AudioFile {
            canister_id: Principal::anonymous(),
            file_id: 0,
        }
    }
}

impl AudioFile {
    pub fn url(&self) -> String {
        format!(
            "https://{}.icp0.io/f/{}",
            self.canister_id.to_text(),
            self.file_id
        )
    }
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct Track {
    pub id: u64,
    pub name: String,
    pub collaborators: Vec<String>,     // Collaborators
    pub versions: Option<TrackVersion>, // List of version names or identifiers
    pub audio_file: AudioFile,          // Reference to an audio file ID
    pub artist: Principal,              // artist as a Principal ID
    pub songwriter: bool,               // Original/Secondary Creation
    pub is_explicit_lyrics: bool,       // Whether the track has explicit lyrics
    pub is_radio_edition: bool,         // Whether the track is a radio edition
    pub instrumental: bool,             // Whether the track is instrumental
    pub duration: Option<u64>,
    pub album_id: Option<u64>,
    pub public_at: u64,
    pub file_size: u64,
    pub file_format: String,
    pub cert_key: Option<String>,
    pub cert_hex: Option<String>,
    pub attributes: Vec<Attribute>,

    pub created: u64,
    pub updated: u64,
}

impl Track {
    pub fn to_msg(
        &self,
        user_pid: Principal,
        cover_image: String,
    ) -> (MsgShareTrack, Option<MessageSource>) {
        (
            MsgShareTrack {
                user_pid,
                name: self.name.clone(),
                audio_file: self.audio_file.clone(),
                audio_url: self.audio_file.url(),
                cover_image,
                duration: self.duration,
                created: self.created,
            },
            Some(MessageSource {
                canister_id: ic_cdk::id(),
                resource_type: String::from("Track"),
                resource_id: self.id,
            }),
        )
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum TrackVersion {
    Live,
    Remix,
    Radio,
    Orginal,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct CreateTrackArg {
    pub name: String,
    pub collaborators: Vec<String>,
    pub versions: Option<TrackVersion>,
    pub audio_file: AudioFile,
    pub songwriter: bool,
    pub is_explicit_lyrics: bool,
    pub is_radio_edition: bool,
    pub instrumental: bool,
    pub duration: Option<u64>,
    pub file_size: u64,
    pub file_format: String,
}

#[derive(CandidType, Deserialize, Default)]
pub struct EditTrackArg {
    pub name: Option<String>,
    pub collaborators: Option<Vec<String>>,
    pub versions: Option<TrackVersion>,
    pub audio_file: Option<AudioFile>,
    pub songwriter: Option<bool>,
    pub is_explicit_lyrics: Option<bool>,
    pub is_radio_edition: Option<bool>,
    pub instrumental: Option<bool>,
    pub duration: Option<u64>,
    pub file_size: Option<u64>,
    pub file_format: Option<String>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum CommentStatus {
    Invisible,
    Visible,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct Comment {
    pub id: u32,
    pub album_id: String,
    pub owner: Principal,
    pub commenter_id: Principal,
    pub content: String,
    pub created: u64,
    pub likes: u32,
    pub status: CommentStatus,
    pub reply: Option<Box<Comment>>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct SortedAlbum {
    pub id: String,
    pub toped: u64,
    pub created: u64,
}

impl Album {
    pub fn before_created(a: &Album, b: &Album) -> bool {
        if b.toped > 0 {
            false
        } else {
            a.created > b.created
        }
    }
}

impl SortedAlbum {
    pub fn compare_albums_desc(a: &SortedAlbum, b: &SortedAlbum) -> Ordering {
        if a.toped > 0 && b.toped > 0 {
            // If both albums are "toped", compare them by `toped` in descending order
            b.toped.cmp(&a.toped)
        } else if a.toped > 0 && b.toped == 0 {
            // If only `a` is "toped", `a` should come before `b`
            Ordering::Less
        } else if b.toped > 0 && a.toped == 0 {
            // If only `b` is "toped", `b` should come before `a`
            Ordering::Greater
        } else {
            // If neither is "toped", compare them by `created` in descending order
            b.created.cmp(&a.created)
        }
    }
}

impl Default for Track {
    fn default() -> Self {
        let current_time = time();
        Self {
            id: 0,
            name: String::new(),
            collaborators: vec![],
            versions: None,
            audio_file: AudioFile::default(),
            artist: Principal::anonymous(),
            songwriter: true,
            is_explicit_lyrics: false,
            is_radio_edition: false,
            instrumental: false,
            album_id: None,
            duration: None,
            public_at: 0,
            file_size: 0,
            file_format: String::new(),
            cert_key: None,
            cert_hex: None,
            attributes: vec![],

            created: current_time,
            updated: current_time,
        }
    }
}

impl Album {
    /// Adds a track ID to the end of the album.
    pub fn add_track(&mut self, track_id: u64) {
        self.track_ids.push(track_id);
    }

    /// Inserts a track ID at a specific position. Returns an error if the position is out of bounds.
    pub fn insert_track(&mut self, index: usize, track_id: u64) -> Result<(), String> {
        if index > self.track_ids.len() {
            return Err(format!(
                "Insert position {} is out of bounds for track_ids of length {}",
                index,
                self.track_ids.len()
            ));
        }
        self.track_ids.insert(index, track_id);
        Ok(())
    }

    /// Removes a track ID at a specific position. Returns the removed track ID or an error if the position is invalid.
    pub fn remove_track(&mut self, index: usize) -> Result<u64, String> {
        if index >= self.track_ids.len() {
            return Err(format!(
                "Remove position {} is out of bounds for track_ids of length {}",
                index,
                self.track_ids.len()
            ));
        }
        Ok(self.track_ids.remove(index))
    }

    /// Moves a track ID from one position to another. Returns an error if any position is invalid.
    pub fn move_track(&mut self, from: usize, to: usize) -> Result<(), String> {
        if from >= self.track_ids.len() || to > self.track_ids.len() {
            return Err(format!(
                "Move positions out of bounds: from {} to {} for track_ids of length {}",
                from,
                to,
                self.track_ids.len()
            ));
        }
        if from == to || from + 1 == to {
            // No change needed
            return Ok(());
        }
        let track_id = self.track_ids.remove(from);
        self.track_ids.insert(to, track_id);
        Ok(())
    }

    /// Returns the number of tracks in the album.
    pub fn track_count(&self) -> usize {
        self.track_ids.len()
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct BlacklistedUser {
    pub pid: Principal,
    pub created: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct QueryCategory {
    pub id: u32,
    pub name: String,
    pub children: Vec<QueryCategory>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct OpResult {
    pub result: Result<String, String>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct AlbumStat {
    pub total: u32,
    pub public_count: u32,
    pub private_count: u32,
    pub subscription_count: u32,
    pub draft_count: u32,
}

impl Default for AlbumStat {
    fn default() -> Self {
        AlbumStat {
            total: 0,
            public_count: 0,
            private_count: 0,
            subscription_count: 0,
            draft_count: 0,
        }
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct AlbumArgs {
    pub id: String,
    pub album_type: AlbumType,
    pub title: String,
    pub cover_image: String,
    pub description: String,
    pub category: u32,
    pub sub_category: u32,
    pub status: AlbumStatus,
    pub allow_comments: bool,
    pub is_original: bool,
    pub external_link: String,
    pub tags: Vec<String>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct QueryAlbum {
    pub id: String,
    pub album_type: AlbumType,
    pub title: String,
    pub cover_image: String,
    pub producer: Principal,
    pub description: String,
    pub category: u32,
    pub sub_category: u32,
    pub created: u64,
    pub updated: u64,
    pub toped: u64,
    pub status: AlbumStatus,
    pub allow_comments: bool,
    pub likes: u32,
    pub dislikes: u32,
    pub plays: u64,
    pub comment_count: u32,
    pub new_comment_count: u32,
    pub is_original: bool,
    pub external_link: String,
    pub tags: Vec<String>,
    pub copyright: Option<String>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct QueryDetailResp {
    pub result: Result<QueryAlbum, String>,
    pub content: String,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum QuerySort {
    TimeDesc,
    TimeAsc,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct QueryAlbumReq {
    pub page: u32,
    pub size: u32,
    pub category: u32,
    pub sub_category: u32,
    pub status: Option<AlbumStatus>,
    pub album_type: Option<AlbumType>,
    pub search: String,
    pub sort: QuerySort,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct QueryAlbumResp {
    pub page: u32,
    pub total: u32,
    pub has_more: bool,
    pub stat: AlbumStat,
    pub data: Vec<QueryAlbum>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct CommentArgs {
    pub album_id: String,
    pub content: String,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct QueryComment {
    pub id: u32,
    pub album_id: String,
    pub commenter_id: Principal,
    pub content: String,
    pub likes: u32,
    pub status: CommentStatus,
    pub created: u64,
    pub reply: Option<Box<QueryComment>>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct QueryCommentReq {
    pub page: u32,
    pub size: u32,
    pub album_id: String,
    pub commenter_id: Option<Principal>,
    pub sort: QuerySort,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct QueryCommentResp {
    pub page: u32,
    pub total: u32,
    pub has_more: bool,
    pub data: Vec<QueryComment>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct QueryCommonReq {
    pub page: u32,
    pub size: u32,
    pub sort: QuerySort,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct QueryBlackUserResp {
    pub page: u32,
    pub total: u32,
    pub has_more: bool,
    pub data: Vec<BlacklistedUser>,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct OssInitArgs {
    pub name: String,              // bucket name
    pub file_id: u32,              // the first file id, default is 0
    pub max_file_size: u64,        // in bytes, default is 384GB
    pub max_folder_depth: u8,      // default is 10
    pub max_children: u16, // maximum number of subfolders and subfiles in a folder, default is 1000
    pub max_custom_data_size: u16, // in bytes, default is 4KB
    pub enable_hash_index: bool, // if enabled, indexing will be built using file hash
    pub visibility: u8,    // 0: private; 1: public, can be accessed by anyone, default is 0
    pub default_admin_user: Option<Principal>,
}

impl Default for OssInitArgs {
    fn default() -> Self {
        OssInitArgs {
            name: String::from("Canistore OSS"), // Default to an empty bucket name
            file_id: 0,                          // Default first file ID is 0
            max_file_size: DEFAULT_OSS_MAX_FILE_SIZE, // Default to 300GB (in bytes)
            max_folder_depth: 10,                // Default folder depth is 10
            max_children: 1000,                  // Default maximum number of children is 1000
            max_custom_data_size: 4096,          // Default custom data size is 4KB (in bytes)
            enable_hash_index: false,            // Disable hash indexing by default
            visibility: 1,                       // Default visibility is private (0)
            default_admin_user: None,
        }
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct OssUpgradeArgs {
    max_file_size: Option<u64>,
    max_folder_depth: Option<u8>,
    max_children: Option<u16>,
    max_custom_data_size: Option<u16>,
    enable_hash_index: Option<bool>,
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct QueryTrackResp {
    pub id: u64,
    pub name: String,
    pub collaborators: Vec<String>,     // Collaborators
    pub versions: Option<TrackVersion>, // List of version names or identifiers
    pub audio_file: AudioFile,          // Reference to an audio file ID
    pub artist: Principal,              // Artist as a Principal ID
    pub songwriter: bool,               // Original/Secondary Creation
    pub is_explicit_lyrics: bool,       // Whether the track has explicit lyrics
    pub is_radio_edition: bool,         // Whether the track is a radio edition
    pub instrumental: bool,             // Whether the track is instrumental
    pub duration: Option<u64>,          // Duration of the track
    pub album_id: Option<u64>,          // Optional Album ID
    pub public_at: u64,                 // The track is public at time
    pub has_license: bool,              // Whether a license has been created for the track
    pub has_share: bool,                // Whether a share has been created for the track
    pub has_share_store: bool,          // Whether a share_store has been created for the track
    pub cert_key: Option<String>,       // Certification key
    pub cert_hex: Option<String>,       // Certification hex

    pub created: u64, // Creation time
    pub updated: u64, // Last update time
}

impl QueryTrackResp {
    pub fn from_with_license(
        track: Track,
        has_license: bool,
        has_share: bool,
        has_share_store: bool,
    ) -> Self {
        QueryTrackResp {
            id: track.id,
            name: track.name,
            collaborators: track.collaborators,
            versions: track.versions,
            audio_file: track.audio_file,
            artist: track.artist,
            songwriter: track.songwriter,
            is_explicit_lyrics: track.is_explicit_lyrics,
            is_radio_edition: track.is_radio_edition,
            instrumental: track.instrumental,
            duration: track.duration,
            album_id: track.album_id,
            public_at: track.public_at,
            has_license,
            has_share,
            has_share_store,
            cert_key: track.cert_key,
            cert_hex: track.cert_hex,
            created: track.created,
            updated: track.updated,
        }
    }
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct SharedTrack {
    pub track_id: u64,
    pub channel_id: u64,
    pub created_at: u64,
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct SharedTrackListResp {
    pub track_id: u64,
    pub channel_id: u64,
    pub created_at: u64,
    pub track: Track,
}

impl SharedTrackListResp {
    pub fn new(shared_track: SharedTrack, track: Track) -> Self {
        SharedTrackListResp {
            track_id: shared_track.track_id,
            channel_id: shared_track.channel_id,
            created_at: shared_track.created_at,
            track,
        }
    }
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct UserPost {
    pub post_id: u64,
    pub content: String,
    pub created_at: u64,
}

impl UserPost {
    pub fn to_msg(
        &self,
        space_id: Principal,
        user_pid: Principal,
        user_handler: String,
    ) -> (MsgUserPost, Option<MessageSource>) {
        (
            MsgUserPost {
                space_id,
                user_pid,
                user_handler,
                content: self.content.clone(),
                created_at: self.created_at,
            },
            Some(MessageSource {
                canister_id: ic_cdk::id(),
                resource_type: String::from("UserPost"),
                resource_id: self.post_id,
            }),
        )
    }
}
