use crate::{
    guards::{dao_guard, owner_guard, space_guard},
    store::{self, state},
    DEPLOY_THRESHOLD,
};
use candid::{CandidType, Encode, Principal};
use canistore_types::{
    certificate::{MusicCertificate, MusicCertificateResp},
    cose::sha256,
    error::{CustomError, ErrorCode},
    format_error,
    space::{CanisterArgs as SpaceCanisterArgs, OssInitArgs, SpaceOssCanisterArgs},
};
use ic_cdk::api::management_canister::{
    main::{
        canister_status, create_canister, install_code, CanisterIdRecord, CanisterInstallMode,
        CreateCanisterArgument, InstallCodeArgument,
    },
    provisional::CanisterSettings,
};
use ic_cdk::update;
use serde_bytes::ByteArray;

pub const SPACE_WASM: &[u8] = std::include_bytes!("./../../../wasm/canistore_space.wasm.gz");
pub const OSS_WASM: &[u8] = std::include_bytes!("./../../../wasm/canistore_oss_bucket.wasm.gz");

fn check_system_open() -> Result<(), String> {
    if !state::is_open() {
        Err(CustomError::new(ErrorCode::SystemClose, None).to_string())
    } else {
        Ok(())
    }
}

async fn check_cycles() -> Result<(), String> {
    let arg = CanisterIdRecord {
        canister_id: ic_cdk::id(),
    };
    let (status,) = canister_status(arg).await.map_err(format_error)?;

    if status.cycles <= DEPLOY_THRESHOLD {
        return Err(CustomError::new(ErrorCode::InsufficientCycles, None).to_string());
    }
    Ok(())
}

async fn validate_deploy() -> Result<(), String> {
    check_system_open()?;
    check_cycles().await?;
    Ok(())
}

async fn create_canister_with_settings(caller: Principal) -> Result<Principal, String> {
    if caller == Principal::anonymous() {
        return Err("Anonymous Caller".into());
    }

    let admin_user =
        Principal::from_text("xr4tj-xppkb-gimlp-62puo-pyheq-ksj3q-5p3fn-x6bwp-pellg-isj7a-6qe")
            .unwrap();

    let create_result = create_canister(
        CreateCanisterArgument {
            settings: Some(CanisterSettings {
                controllers: Some(vec![ic_cdk::id(), caller, admin_user]),
                compute_allocation: None,
                memory_allocation: None,
                freezing_threshold: None,
                reserved_cycles_limit: None,
                log_visibility: None,
                wasm_memory_limit: None,
            }),
        },
        1_000_000_000_000,
    )
    .await;

    match create_result {
        Ok((principal,)) => Ok(principal.canister_id),
        Err((code, msg)) => Err(format!(
            "Failed to create canister. Code: {:?}, Message: {:?}",
            code, msg
        )),
    }
}

async fn install_canister_code(
    canister_id: Principal,
    wasm_module: &[u8],
    arg: &[u8],
    mode: ic_cdk::api::management_canister::main::CanisterInstallMode,
) -> Result<(), String> {
    let install_result = install_code(InstallCodeArgument {
        mode,
        canister_id,
        wasm_module: wasm_module.to_vec(),
        arg: arg.to_vec(),
    })
    .await;

    match install_result {
        Ok(()) => Ok(()),
        Err((code, msg)) => Err(format!(
            "Failed to install code. Code: {:?}, Message: {:?}",
            code, msg
        )),
    }
}

async fn create_and_install_canister(
    caller: Principal,
    wasm_name: &str,
    wasm_module: &[u8],
    init_arg: &[u8],
) -> Result<Principal, String> {
    let hash: ByteArray<32> = sha256(wasm_module).into();
    let principal = create_canister_with_settings(caller).await?;

    // save deploy info
    state::add_canister(principal, wasm_name.to_string(), hash);

    install_canister_code(
        principal,
        wasm_module,
        &init_arg,
        CanisterInstallMode::Install,
    )
    .await?;

    Ok(principal)
}

async fn upgrade_canister<T: CandidType>(
    canister_id: Principal,
    wasm_module: &[u8],
    upgrade_arg: T,
) -> Result<(), String> {
    let encoded_arg = Encode!(&upgrade_arg)
        .map_err(|_| CustomError::new(ErrorCode::FailedEncodeArgs, None).to_string())?;
    install_canister_code(
        canister_id,
        wasm_module,
        &encoded_arg,
        CanisterInstallMode::Upgrade(None),
    )
    .await?;

    Ok(())
}

#[update(guard = "dao_guard")]
async fn create_space_canister(arg: Option<SpaceCanisterArgs>) -> Result<Principal, String> {
    validate_deploy().await?;
    let init_arg = Encode!(&arg)
        .map_err(|_| CustomError::new(ErrorCode::FailedEncodeArgs, None).to_string())?;

    create_and_install_canister(ic_cdk::caller(), "SPACE_WASM", SPACE_WASM, &init_arg).await
}

#[update(guard = "dao_guard")]
async fn create_oss_canister(arg: OssInitArgs) -> Result<Principal, String> {
    validate_deploy().await?;
    let init_arg = Encode!(&arg)
        .map_err(|_| CustomError::new(ErrorCode::FailedEncodeArgs, None).to_string())?;

    create_and_install_canister(ic_cdk::caller(), "OSS_WASM", OSS_WASM, &init_arg).await
}

#[update(guard = "dao_guard")]
async fn create_space_and_oss_canister(
    space_oss_arg: SpaceOssCanisterArgs,
) -> Result<(Principal, Principal), String> {
    validate_deploy().await?;
    let owner = match &space_oss_arg.space_arg {
        Some(SpaceCanisterArgs::Init(space_init_args)) => space_init_args.owner,
        _ => ic_cdk::caller(),
    };

    let space_init_arg = Encode!(&space_oss_arg.space_arg)
        .map_err(|_| CustomError::new(ErrorCode::FailedEncodeArgs, Some("Space")).to_string())?;

    let oss_init_arg = Encode!(&space_oss_arg.oss_arg)
        .map_err(|_| CustomError::new(ErrorCode::FailedEncodeArgs, Some("OSS")).to_string())?;

    // create space canister
    let space_principal =
        create_and_install_canister(owner, "SPACE_WASM", SPACE_WASM, &space_init_arg).await?;

    // create oss canister
    let oss_principal =
        create_and_install_canister(owner, "OSS_WASM", OSS_WASM, &oss_init_arg).await?;

    // save state
    state::with_mut(|state| {
        state
            .user_space_infos
            .entry(space_principal)
            .or_insert_with(Vec::new)
            .push(oss_principal);
    });

    Ok((space_principal, oss_principal))
}

#[update(guard = "dao_guard")]
async fn upgrade_space_canister(
    canister_id: Principal,
    arg: Option<SpaceCanisterArgs>,
) -> Result<(), String> {
    upgrade_canister(canister_id, SPACE_WASM, arg).await
}

#[update(guard = "dao_guard")]
async fn upgrade_oss_canister(canister_id: Principal, arg: OssInitArgs) -> Result<(), String> {
    upgrade_canister(canister_id, OSS_WASM, arg).await
}

#[update(guard = "dao_guard")]
async fn upgrade_space_and_oss_canister(
    space_oss_arg: SpaceOssCanisterArgs,
    space_canister_id: Principal,
    oss_canister_id: Principal,
) -> Result<(), String> {
    upgrade_canister(space_canister_id, SPACE_WASM, space_oss_arg.space_arg).await?;

    upgrade_canister(oss_canister_id, OSS_WASM, space_oss_arg.oss_arg).await?;

    Ok(())
}

#[update(guard = "owner_guard")]
fn update_space_canister(space_id: Principal, oss_ids: Vec<Principal>) -> Result<bool, String> {
    state::with_mut(|state| {
        let user_spaces = state.user_space_infos.entry(space_id).or_insert(vec![]);
        user_spaces.extend(oss_ids);
        Ok(true)
    })
}

#[update(guard = "owner_guard")]
async fn update_user_canister(user_canister: Principal) -> Result<Principal, String> {
    store::state::with_mut(|r| {
        r.user_canister_id = user_canister;
    });
    Ok(user_canister)
}

#[update(guard = "owner_guard")]
async fn update_is_open(is_open: bool) -> Result<bool, String> {
    store::state::with_mut(|r| {
        r.is_open = is_open;
    });
    Ok(is_open)
}

#[update(guard = "space_guard")]
async fn store_certificate(certificate: MusicCertificate) -> Result<MusicCertificateResp, String> {
    store::certified::store_certificate(certificate)
}
