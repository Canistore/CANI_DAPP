use candid::Principal;
use canistore_types::{
    certificate::{MusicCertificate, MusicCertificateResp},
    constant::CanisterType,
    dao::DaoStateInfo,
    error::{CustomError, ErrorCode},
    license::CreateTrackLicenseArg,
    message::{MessageSource, MessageType, MsgShareTrack, MsgUserPost},
    payment::{LicensePrice, PaymentInfo, PaymentType, SPACE_LICENSE_PRICE_ICP},
    platform::TrackInfo,
    space::{
        Album, CreateAlbumArg, CreateTrackArg, EditAlbumArg, EditTrackArg, SharedTrack, Track,
        UserPost,
    },
    user::Attribute,
};
use ic_cdk::{api::time, caller};
use ic_ledger_types::{
    AccountBalanceArgs, Tokens, TransferArgs, TransferResult, MAINNET_LEDGER_CANISTER_ID,
};

use crate::{
    api_query::canister_account,
    canister_service::CanisterService,
    guards::{anonymous_guard, owner_guard, write_guard},
    store::{self, state},
    SHARE_PLATFORM_CHANNEL_ID,
};

#[ic_cdk::update(guard = "write_guard")]
fn create_album(args: CreateAlbumArg) -> Result<u64, String> {
    // Step 1: Load current state of Space.
    state::load();

    // Step 2: Access and modify the Space state.
    let album_id = state::with_mut(|space| {
        // Check if the maximum number of albums has been reached.
        if space.total_albums >= space.max_albums as u64 {
            return Err(
                CustomError::new(ErrorCode::MaximumRecords, Some("max_albums")).to_string(),
            );
        }

        // Step 3: Generate a new album ID.
        let new_album_id = space.next_album_id + 1;

        // Step 4: Create a new Album using the `new` method.
        let new_album = Album::new(
            new_album_id,
            args.album_type,
            args.title,
            args.artist,
            args.cover_image,
            args.producer.unwrap_or_else(|| caller()),
            args.description,
            args.category,
            args.sub_category,
            args.is_original,
            args.external_link,
            args.tags,
            args.language,
            args.release_at,
            args.copyright,
            args.subscription_prices,
        );

        store::album::add_album(new_album_id, new_album);

        space.total_albums += 1;
        space.next_album_id += 1;

        // Return the new album ID for reference.
        Ok(new_album_id)
    });

    // Save the modified Space state.
    state::save();

    // Return the result (new album ID or error message).
    album_id
}

#[ic_cdk::update(guard = "write_guard")]
fn edit_album(id: u64, args: EditAlbumArg) -> Result<(), String> {
    if let Some(existing_album) = store::album::get_album(id) {
        let updated_sub_category = if let Some(sub_category) = args.sub_category {
            Some(sub_category)
        } else {
            existing_album.0.sub_category
        };
        let updated_release_at = if let Some(release_at) = args.release_at {
            Some(release_at)
        } else {
            existing_album.0.release_at
        };
        let updated_copyright = if let Some(copyright) = args.copyright {
            Some(copyright)
        } else {
            existing_album.0.copyright
        };

        let updated_album = Album {
            id: existing_album.0.id,
            album_type: args.album_type.unwrap_or(existing_album.0.album_type),
            title: args.title.unwrap_or(existing_album.0.title),
            artist: args.artist.unwrap_or(existing_album.0.artist),
            cover_image: args.cover_image.unwrap_or(existing_album.0.cover_image),
            producer: args.producer.unwrap_or(existing_album.0.producer),
            description: args.description.unwrap_or(existing_album.0.description),
            category: args.category.unwrap_or(existing_album.0.category),
            sub_category: updated_sub_category,
            is_original: args.is_original.unwrap_or(existing_album.0.is_original),
            external_link: args.external_link.unwrap_or(existing_album.0.external_link),
            tags: args.tags.unwrap_or(existing_album.0.tags),
            language: args.language.unwrap_or(existing_album.0.language),
            release_at: updated_release_at,
            copyright: updated_copyright,
            subscription_prices: args
                .subscription_prices
                .unwrap_or(existing_album.0.subscription_prices),
            created: existing_album.0.created,
            updated: ic_cdk::api::time(),
            ..existing_album.0
        };

        store::album::edit_album(id, updated_album)?;
        Ok(())
    } else {
        Err(CustomError::new(ErrorCode::NoDataFound, Some("Album")).to_string())
    }
}

#[ic_cdk::update(guard = "write_guard")]
fn delete_album(album_id: u64) -> Result<String, String> {
    state::load();

    let result = state::with_mut(|space| {
        // Check if the album exists in the store.
        if store::album::get_album(album_id).is_none() {
            return Err(CustomError::new(ErrorCode::NoDataFound, Some("Album")).to_string());
        }

        // Delete the album from the store.
        store::album::delete_album(album_id);

        // Update the album count.
        if space.total_albums > 0 {
            space.total_albums -= 1;
        }

        Ok(CustomError::new(
            ErrorCode::DataIsDeleted,
            Some(album_id.to_string().as_str()),
        )
        .to_string())
    });

    state::save();

    result
}

#[ic_cdk::update(guard = "write_guard")]
fn create_track(args: CreateTrackArg) -> Result<u64, String> {
    // Step 1: Load current state of Space.
    state::load();

    // Step 2: Access and modify the Space state.
    let track_id = state::with_mut(|space| {
        // Check if the maximum number of tracks has been reached.
        if space.total_tracks >= space.max_track_files as u64 {
            return Err(
                CustomError::new(ErrorCode::MaximumRecords, Some("max_track_files")).to_string(),
            );
        }

        // Step 3: Generate a new track ID.
        let new_track_id = space.next_track_id + 1;

        // Step 4: Create a new Track using the provided args.
        let new_track = Track {
            id: new_track_id,
            name: args.name.clone(),
            collaborators: args.collaborators.clone(),
            versions: args.versions.clone(),
            audio_file: args.audio_file.clone(),
            artist: caller(),
            songwriter: args.songwriter,
            is_explicit_lyrics: args.is_explicit_lyrics,
            is_radio_edition: args.is_radio_edition,
            instrumental: args.instrumental,
            duration: args.duration,
            file_size: args.file_size,
            file_format: args.file_format,
            ..Default::default()
        };

        println!("Adding new track: {:?}", new_track);
        store::track::add_track(new_track_id, new_track);

        space.total_tracks += 1;
        space.next_track_id += 1;

        // Return the new track ID for reference.
        Ok(new_track_id)
    });

    // Step 7: Save the modified Space state.
    state::save();

    // Step 8: Create a music certificate asynchronously.
    if let Ok(track_id) = track_id {
        ic_cdk::spawn(async move {
            match create_music_certificate(track_id).await {
                Ok(_) => println!("Music certificate created for track ID: {}", track_id),
                Err(err) => println!("Failed to create music certificate: {}", err),
            }
        });
    }

    track_id
}

#[ic_cdk::update(guard = "write_guard")]
fn edit_track(id: u64, args: EditTrackArg) -> Result<(), String> {
    if let Some(existing_track) = store::track::get_track(id) {
        let updated_versions = if let Some(versions) = args.versions {
            Some(versions)
        } else {
            existing_track.0.versions
        };
        let updated_duration = if let Some(duration) = args.duration {
            Some(duration)
        } else {
            existing_track.0.duration
        };

        let updated_track = Track {
            id: existing_track.0.id,
            name: args.name.unwrap_or(existing_track.0.name),
            collaborators: args.collaborators.unwrap_or(existing_track.0.collaborators),
            versions: updated_versions,
            audio_file: args.audio_file.unwrap_or(existing_track.0.audio_file),
            songwriter: args.songwriter.unwrap_or(existing_track.0.songwriter),
            is_explicit_lyrics: args
                .is_explicit_lyrics
                .unwrap_or(existing_track.0.is_explicit_lyrics),
            is_radio_edition: args
                .is_radio_edition
                .unwrap_or(existing_track.0.is_radio_edition),
            instrumental: args.instrumental.unwrap_or(existing_track.0.instrumental),
            duration: updated_duration,
            file_size: args.file_size.unwrap_or(existing_track.0.file_size),
            file_format: args.file_format.unwrap_or(existing_track.0.file_format),
            created: existing_track.0.created,
            updated: ic_cdk::api::time(),
            ..existing_track.0
        };

        store::track::edit_track(id, updated_track)?;
        Ok(())
    } else {
        Err(CustomError::new(ErrorCode::NoDataFound, Some("Track")).to_string())
    }
}

#[ic_cdk::update(guard = "write_guard")]
fn add_track_attribute(track_id: u64, new_attribute: Attribute) -> Result<bool, String> {
    if let Some(_) = store::track::get_track(track_id) {
        store::track::update_track_field(track_id, |track| {
            // Retain attributes that do not have the same key as the new attribute
            track
                .attributes
                .retain(|attr| attr.key != new_attribute.key);
            // Add the new attribute
            track.attributes.push(new_attribute);
            // Update the updated_at timestamp
            track.updated = ic_cdk::api::time();
        })
        .map_err(|_| {
            CustomError::new(ErrorCode::DataUpdateError, Some("Track Attribute")).to_string()
        })?;

        Ok(true)
    } else {
        Err(CustomError::new(ErrorCode::NoDataFound, Some("Track")).to_string())
    }
}

#[ic_cdk::update(guard = "write_guard")]
fn delete_track(id: u64) -> Result<(), String> {
    state::load();

    let result = state::with_mut(|space| {
        let delete_result = store::track::delete_track(id);

        match delete_result {
            Ok(()) => {
                space.total_tracks = space.total_tracks.saturating_sub(1);
                Ok(())
            }
            Err(err) => Err(err),
        }
    });
    state::save();
    result
}

#[ic_cdk::update(guard = "write_guard")]
fn add_track_ids_to_album(album_id: u64, new_track_ids: Vec<u64>) -> Result<(), String> {
    // Step 1: Check new_track_ids exists.
    if new_track_ids.len() == 0 || store::track::check_track_ids(new_track_ids.clone()) == false {
        return Err(CustomError::new(ErrorCode::ParamsError, None).to_string());
    }

    // Step 2: Check if the album exists.
    match store::album::get_album(album_id) {
        Some(mut album_wrapper) => {
            // Extract the album from the wrapper.
            let album = &mut album_wrapper.0;

            // Get the existing track IDs.
            let existing_track_ids = &album.track_ids;

            // Filter out any duplicate track IDs from the new ones.
            let unique_track_ids: Vec<u64> = new_track_ids
                .into_iter()
                .filter(|id| !existing_track_ids.contains(id))
                .collect();

            if unique_track_ids.is_empty() {
                return Err("No new unique track IDs to add".to_string());
            }

            for track_id in unique_track_ids.clone() {
                store::track::set_track_album_id(track_id, Some(album_id))?;
            }

            store::album::add_track_ids_to_album(album_id, unique_track_ids)
        }
        None => Err(CustomError::new(ErrorCode::NoDataFound, Some("Album")).to_string()),
    }
}

#[ic_cdk::update(guard = "write_guard")]
fn remove_track_ids_to_album(album_id: u64, remove_track_ids: Vec<u64>) -> Result<(), String> {
    if remove_track_ids.len() == 0
        || store::track::check_track_ids(remove_track_ids.clone()) == false
    {
        return Err(CustomError::new(ErrorCode::ParamsError, Some("remove_track_ids")).to_string());
    }
    match store::album::get_album(album_id) {
        Some(_album_wrapper) => {
            let result =
                store::album::remove_track_ids_from_album(album_id, remove_track_ids.clone());
            if result.is_ok() {
                for track_id in remove_track_ids {
                    store::track::set_track_album_id(track_id, None)?;
                }
            }
            result
        }
        None => Err(CustomError::new(ErrorCode::NoDataFound, Some("Album")).to_string()),
    }
}

#[ic_cdk::update(guard = "write_guard")]
async fn create_track_license(args: CreateTrackLicenseArg) -> Result<u64, String> {
    // Step 1: Retrieve both the track and associated album.
    let (track, album) = store::track::get_track_and_album(args.track_id)
        .map_err(|err| format!("Failed to retrieve track and album: {}", err))?;

    // genesis_license
    let user_pid = if args.user_pid == ic_cdk::caller() {
        ic_cdk::id()
    } else {
        args.user_pid
    };

    // Step 2: Load current state of Space.
    state::load();

    // Step 3: Access and modify the Space state.
    let license_id = state::with_mut(|space| {
        // Step 4: Generate a new license ID.
        let new_license_id = space.next_license_id;

        // Step 5: Add a new license using the provided args.
        store::license::add_license(
            new_license_id,
            None,
            Some(args.track_id),
            user_pid,
            args.channel,
            args.asset_type,
            args.usage_rights,
            args.licensed_media,
            args.licensed_territory,
            args.right_period,
            args.fee,
        );

        // Update total licenses and next license ID.
        space.total_licenses += 1;
        space.next_license_id += 1;

        // Return the new license ID for reference.
        Ok(new_license_id)
    });
    state::save();

    // Step 6: set track public at time
    store::track::set_track_public_status(args.track_id, time())?;

    // Step 7: Generate the share track message with the album cover image as an additional parameter.
    let (msg_share_track, msg_resource) = track.to_msg(
        ic_cdk::caller(), // user_pid
        album.cover_image,
    );

    // Step 8: Send the message using the utility function.
    send_share_track_message(
        msg_share_track,
        args.track_id,
        msg_resource,
        MessageType::Create,
    )
    .await?;

    license_id
}

#[ic_cdk::update(guard = "write_guard")]
async fn delete_track_license(track_id: u64) -> Result<u64, String> {
    let (track, album) = store::track::get_track_and_album(track_id)
        .map_err(|err| format!("Failed to retrieve track and album: {}", err))?;

    state::load();
    let delete_result = state::with_mut(|space| {
        let delete_result = store::license::delete_license(None, Some(track_id));

        match delete_result {
            Ok(()) => {
                space.total_licenses = space.total_licenses.saturating_sub(1);
                Ok(())
            }
            Err(err) => Err(err),
        }
    });

    if let Err(_) = delete_result {
        return Err(CustomError::new(ErrorCode::FailedUpdateState, None).to_string());
    }
    state::save();

    let (msg_share_track, msg_resource) = track.to_msg(ic_cdk::caller(), album.cover_image);

    // Send the message using the utility function
    send_share_track_message(msg_share_track, track_id, msg_resource, MessageType::Delete).await?;

    Ok(track_id)
}

#[ic_cdk::update(guard = "write_guard")]
fn add_track_license_for_platform(track_id: u64) -> Result<u64, String> {
    state::load();
    let license_id = state::with_mut(|space| {
        let new_license_id = space.next_license_id + 1;

        store::license::add_track_license_for_platform(new_license_id, track_id);
        store::track::set_track_public_status(track_id, time())?;

        space.total_licenses += 1;
        space.next_license_id += 1;

        // Return the new track ID for reference.
        Ok(new_license_id)
    });
    state::save();

    license_id
}

#[ic_cdk::update(guard = "write_guard")]
fn remove_track_license_for_platform(track_id: u64) -> Result<(), String> {
    state::load();
    let result = state::with_mut(|space| {
        if let Some(_existing_license_id) = store::license::get_license_by_track(track_id) {
            store::license::remove_track_license_for_platform(track_id);
            store::track::set_track_public_status(track_id, 0)?;

            space.total_licenses -= 1;
            Ok(())
        } else {
            Err(CustomError::new(ErrorCode::NoDataFound, Some("License")).to_string())
        }
    });
    state::save();
    result
}

#[ic_cdk::update(guard = "owner_guard")]
pub async fn canister_balance() -> Result<Tokens, String> {
    let account_identifier = canister_account();

    let result: Result<(Tokens,), _> = ic_cdk::api::call::call(
        MAINNET_LEDGER_CANISTER_ID,
        "account_balance",
        (AccountBalanceArgs {
            account: account_identifier,
        },),
    )
    .await;

    result
        .map(|(balance,)| balance)
        .map_err(|_| CustomError::new(ErrorCode::FetchDataError, Some("Balance")).to_string())
}

#[ic_cdk::update(guard = "owner_guard")]
async fn canister_transfer(args: TransferArgs) -> Result<bool, String> {
    // let caller_pid = caller();
    // let from_account = account_id(ic_cdk::id(), None);
    let balance_result = canister_balance();

    let balance = match balance_result.await {
        Ok(tokens) => tokens.e8s(),
        Err(_) => return Err(CustomError::new(ErrorCode::BalanceRetrieveError, None).to_string()),
    };

    if balance < (args.amount.e8s() + args.fee.e8s()) {
        return Ok(false);
    }

    let transfer_args = TransferArgs {
        memo: args.memo,
        amount: args.amount,
        fee: args.fee,
        from_subaccount: None,
        to: args.to,
        created_at_time: None,
    };

    let transfer_result: Result<(TransferResult,), _> =
        ic_cdk::api::call::call(MAINNET_LEDGER_CANISTER_ID, "transfer", (transfer_args,)).await;

    match transfer_result {
        Ok((TransferResult::Ok(_),)) => Ok(true),
        Ok((TransferResult::Err(_),)) => Ok(false),
        Err(_) => Err(CustomError::new(ErrorCode::BalanceTransferError, None).to_string()),
    }
}

#[ic_cdk::update(guard = "anonymous_guard")]
fn create_award_order(
    source: String,
    token: String,
    amount: u64,
    payment_type: PaymentType,
) -> Result<Option<PaymentInfo>, String> {
    let payer = caller();
    let mut payment_info: Option<PaymentInfo> = None;

    state::load();
    state::with_mut(|space| {
        let new_order_id = space.next_order_id + 1;

        payment_info = Some(store::payment::create_award_order(
            new_order_id,
            payer,
            source,
            token,
            amount,
            payment_type,
        ));

        space.total_orders += 1;
        space.next_order_id += 1;
    });
    state::save();

    Ok(payment_info)
}

#[ic_cdk::update(guard = "anonymous_guard")]
fn create_license_order(track_id: u64, source: String) -> Result<Option<PaymentInfo>, String> {
    let payer = caller();

    let license = match store::license::get_track_genesis_license(track_id) {
        Some(license) => license,
        None => return Err("Genesis license not found for the provided track ID.".to_string()),
    };
    let amount = license.fee.unwrap_or(SPACE_LICENSE_PRICE_ICP as u128) as u64;
    let token = "ICP".to_string();
    let payment_type = PaymentType::LicensePrice(LicensePrice::new_for_license(track_id, amount));

    let mut payment_info: Option<PaymentInfo> = None;

    state::load();
    state::with_mut(|space| {
        let new_order_id = space.next_order_id + 1;

        payment_info = Some(store::payment::create_award_order(
            new_order_id,
            payer,
            source,
            token,
            amount,
            payment_type,
        ));

        space.total_orders += 1;
        space.next_order_id += 1;
    });
    state::save();

    Ok(payment_info)
}

#[ic_cdk::update(guard = "anonymous_guard")]
async fn confirm_award_order(pay_id: u64) -> Result<bool, String> {
    let result = store::payment::confirm_payment_order(pay_id).await;
    result
}

#[ic_cdk::update(guard = "anonymous_guard")]
async fn confirm_license_order(pay_id: u64) -> Result<u64, String> {
    // Step 1: Retrieve the order and ensure it is a LicensePrice order.
    let order = store::payment::get_payment_order(pay_id).ok_or("Order not found")?;
    let track_id = match &order.payment_type {
        PaymentType::LicensePrice(license_price) => license_price.track_id,
        _ => {
            return Err(CustomError::new(
                ErrorCode::DataInvalid,
                Some("Payment type is not LicensePrice"),
            )
            .to_string())
        }
    };

    // Step 2: Retrieve the genesis license for the track.
    let genesis_license = store::license::get_track_genesis_license(track_id)
        .ok_or("Genesis license not found for the given track_id")?;

    // Step 3: Confirm the payment order.
    if !store::payment::confirm_payment_order(pay_id).await? {
        return Ok(0);
    }

    // Step 4: Clone the genesis license, update the user, and save as a new license.
    let new_license_id = state::with_mut(|space| {
        let new_license_id = space.next_license_id;

        // Clone and update the genesis license with the payer's user information.
        let mut new_license = genesis_license.clone();
        new_license.user = order.payer;

        // Add the new license.
        store::license::add_license(
            new_license_id,
            None,
            Some(track_id),
            new_license.user,
            new_license.channel,
            new_license.asset_type,
            new_license.usage_rights,
            new_license.licensed_media,
            new_license.licensed_territory,
            new_license.right_period,
            new_license.fee,
        );

        // Update license counters.
        space.total_licenses += 1;
        space.next_license_id += 1;

        new_license_id
    });

    state::save();

    Ok(new_license_id)
}

#[ic_cdk::update(guard = "anonymous_guard")]
async fn refund_payment_order(pay_id: u64, to: Vec<u8>) -> Result<bool, String> {
    let from = caller();
    let result = store::payment::refund_payment_order(pay_id, from, to).await;
    result
}

#[ic_cdk::update(guard = "write_guard")]
async fn create_music_certificate(track_id: u64) -> Result<MusicCertificateResp, String> {
    let track_info = match store::track::get_track(track_id) {
        Some(track) => track.into_inner(),
        None => return Err(CustomError::new(ErrorCode::NoDataFound, Some("Track")).to_string()),
    };

    if let Some(cert_key) = &track_info.cert_key {
        return Err(CustomError::new(ErrorCode::DataIsExists, Some(cert_key.as_str())).to_string());
    }

    let title = track_info.name.clone();
    let owner = track_info.artist;
    let unique_id = format!("{}_{}", ic_cdk::api::id(), track_info.id);
    let artist = track_info.collaborators.join(", "); // Combine collaborators as artist string

    // Construct the MusicCertificate object
    let certificate = MusicCertificate {
        title,
        owner,
        unique_id,
        artist,
    };

    // Call the `store_certificate` method on the DAO canister
    let env = state::get_env();
    let dao_service = CanisterService::init(&env, &CanisterType::Dao)?;

    let result = dao_service.store_certificate(certificate).await;

    match result {
        Ok((Ok(certificate_resp),)) => {
            match store::track::set_track_cert(
                track_info.id,
                Some(certificate_resp.key.clone()),
                Some(certificate_resp.music_cert_hex.clone()),
            ) {
                Ok(_) => Ok(certificate_resp),
                Err(_) => Err(CustomError::new(
                    ErrorCode::DataUpdateError,
                    Some("track certificate"),
                )
                .to_string()),
            }
        }
        Ok((Err(error_msg),)) => {
            Err(CustomError::new(ErrorCode::RemoteCallCreateError, Some(&error_msg)).to_string())
        }
        Err(ic_error) => Err(CustomError::new(
            ErrorCode::RemoteCallCreateError,
            Some((ic_error.0 as u8).to_string().as_str()),
        )
        .to_string()),
    }
}

#[ic_cdk::update(guard = "owner_guard")]
async fn update_dao_canister(dao_canister: Principal) -> Result<Principal, String> {
    store::state::with_mut(|r| {
        r.dao_canister = dao_canister;
    });
    Ok(dao_canister)
}

#[ic_cdk::update(guard = "owner_guard")]
async fn add_managers(new_managers: Vec<Principal>) -> Result<(), String> {
    state::add_managers(new_managers);
    Ok(())
}

#[ic_cdk::update(guard = "write_guard")]
async fn add_contract_services(new_service: String) -> Result<String, String> {
    store::state::with_mut(|r| {
        if r.services.contains(&new_service) {
            return Ok(new_service.clone());
        }

        if r.services.len() >= 20 {
            return Err(CustomError::new(ErrorCode::MaximumRecords, Some("services")).to_string());
        }

        r.services.push(new_service.clone());
        Ok(new_service.clone())
    })
}

#[ic_cdk::update(guard = "write_guard")]
async fn create_post(content: String) -> Result<String, String> {
    let mut new_post_id: u64 = 0;
    state::load();

    // Step 1: Create the UserPost
    state::with_mut(|space| {
        new_post_id = space.next_post_id;

        let user_post = UserPost {
            post_id: new_post_id,
            content: content.clone(),
            created_at: ic_cdk::api::time(),
        };

        store::post::create_post(new_post_id, user_post);

        space.total_post += 1;
        space.next_post_id += 1;
    });

    state::save();

    let user_post = store::post::get_post(new_post_id)
        .ok_or(CustomError::new(ErrorCode::NoDataFound, Some("Post")).to_string())?;
    let (msg_user_post, msg_resource) = user_post.into_inner().to_msg(
        ic_cdk::id(),     // space_id
        ic_cdk::caller(), // user_pid
        String::from("user_handler_example"),
    );

    // Send the message using the utility function
    send_post_message(msg_user_post, new_post_id, msg_resource).await
}

#[ic_cdk::update(guard = "write_guard")]
fn delete_post(post_id: u64) -> Result<(), String> {
    state::load();

    store::post::delete_post(post_id)?;
    state::with_mut(|space| {
        if space.total_post > 0 {
            space.total_post -= 1;
        }
    });

    state::save();

    Ok(())
}

#[ic_cdk::update(guard = "write_guard")]
async fn remove_contract_services(service_to_remove: String) -> Result<String, String> {
    store::state::with_mut(|r| {
        if let Some(pos) = r.services.iter().position(|s| *s == service_to_remove) {
            r.services.remove(pos);
            Ok(service_to_remove.clone())
        } else {
            Err(CustomError::new(ErrorCode::NoDataFound, Some("Service")).to_string())
        }
    })
}

#[ic_cdk::update]
async fn remote_get_dao_info() -> Result<DaoStateInfo, String> {
    let env = state::get_env();
    let dao_service = CanisterService::init(&env, &CanisterType::Dao)?;

    let result = dao_service.get_dao_info().await;

    match result {
        Ok((dao_info,)) => Ok(dao_info),
        Err((code, msg)) => {
            ic_cdk::print(format!(
                "An error occurred during get_dao_info: {}: {}",
                code as u8, msg
            ));
            Err(format!("Failed to fetch DAO info: {}: {}", code as u8, msg))
        }
    }
}

#[ic_cdk::update(guard = "write_guard")]
async fn remote_share_track_to_platform(
    track_id: u64,
    channel_id: Option<u64>,
    external_url: Option<String>,
    animation_url: Option<String>,
    attributes: Option<Vec<Attribute>>,
) -> Result<(), String> {
    let (track, album) = store::track::get_track_and_album(track_id)?;

    let space_canister_id = ic_cdk::id();
    let position = 0;
    let selected_channel_id = channel_id.unwrap_or(SHARE_PLATFORM_CHANNEL_ID);

    let track_info = TrackInfo::new(
        &album,
        &track,
        space_canister_id,
        position,
        external_url,
        animation_url,
        attributes,
    );

    let env = state::get_env();
    let platform_service = CanisterService::init(&env, &CanisterType::Platform)?;

    match platform_service
        .add_track_to_channel(selected_channel_id, track_info)
        .await
    {
        Ok((Ok(()),)) => {
            let shared_track = SharedTrack {
                track_id,
                channel_id: selected_channel_id,
                created_at: ic_cdk::api::time(),
            };

            store::share::create_share(track_id, shared_track).map_err(|_| {
                CustomError::new(ErrorCode::DataCreateError, Some("Share")).to_string()
            })?;

            Ok(())
        }
        Ok((Err(msg),)) => {
            ic_cdk::print(format!("Failed to add track to channel: {}", msg));
            Err(CustomError::new(ErrorCode::FailedAddTrackToChannel, None).to_string())
        }
        Err((code, msg)) => {
            ic_cdk::print(format!(
                "Error occurred during add_track_to_channel: {}: {}",
                code as u8, msg
            ));
            Err(CustomError::new(
                ErrorCode::FailedAddTrackToChannel,
                Some((code as u8).to_string().as_str()),
            )
            .to_string())
        }
    }
}

#[ic_cdk::update(guard = "write_guard")]
async fn remote_batch_share_track_to_platform(
    track_ids: Vec<u64>,
    channel_id: Option<u64>,
    external_urls: Option<Vec<String>>,
    animation_urls: Option<Vec<String>>,
    attributes: Option<Vec<Vec<Attribute>>>,
) -> Result<(), String> {
    if !store::track::check_track_ids(track_ids.clone()) {
        return Err(CustomError::new(ErrorCode::NoDataFound, Some("track_ids")).to_string());
    }

    let mut track_infos = Vec::new();
    let selected_channel_id = channel_id.unwrap_or(SHARE_PLATFORM_CHANNEL_ID);

    for (i, track_id) in track_ids.iter().enumerate() {
        let (track, album) = store::track::get_track_and_album(*track_id)?;

        let space_canister_id = ic_cdk::id();
        let position = i as u64;

        let track_info = TrackInfo::new(
            &album,
            &track,
            space_canister_id,
            position,
            external_urls.as_ref().and_then(|urls| urls.get(i).cloned()),
            animation_urls
                .as_ref()
                .and_then(|urls| urls.get(i).cloned()),
            attributes.as_ref().and_then(|attrs| attrs.get(i).cloned()),
        );

        track_infos.push(track_info);
    }

    let env = state::get_env();
    let platform_service = CanisterService::init(&env, &CanisterType::Platform)?;

    match platform_service
        .batch_add_tracks_to_channel(selected_channel_id, track_infos)
        .await
    {
        Ok((Ok(()),)) => {
            let created_at = ic_cdk::api::time();
            store::share::batch_create_share(track_ids.clone(), selected_channel_id, created_at)
                .map_err(|_| {
                    CustomError::new(ErrorCode::DataCreateError, Some("Share")).to_string()
                })?;
            Ok(())
        }
        Ok((Err(msg),)) => {
            ic_cdk::print(format!("Failed to add tracks to channel: {}", msg));
            Err(CustomError::new(ErrorCode::FailedAddTrackToChannel, None).to_string())
        }
        Err((code, msg)) => {
            ic_cdk::print(format!(
                "Error occurred during batch_add_tracks_to_channel: {}: {}",
                code as u8, msg
            ));
            Err(CustomError::new(
                ErrorCode::RemoteCallBatchCreateError,
                Some((code as u8).to_string().as_str()),
            )
            .to_string())
        }
    }
}

#[ic_cdk::update(guard = "write_guard")]
async fn remote_delete_share_track_to_platform(track_id: u64) -> Result<(), String> {
    let shared_track = store::share::get_share(track_id)
        .ok_or_else(|| format!("Share for track ID {} not found", track_id))?;

    let selected_channel_id = shared_track.into_inner().channel_id;
    let space_canister_id = ic_cdk::id();

    let env = state::get_env();
    let platform_service = CanisterService::init(&env, &CanisterType::Platform)?;

    // Call delete_track_from_channel_by_share method
    match platform_service
        .delete_track_from_channel_by_share(selected_channel_id, space_canister_id, track_id)
        .await
    {
        Ok((Ok(()),)) => {
            // After successfully removing from platform, delete local share record
            store::share::delete_share(track_id).map_err(|_| {
                CustomError::new(ErrorCode::DataDeleteError, Some("local share")).to_string()
            })?;

            Ok(())
        }
        Ok((Err(msg),)) => {
            ic_cdk::print(format!("Failed to delete track from platform: {}", msg));
            Err(CustomError::new(
                ErrorCode::RemoteCallDeleteError,
                Some("track from platform"),
            )
            .to_string())
        }
        Err((code, msg)) => {
            ic_cdk::print(format!(
                "Error occurred during delete_track_from_channel_by_share: {}: {}",
                code as u8, msg
            ));
            Err(CustomError::new(
                ErrorCode::RemoteCallDeleteError,
                Some((code as u8).to_string().as_str()),
            )
            .to_string())
        }
    }
}

// #[ic_cdk::update(guard = "owner_guard")]
// async fn share_track_to_store(track_id: u64) -> Result<u64, String> {
//     state::with_mut(|state| {
//         if !state.store_track_ids.contains(&track_id) {
//             state.store_track_ids.push(track_id);
//         }
//     });

//     let track = store::track::get_track(track_id).ok_or("Failed to retrieve the track")?;
//     let (msg_share_track, msg_resource) = track.into_inner().to_msg(
//         ic_cdk::caller(), // user_pid
//     );

//     // Send the message using the utility function
//     send_share_track_message(msg_share_track, track_id, msg_resource, MessageType::Create).await?;
//     Ok(track_id)
// }

// #[ic_cdk::update(guard = "owner_guard")]
// async fn delete_share_track_to_store(track_id: u64) -> Result<u64, String> {
//     state::with_mut(|state| {
//         if let Some(pos) = state.store_track_ids.iter().position(|&id| id == track_id) {
//             state.store_track_ids.remove(pos);
//         }
//     });

//     let track = store::track::get_track(track_id).ok_or("Failed to retrieve the track")?;
//     let (msg_share_track, msg_resource) = track.into_inner().to_msg(
//         ic_cdk::caller(), // user_pid
//     );

//     // Send the message using the utility function
//     send_share_track_message(msg_share_track, track_id, msg_resource, MessageType::Delete).await?;

//     Ok(track_id)
// }

#[ic_cdk::update]
async fn remote_send_post_message() -> Result<String, String> {
    let msg_user_post = MsgUserPost {
        space_id: ic_cdk::id(),
        user_pid: ic_cdk::caller(),
        user_handler: String::from("user_handler_example"),
        content: String::from("This is a post content example"),
        created_at: ic_cdk::api::time(),
    };

    send_post_message(msg_user_post, 1, None).await
}

async fn send_post_message(
    msg_user_post: MsgUserPost,
    message_id: u64,
    msg_resource: Option<MessageSource>,
) -> Result<String, String> {
    store::message::send_message(
        MessageType::Create,
        "MsgUserPost",
        message_id,
        msg_user_post,
        msg_resource,
    )
    .await
}

async fn send_share_track_message(
    msg_share_track: MsgShareTrack,
    message_id: u64,
    msg_resource: Option<MessageSource>,
    msg_type: MessageType,
) -> Result<String, String> {
    store::message::send_message(
        msg_type,
        "MsgShareTrack",
        message_id,
        msg_share_track,
        msg_resource,
    )
    .await
}
