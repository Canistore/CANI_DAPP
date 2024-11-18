use core::fmt;

use candid::{CandidType, Principal};
use ic_cdk::api::time;
use serde::{Deserialize, Serialize};

use crate::{
    space::{Album, AudioFile, MusicCategory, Track},
    user::Attribute,
};

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum CanisterArgs {
    Init(PlatformInitArgs),
    Upgrade(PlatformUpgradeArgs),
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct PlatformInitArgs {
    pub name: String,     // Canister name
    pub owner: Principal, // Owner
    pub ecdsa_key_name: String,
    pub token_expiration: u64,
    pub init_channel: bool,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct PlatformUpgradeArgs {
    pub owner: Option<Principal>,
    pub token_expiration: Option<u64>,
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug, Copy, PartialEq, Eq)]
pub enum MusicType {
    Pop,
    Rock,
    Jazz,
    Classical,
    HipHop,
    Electronic,
    Country,
    Reggae,
    Blues,
    Other, // Default for any other type
}

impl fmt::Display for MusicType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let music_type_str = match self {
            MusicType::Pop => "Pop",
            MusicType::Rock => "Rock",
            MusicType::Jazz => "Jazz",
            MusicType::Classical => "Classical",
            MusicType::HipHop => "HipHop",
            MusicType::Electronic => "Electronic",
            MusicType::Country => "Country",
            MusicType::Reggae => "Reggae",
            MusicType::Blues => "Blues",
            MusicType::Other => "Other",
        };
        write!(f, "{}", music_type_str)
    }
}

impl MusicType {
    pub fn iter() -> impl Iterator<Item = MusicType> {
        [
            MusicType::Pop,
            MusicType::Rock,
            MusicType::Jazz,
            MusicType::Classical,
            MusicType::HipHop,
            MusicType::Electronic,
            MusicType::Country,
            MusicType::Reggae,
            MusicType::Blues,
            MusicType::Other,
        ]
        .iter()
        .copied()
    }
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub enum ChannelCategory {
    Playlist, // A collection of songs (playlist)
    Radio,    // A streaming radio station
    Other,    // Any other type of music channel
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct MusicChannel {
    pub id: u64,                           // Unique identifier for the music channel
    pub name: String,                      // Name of the music channel
    pub music_type: MusicType,             // Type of music in this channel
    pub owner: Principal,                  // Owner of the music channel (Principal ID)
    pub tracks: Vec<TrackInfo>,            // List of tracks shared in this channel
    pub total_plays: u64,                  // Total play count for the channel
    pub total_likes: u64,                  // Total like count for all tracks in the channel
    pub created: u64,                      // Timestamp when the channel was created
    pub updated: u64,                      // Timestamp when the channel was last updated
    pub sorted: bool,                      // Boolean to indicate if the tracks are sorted
    pub category: Option<ChannelCategory>, // Category of the music channel
    pub image: Option<String>,
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct OssFileInfo {
    pub space_canister_id: Principal, // Space canister that shared the track
    pub track_id: u64,
    pub oss_canister_id: Principal, // OSS canister storing the music file
    pub file_id: u32,               // File ID of the music file in the OSS canister
}

impl OssFileInfo {
    pub fn new(audio_file: AudioFile, space_canister_id: Principal, track_id: u64) -> Self {
        OssFileInfo {
            space_canister_id,
            track_id,
            oss_canister_id: audio_file.canister_id,
            file_id: audio_file.file_id,
        }
    }
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct TrackInfo {
    pub name: String,
    pub artist_name: String,
    pub description: String,
    pub external_url: String,
    pub image: String,
    pub animation_url: String,
    pub audio_url: String,
    pub attributes: Vec<Attribute>,
    pub oss_file_info: Option<OssFileInfo>,
    pub album_name: Option<String>,
    pub duration: Option<u64>,
    pub release_at: Option<u64>,
    pub category: Option<MusicCategory>,
    pub likes: u64,    // Number of likes for this track
    pub plays: u64,    // Number of plays for this track
    pub position: u64, // Position of the track within the channel for sorting
    pub created_at: u64,
}

impl TrackInfo {
    pub fn new(
        album: &Album,
        track: &Track,
        space_canister_id: Principal,
        position: u64,
        external_url: Option<String>,
        animation_url: Option<String>,
        attributes: Option<Vec<Attribute>>,
    ) -> Self {
        let external_url = external_url.unwrap_or_else(|| "".to_string());
        let animation_url = animation_url.unwrap_or_else(|| "".to_string());
        let attributes = attributes.unwrap_or_else(|| vec![]);
        let oss_file_info = Some(OssFileInfo::new(
            track.audio_file.clone(),
            space_canister_id,
            track.id,
        ));

        TrackInfo {
            name: track.name.clone(),
            artist_name: album.artist.clone(),
            description: album.description.clone(),
            external_url,
            animation_url,
            attributes,
            oss_file_info,
            image: album.cover_image.clone(),
            audio_url: track.audio_file.url(),
            album_name: Some(album.title.clone()),
            duration: track.duration,
            release_at: album.release_at,
            category: Some(album.category.clone()),
            likes: album.likes as u64,
            plays: album.plays,
            position,
            created_at: track.created,
        }
    }
}

impl MusicChannel {
    // Create a new music channel
    pub fn new(
        id: u64,
        name: String,
        owner: Principal,
        music_type: MusicType,
        category: Option<ChannelCategory>,
        image: Option<String>,
    ) -> Self {
        MusicChannel {
            id,
            name,
            owner,
            tracks: Vec::new(),
            total_plays: 0,
            total_likes: 0,
            music_type,
            created: time(),
            updated: time(),
            sorted: false,
            category,
            image,
        }
    }

    // Add a new track to the channel
    pub fn add_track(&mut self, track: TrackInfo) {
        self.tracks.push(track);
    }

    // Delete a track from the channel by its position
    pub fn delete_track(&mut self, position: u64) -> Result<(), String> {
        if position as usize >= self.tracks.len() {
            return Err("Track position out of range".to_string());
        }

        self.tracks.remove(position as usize);
        Ok(())
    }

    // Delete a track from the channel by matching the space_canister_id and track_id in OssFileInfo
    pub fn delete_track_oss_file(
        &mut self,
        space_canister_id: Principal,
        track_id: u64,
    ) -> Result<(), String> {
        let track_index = self.tracks.iter().position(|track| {
            if let Some(oss_file_info) = &track.oss_file_info {
                oss_file_info.space_canister_id == space_canister_id
                    && oss_file_info.track_id == track_id
            } else {
                false
            }
        });

        if let Some(index) = track_index {
            self.tracks.remove(index);
            Ok(())
        } else {
            Err(format!(
                "No track found with space_canister_id: {} and track_id: {}",
                space_canister_id, track_id
            ))
        }
    }

    // Sort tracks within the channel (e.g., by play count, like count, etc.)
    pub fn sort_tracks_by_play_count(&mut self) {
        self.tracks.sort_by(|a, b| b.plays.cmp(&a.plays));
    }

    // Sort tracks by like count
    pub fn sort_tracks_by_like_count(&mut self) {
        self.tracks.sort_by(|a, b| b.likes.cmp(&a.likes));
    }

    // Increment play count of the entire channel
    pub fn increment_play_count(&mut self) {
        self.total_plays += 1;
    }
}
