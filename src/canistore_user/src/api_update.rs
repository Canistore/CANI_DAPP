use crate::{
    guards::{anonymous_guard, owner_guard},
    MAX_CREATE_SPACE_SIZE,
};
use candid::Principal;
use canistore_types::{
    error::{CustomError, ErrorCode},
    payment::{PaymentInfo, PaymentType, TokenPrice},
    space::{CanisterArgs, OssCanisterArgs, OssInitArgs, SpaceInitArgs, SpaceOssCanisterArgs},
    user::{Attribute, UpdateUserInfo, User, UserInfo, UserSpaceInfo},
    ByteN,
};
use ic_cdk::{api::call::CallResult, caller, update};
use serde_bytes::ByteBuf;

use crate::store::{self};

async fn create_user_space_core() -> Result<Principal, String> {
    let caller = ic_cdk::caller();

    if store::user::get_user(caller).is_none() {
        return Err(
            CustomError::new(ErrorCode::NoDataFound, Some("User not registered")).to_string(),
        );
    }

    let user_space_count = store::user::get_user_spaces_count(caller);
    if user_space_count >= MAX_CREATE_SPACE_SIZE {
        return Err(
            CustomError::new(ErrorCode::MaximumRecords, Some("User Spaces Count")).to_string(),
        );
    }

    let dao_canister_id = store::state::with(|state| state.dao_canister_id);
    let env = store::state::with(|state| state.env.clone());

    if dao_canister_id == Principal::anonymous() {
        return Err(
            CustomError::new(ErrorCode::StateNotSetting, Some("dao_canister_id")).to_string(),
        );
    }

    let init_args = SpaceOssCanisterArgs {
        space_arg: Some(CanisterArgs::Init(SpaceInitArgs {
            owner: caller,
            dao_canister: dao_canister_id,
            env,
            ..SpaceInitArgs::default()
        })),
        oss_arg: OssCanisterArgs::Init(OssInitArgs {
            default_admin_user: Some(caller),
            visibility: 1,
            ..OssInitArgs::default()
        }),
    };

    let result: CallResult<(Result<(Principal, Principal), String>,)> = ic_cdk::api::call::call(
        dao_canister_id,
        "create_space_and_oss_canister",
        (init_args,),
    )
    .await;

    match result {
        Ok((response,)) => match response {
            Ok((new_space_id, new_oss_id)) => {
                store::user::add_space_to_user(
                    caller,
                    UserSpaceInfo {
                        space_id: new_space_id,
                        oss_id: vec![new_oss_id],
                    },
                )
                .map_err(|_| {
                    CustomError::new(ErrorCode::DataUpdateError, Some("add_space_to_user"))
                        .to_string()
                })?;
                Ok(new_space_id)
            }
            Err(err_msg) => {
                Err(CustomError::new(ErrorCode::RemoteCallCreateError, Some(&err_msg)).to_string())
            }
        },
        Err(err) => {
            ic_cdk::println!("{:?}", err);
            Err(CustomError::new(
                ErrorCode::RemoteCallCreateError,
                Some("create_space_and_oss_canister"),
            )
            .to_string())
        }
    }
}

#[update(guard = "anonymous_guard")]
fn user_login() -> Result<UserInfo, String> {
    let caller = caller();

    match store::user::get_user(caller) {
        Some(user) => Ok(user.into_inner().to_user_info(caller)),
        None => {
            let new_user = User::new();
            store::user::add_user(caller, new_user.clone());
            Ok(new_user.to_user_info(caller))
        }
    }
}

#[update(guard = "owner_guard")]
fn admin_login(user_pid: Principal) -> Result<UserInfo, String> {
    match store::user::get_user(user_pid) {
        Some(user) => Ok(user.into_inner().to_user_info(user_pid)),
        None => {
            let new_user = User::new();
            store::user::add_user(user_pid, new_user.clone());
            Ok(new_user.to_user_info(user_pid))
        }
    }
}

#[ic_cdk::update(guard = "anonymous_guard")]
fn set_avatar(new_avatar: String) -> Result<bool, String> {
    let caller = ic_cdk::caller();
    let user_result = store::user::get_user(caller);

    match user_result {
        Some(_user_wrapper) => {
            store::user::update_user(caller, |user| {
                user.avatar = new_avatar.clone();
                user.updated_at = ic_cdk::api::time();
            })
            .map_err(|_| {
                CustomError::new(ErrorCode::DataUpdateError, Some("User Avatar")).to_string()
            })?;

            Ok(true)
        }
        None => Ok(false),
    }
}

#[ic_cdk::update(guard = "anonymous_guard")]
fn set_email(email: String) -> Result<bool, String> {
    let caller = ic_cdk::caller();
    let user_result = store::user::get_user(caller);

    match user_result {
        Some(_user_wrapper) => {
            store::user::update_user(caller, |user| {
                user.email = email.clone();
                user.updated_at = ic_cdk::api::time();
            })
            .map_err(|_| {
                CustomError::new(ErrorCode::DataUpdateError, Some("User Email")).to_string()
            })?;

            Ok(true)
        }
        None => Ok(false),
    }
}

#[ic_cdk::update(guard = "anonymous_guard")]
fn set_public_key(
    trusted_ecdsa_pub_key: Option<ByteBuf>,
    trusted_eddsa_pub_key: Option<ByteN<32>>,
) -> Result<bool, String> {
    let caller = ic_cdk::caller();
    let user_result = store::user::get_user(caller);

    match user_result {
        Some(_user_wrapper) => {
            store::user::update_user(caller, |user| {
                user.trusted_ecdsa_pub_key = trusted_ecdsa_pub_key.clone();
                user.trusted_eddsa_pub_key = trusted_eddsa_pub_key.clone();
                user.updated_at = ic_cdk::api::time();
            })
            .map_err(|_| {
                CustomError::new(ErrorCode::DataUpdateError, Some("User Public Key")).to_string()
            })?;

            Ok(true)
        }
        None => Ok(false),
    }
}

#[ic_cdk::update(guard = "anonymous_guard")]
fn add_user_attribute(new_attribute: Attribute) -> Result<bool, String> {
    let caller = ic_cdk::caller();
    let user_wrapper = store::user::get_user(caller);

    match user_wrapper {
        Some(_user_wrapper) => {
            store::user::update_user(caller, |user| {
                user.attributes.retain(|attr| attr.key != new_attribute.key);
                user.attributes.push(new_attribute);
                user.updated_at = ic_cdk::api::time();
            })
            .map_err(|_| {
                CustomError::new(ErrorCode::DataUpdateError, Some("User Attribute")).to_string()
            })?;

            Ok(true)
        }
        None => Err("User not found".to_string()),
    }
}

#[ic_cdk::update(guard = "anonymous_guard")]
fn set_user_info(update_info: UpdateUserInfo) -> Result<bool, String> {
    let caller = ic_cdk::caller();
    let user_result = store::user::get_user(caller);

    match user_result {
        Some(_user_wrapper) => {
            store::user::update_user(caller, |user| {
                if let Some(avatar) = &update_info.avatar {
                    user.avatar = avatar.clone();
                }
                if let Some(artist_name) = &update_info.artist_name {
                    user.artist_name = artist_name.clone();
                }
                if let Some(location) = &update_info.location {
                    user.location = location.clone();
                }
                if let Some(genre) = &update_info.genre {
                    user.genre = genre.clone();
                }
                if let Some(website) = &update_info.website {
                    user.website = website.clone();
                }
                if let Some(bio) = &update_info.bio {
                    user.bio = bio.clone();
                }
                if let Some(handler) = &update_info.handler {
                    user.handler = handler.clone();
                }
                if let Some(music_content_type) = &update_info.music_content_type {
                    user.music_content_type = Some(music_content_type.clone());
                }
                if let Some(born) = update_info.born {
                    user.born = Some(born);
                }
                if let Some(confirm_agreement) = update_info.confirm_agreement {
                    user.confirm_agreement = confirm_agreement;
                }
                user.updated_at = ic_cdk::api::time();
            })
            .map_err(|_| {
                CustomError::new(ErrorCode::DataUpdateError, Some("User Info")).to_string()
            })?;

            Ok(true)
        }
        None => Ok(false),
    }
}

// #[update(guard = "anonymous_guard")]
// async fn create_user_space() -> Result<Principal, String> {
//     create_user_space_core().await
// }

#[update(guard = "anonymous_guard")]
async fn create_user_space_by_invite_code(invite_code: String) -> Result<Principal, String> {
    if !store::state::check_invite_code(&invite_code) {
        return Err(CustomError::new(
            ErrorCode::NoDataFound,
            Some("Invalid or missing invite code"),
        )
        .to_string());
    }
    store::state::delete_invite_code(invite_code)?;
    create_user_space_core().await
}

#[update(guard = "anonymous_guard")]
async fn create_user_space_by_payment(order_id: u64) -> Result<Principal, String> {
    let caller = ic_cdk::caller();
    if !store::payment::check_payment_order(caller, order_id) {
        return Err(
            CustomError::new(ErrorCode::DataInvalid, Some("Invalid Payment Order Info"))
                .to_string(),
        );
    }
    create_user_space_core().await
}

#[update(guard = "owner_guard")]
async fn update_dao_canister(dao_canister: Principal) -> Result<Principal, String> {
    store::state::with_mut(|r| {
        r.dao_canister_id = dao_canister;
    });
    Ok(dao_canister)
}

#[ic_cdk::update(guard = "owner_guard")]
fn add_user_space_info(user_pid: Principal, space_info: UserSpaceInfo) -> Result<bool, String> {
    let user_wrapper = store::user::get_user(user_pid);

    match user_wrapper {
        Some(_user_wrapper) => {
            store::user::update_user(user_pid, |user| {
                user.spaces.push(space_info);
            })
            .map_err(|_| {
                CustomError::new(ErrorCode::DataUpdateError, Some("User space info")).to_string()
            })?;

            Ok(true)
        }
        None => Err(CustomError::new(ErrorCode::NoDataFound, Some("User")).to_string()),
    }
}

#[ic_cdk::update(guard = "anonymous_guard")]
fn create_payment_order(source: String) -> Result<Option<PaymentInfo>, String> {
    let payer = caller();
    let mut payment_info: Option<PaymentInfo> = None;

    store::state::load();
    store::state::with_mut(|space| {
        let new_order_id = space.next_order_id;
        let token_price = TokenPrice::new_for_creation_space();
        let payment_type = PaymentType::CreationPrice(token_price.clone());

        payment_info = Some(store::payment::create_payment_order(
            new_order_id,
            payer,
            source,
            token_price.token_name,
            token_price.price,
            payment_type,
        ));

        space.total_orders += 1;
        space.next_order_id += 1;
    });
    store::state::save();

    Ok(payment_info)
}

#[ic_cdk::update(guard = "anonymous_guard")]
async fn confirm_payment_order(pay_id: u64) -> Result<bool, String> {
    let result = store::payment::confirm_payment_order(pay_id).await;
    result
}

#[ic_cdk::update(guard = "anonymous_guard")]
async fn refund_payment_order(pay_id: u64, to: Vec<u8>) -> Result<bool, String> {
    let from = caller();
    let result = store::payment::refund_payment_order(pay_id, from, to).await;
    result
}

#[ic_cdk::update(guard = "owner_guard")]
async fn add_invite_code(invite_code: String) -> Result<String, String> {
    store::state::add_invite_code(invite_code)
}
