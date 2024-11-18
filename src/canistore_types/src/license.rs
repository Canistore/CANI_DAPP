use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

use crate::space::{Album, Track};

#[derive(CandidType, Clone, Deserialize, Serialize, Debug, Eq, PartialEq, Hash)]
pub struct LicenseKey {
    pub album_id: Option<u64>, // Optional album ID
    pub track_id: Option<u64>, // Optional track ID
}

impl LicenseKey {
    pub fn new(album_id: Option<u64>, track_id: Option<u64>) -> Self {
        Self { album_id, track_id }
    }

    pub fn to_tuple(&self) -> (u64, u64) {
        (self.album_id.unwrap_or(0), self.track_id.unwrap_or(0))
    }
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub enum ChannelType {
    Platform,
    User,     // e.g., Direct user assignment
    Merchant, // e.g., Partner or merchant platform
    Other,    // Any other category
}

// Asset Type
#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub enum AssetType {
    RecordingWithVocals,
    RecordingInstrumental,
    AudioSample,
    SoundFX,
}

// Usage Rights
#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub enum UsageRights {
    Commercial,
    NonCommercial,
}

// Licensed Media
#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub enum LicensedMedia {
    Exclusive,
    AllMedia,
}

// Licensed Territory
#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub enum LicensedTerritory {
    ListedTerritories,
    Worldwide,
}

// Right Period
#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub enum RightPeriod {
    Months12,
    Years3,
    Years5,
    Years10,
    Perpetuity,
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct License {
    pub id: u64,                     // License ID
    pub user: Principal,             // Licensed user
    pub resource_key: LicenseKey,    // License resource key (album_id, track_id)
    pub start_time: u64,             // License start time
    pub valid_duration: Option<u64>, // License validity duration (in seconds)
    pub revoke_time: Option<u64>,    // License revocation time (optional)
    pub channel: ChannelType,        // License channel (e.g., application, website)
    pub asset_type: Vec<AssetType>,
    pub usage_rights: Vec<UsageRights>,
    pub licensed_media: Vec<LicensedMedia>,
    pub licensed_territory: Vec<LicensedTerritory>,
    pub right_period: Vec<RightPeriod>,
    pub fee: Option<u128>,
    pub created: u64,
}

impl License {
    pub fn is_active(&self, current_time: u64) -> bool {
        // Check if the license has been revoked
        if let Some(revoke_time) = self.revoke_time {
            return current_time < revoke_time;
        }

        // If valid_duration is None, the license is considered perpetual (no expiry)
        match self.valid_duration {
            Some(duration) => current_time < self.start_time + duration,
            None => true,
        }
    }
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub enum LicenseSource {
    Track(Track),
    Album(Album),
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct QueryLicenseResp {
    pub id: u64,                     // License ID
    pub user: Principal,             // Licensed user
    pub resource_key: LicenseKey,    // License resource key (album_id, track_id)
    pub start_time: u64,             // License start time
    pub valid_duration: Option<u64>, // License validity duration (in seconds)
    pub revoke_time: Option<u64>,    // License revocation time (optional)
    pub channel: ChannelType,        // License channel (e.g., application, website)
    pub asset_type: Vec<AssetType>,
    pub usage_rights: Vec<UsageRights>,
    pub licensed_media: Vec<LicensedMedia>,
    pub licensed_territory: Vec<LicensedTerritory>,
    pub right_period: Vec<RightPeriod>,
    pub fee: Option<u128>,
    pub created: u64,
    pub source: LicenseSource,
}

impl QueryLicenseResp {
    pub fn new(license: License, source: LicenseSource) -> Self {
        QueryLicenseResp {
            id: license.id,
            user: license.user,
            resource_key: license.resource_key,
            start_time: license.start_time,
            valid_duration: license.valid_duration,
            revoke_time: license.revoke_time,
            channel: license.channel,
            asset_type: license.asset_type,
            usage_rights: license.usage_rights,
            licensed_media: license.licensed_media,
            licensed_territory: license.licensed_territory,
            right_period: license.right_period,
            fee: license.fee,
            created: license.created,
            source,
        }
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct LicenseListEntry {
    pub id: u64,                  // License ID
    pub user: Principal,          // Licensed user
    pub resource_key: LicenseKey, // License resource key (album_id, track_id)
    pub start_time: u64,          // License start time
    pub fee: Option<u128>,        // License fee (optional)
    pub created: u64,             // License creation time
}

impl From<&License> for LicenseListEntry {
    fn from(license: &License) -> Self {
        LicenseListEntry {
            id: license.id,
            user: license.user,
            resource_key: license.resource_key.clone(),
            start_time: license.start_time,
            fee: license.fee,
            created: license.created,
        }
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct LicenseTrackListEntry {
    pub id: u64,                  // License ID
    pub user: Principal,          // Licensed user
    pub resource_key: LicenseKey, // License resource key (album_id, track_id)
    pub start_time: u64,          // License start time
    pub fee: Option<u128>,        // License fee (optional)
    pub created: u64,             // License creation time
    pub track: Track,             // Associated Track
}

impl LicenseTrackListEntry {
    pub fn from_license_entry(license_entry: LicenseListEntry, track: Track) -> Self {
        LicenseTrackListEntry {
            id: license_entry.id,
            user: license_entry.user,
            resource_key: license_entry.resource_key,
            start_time: license_entry.start_time,
            fee: license_entry.fee,
            created: license_entry.created,
            track,
        }
    }
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct CreateTrackLicenseArg {
    pub track_id: u64,
    pub user_pid: Principal,
    pub channel: ChannelType,
    pub asset_type: Vec<AssetType>,
    pub usage_rights: Vec<UsageRights>,
    pub licensed_media: Vec<LicensedMedia>,
    pub licensed_territory: Vec<LicensedTerritory>,
    pub right_period: Vec<RightPeriod>,
    pub fee: Option<u128>,
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct LicenseRecord {
    pub id: u64,          // License usage record ID
    pub license_id: u64,  // Corresponding license ID
    pub user: Principal,  // User who used the license
    pub access_time: u64, // Access time
    pub action: String,   // Action performed (e.g., play, download)
}

impl LicenseRecord {
    pub fn new(
        id: u64,
        license_id: u64,
        user: Principal,
        access_time: u64,
        action: String,
    ) -> Self {
        Self {
            id,
            license_id,
            user,
            access_time,
            action,
        }
    }
}
