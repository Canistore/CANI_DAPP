use std::collections::HashMap;

use candid::Principal;
use canistore_types::{
    certificate::{CertificateInfo, MusicCopyright},
    dao::DaoStateInfo,
    error::{CustomError, ErrorCode},
};

use crate::store::{self};

#[ic_cdk::query]
fn get_dao_info() -> DaoStateInfo {
    store::state::with(|state| state.to_state_info())
}

#[ic_cdk::query]
fn get_user_space_info(user_pid: Principal) -> Option<Vec<Principal>> {
    store::state::with(|state| state.user_space_infos.get(&user_pid).cloned())
}

#[ic_cdk::query]
fn get_user_space_info_list() -> HashMap<Principal, Vec<Principal>> {
    store::state::with(|state| state.user_space_infos.clone())
}

#[ic_cdk::query]
pub fn get_certificate(key: String) -> Result<MusicCopyright, String> {
    store::certified::get_certificate(key)
}

#[ic_cdk::query]
pub fn get_certificate_info(key: String) -> Result<CertificateInfo, String> {
    let cert_info = store::cert::get_cret_info(key);
    match cert_info {
        None => Err(CustomError::new(ErrorCode::NoDataFound, None).to_string()),
        Some(cert_info) => Ok(cert_info.into_inner()),
    }
}
