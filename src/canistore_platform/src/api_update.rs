use crate::guards::{anonymous_guard, controller_guard, owner_guard};
use candid::Principal;
use canistore_types::{
    bucket::Token,
    cose::{
        cose_sign1,
        coset::{iana::Algorithm::ES256K, CborSerializable},
        sha256, Token as CoseToken, BUCKET_TOKEN_AAD, PLATFORM_TOKEN_AAD,
    },
    platform::TrackInfo,
    SECONDS,
};
use ic_cdk::update;
use serde_bytes::ByteBuf;

use crate::{
    ecdsa,
    store::{self, channel},
};

#[update(guard = "anonymous_guard")]
fn add_track_to_channel(channel_id: u64, track: TrackInfo) -> Result<(), String> {
    channel::add_track_to_channel(channel_id, track)
        .map_err(|err| format!("Failed to add track: {}", err))
}

#[update]
fn delete_track_from_channel(channel_id: u64, position: u64) -> Result<(), String> {
    channel::delete_track_from_channel(channel_id, position)
        .map_err(|err| format!("Failed to delete track: {}", err))
}

#[update]
fn delete_track_from_channel_by_share(
    channel_id: u64,
    space_canister_id: Principal,
    track_id: u64,
) -> Result<(), String> {
    channel::delete_track_from_channel_by_share(channel_id, space_canister_id, track_id)
        .map_err(|err| format!("Failed to delete remote track: {}", err))
}

#[update]
fn batch_add_tracks_to_channel(channel_id: u64, tracks: Vec<TrackInfo>) -> Result<(), String> {
    for track in tracks {
        channel::add_track_to_channel(channel_id, track.clone())
            .map_err(|err| format!("Failed to add track: {}", err))?;
    }
    Ok(())
}

// #[update]
// async fn init_ecdsa_public_key() -> Result<(), String> {
//     store::state::init_ecdsa_public_key().await?;
//     Ok(())
// }

#[ic_cdk::update(guard = "controller_guard")]
async fn sign_access_token(token: Token) -> Result<ByteBuf, String> {
    let now_sec = ic_cdk::api::time() / SECONDS;
    let (ecdsa_key_name, token_expiration) =
        store::state::with(|r| (r.ecdsa_key_name.clone(), r.token_expiration));
    let mut claims = CoseToken::from(token).to_cwt(now_sec as i64, token_expiration as i64);
    claims.issuer = Some(ic_cdk::id().to_text());
    let mut sign1: canistore_types::cose::coset::CoseSign1 = cose_sign1(claims, ES256K, None)?;
    let tbs_data = sign1.tbs_data(BUCKET_TOKEN_AAD);
    let message_hash = sha256(&tbs_data);

    let sig = ecdsa::sign_with(
        &ecdsa_key_name,
        vec![PLATFORM_TOKEN_AAD.to_vec()],
        message_hash,
    )
    .await?;
    sign1.signature = sig;
    let token = sign1.to_vec().map_err(|err| err.to_string())?;
    Ok(ByteBuf::from(token))
}

#[ic_cdk::update(guard = "owner_guard")]
async fn access_token(audience_canister: Principal) -> Result<ByteBuf, String> {
    let subject = ic_cdk::caller();

    let token = CoseToken {
        subject,
        audience: audience_canister,
        policies: String::from("Folder.*:1 Bucket.Read.*"),
    };

    let now_sec = ic_cdk::api::time() / SECONDS;
    let (ecdsa_key_name, token_expiration) =
        store::state::with(|r| (r.ecdsa_key_name.clone(), r.token_expiration));

    let mut claims = token.to_cwt(now_sec as i64, token_expiration as i64);
    claims.issuer = Some(ic_cdk::id().to_text());
    let mut sign1 = cose_sign1(claims, ES256K, None)?;
    let tbs_data = sign1.tbs_data(BUCKET_TOKEN_AAD);
    let message_hash = sha256(&tbs_data);

    let sig = ecdsa::sign_with(
        &ecdsa_key_name,
        vec![PLATFORM_TOKEN_AAD.to_vec()],
        message_hash,
    )
    .await?;
    sign1.signature = sig;
    let token = sign1.to_vec().map_err(|err| err.to_string())?;
    Ok(ByteBuf::from(token))
}
