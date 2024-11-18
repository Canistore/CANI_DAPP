use candid::Principal;
use canistore_types::error::{CustomError, ErrorCode};
use canistore_types::license::{
    LicenseListEntry, LicenseSource, LicenseTrackListEntry, QueryLicenseResp,
};
use canistore_types::payment::{QueryCommonReq, QueryOrderResp};
use canistore_types::space::{
    Album, AlbumListEntry, QueryTrackResp, SharedTrackListResp, Track, UserPost,
};
use ic_cdk::caller;
use ic_ledger_types::AccountIdentifier;

use crate::guards::{anonymous_guard, controller_guard};
use crate::store::{self, state, SpaceInfo};
use crate::utils::account_id;

#[ic_cdk::query]
fn api_version() -> u16 {
    crate::SPACE_VERSION
}

#[ic_cdk::query(guard = "controller_guard")]
fn get_space_info() -> Result<SpaceInfo, String> {
    let total_shares = store::share::total_shares();
    let total_shares_album = store::share::total_shares_album();

    Ok(store::state::with(|space| {
        SpaceInfo::from_space(space, total_shares_album, total_shares)
    }))
}

#[ic_cdk::query]
fn get_album_info(id: u64) -> Result<Album, String> {
    let album = store::album::get_album(id);
    match album {
        None => Err("album not found".to_string()),
        Some(album) => Ok(album.into_inner()),
    }
}

// #[ic_cdk::query]
// fn get_album_tracks_list(id: u64) -> Result<Vec<Track>, String> {
//     let album = store::album::get_album(id);
//     match album {
//         None => Err("album not found".to_string()),
//         Some(album) => {
//             let tracks = store::track::get_tracks_by_ids(album.into_inner().track_ids);
//             Ok(tracks)
//         }
//     }
// }

#[ic_cdk::query]
fn get_albums_list(limit: usize, offset: usize) -> Vec<AlbumListEntry> {
    store::album::get_albums_list(limit, offset)
}

#[ic_cdk::query]
fn get_license_by_track(track_id: u64, user_pid: Principal) -> Option<QueryLicenseResp> {
    match store::license::get_license_by_track(track_id) {
        Some(license_map) => {
            if let Some(license) = license_map.get(&user_pid).cloned() {
                if let Some(track) = store::track::get_track(track_id) {
                    let source = LicenseSource::Track(track.into_inner());
                    Some(QueryLicenseResp::new(license, source))
                } else {
                    None
                }
            } else {
                None
            }
        }
        None => None,
    }
}

#[ic_cdk::query]
fn get_license_list(limit: usize, offset: usize) -> Vec<LicenseListEntry> {
    store::license::get_all_licenses(limit, offset)
}

#[ic_cdk::query]
fn get_track_license_list(limit: usize, offset: usize) -> Vec<LicenseTrackListEntry> {
    let license_entries: Vec<LicenseListEntry> =
        store::license::get_track_license_list(limit, offset);

    license_entries
        .into_iter()
        .filter_map(|license_entry| {
            let track = license_entry.resource_key.track_id.and_then(|track_id| {
                store::track::get_track(track_id).map(|track_wrapper| track_wrapper.into_inner())
            });

            track.map(|track| LicenseTrackListEntry::from_license_entry(license_entry, track))
        })
        .collect()
}

#[ic_cdk::query]
fn get_track_info(id: u64) -> Result<Track, String> {
    let track = store::track::get_track(id);
    match track {
        None => Err(CustomError::new(ErrorCode::NoDataFound, Some("Album")).to_string()),
        Some(track) => Ok(track.into_inner()),
    }
}

#[ic_cdk::query]
fn get_public_track_ids() -> Vec<u64> {
    let track_ids = store::track::get_public_track_ids();
    track_ids
}

#[ic_cdk::query]
fn get_share_store_track_ids(ids: Vec<u64>) -> Vec<Track> {
    let store_track_ids = state::with(|state| state.store_track_ids.clone());

    let valid_ids: Vec<u64> = ids
        .into_iter()
        .filter(|id| store_track_ids.contains(id))
        .collect();

    store::track::get_tracks_by_ids(valid_ids)
}

#[ic_cdk::query]
fn get_total_tracks() -> u64 {
    let total_tracks = store::track::total_tracks();
    total_tracks
}

#[ic_cdk::query]
pub fn canister_account() -> AccountIdentifier {
    let canister_pid = ic_cdk::id();
    account_id(canister_pid, None)
}

#[ic_cdk::query(guard = "anonymous_guard")]
pub fn query_orders(req: QueryCommonReq) -> QueryOrderResp {
    let caller = caller(); // Get the caller's principal

    // Call limit_orders function
    let (total, has_more, data) = store::payment::limit_orders(caller, &req);

    // Return the response
    QueryOrderResp {
        page: req.page,
        total,
        has_more,
        data,
    }
}

#[ic_cdk::query]
fn get_album_tracks_list(id: u64) -> Result<Vec<QueryTrackResp>, String> {
    let album = store::album::get_album(id);

    match album {
        None => Err(CustomError::new(ErrorCode::NoDataFound, Some("Album")).to_string()),
        Some(album) => {
            let tracks = store::track::get_tracks_by_ids(album.into_inner().track_ids);

            // Map over tracks and convert them to QueryTrackResp with license check
            let query_track_resps: Vec<QueryTrackResp> = tracks
                .into_iter()
                .map(|track| {
                    let has_license = store::license::get_license_by_track(track.id).is_some();
                    let has_share = store::share::get_share(track.id).is_some();
                    let has_share_store = state::get_is_share_store(track.id);
                    QueryTrackResp::from_with_license(
                        track,
                        has_license,
                        has_share,
                        has_share_store,
                    )
                })
                .collect();

            Ok(query_track_resps)
        }
    }
}

#[ic_cdk::query]
fn get_share_list(limit: usize, offset: usize) -> Vec<SharedTrackListResp> {
    store::share::get_share_list(limit, offset)
}

#[ic_cdk::query]
fn get_post_list(limit: usize, offset: usize) -> Vec<UserPost> {
    store::post::get_post_list(limit, offset, true)
}

#[ic_cdk::query]
fn get_share_store_list(limit: usize, offset: usize) -> Vec<Track> {
    state::get_share_store_list(limit, offset)
}
