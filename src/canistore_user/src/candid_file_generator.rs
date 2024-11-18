use candid::{export_service, Principal};
use ic_cdk::query;

use crate::api_cycles::WalletReceiveResult;
use crate::api_init::CanisterArgs;
use canistore_types::{
    canister::{StatusRequest, StatusResponse},
    payment::{PaymentInfo, QueryCommonReq, QueryOrderResp},
    user::{Attribute, UpdateUserInfo, UserInfo, UserSpaceInfo},
    ByteN,
};
use serde_bytes::ByteBuf;

#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    export_service!();
    __export_service()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_candid() {
        use std::env;
        use std::fs::write;
        use std::path::PathBuf;

        let dir = PathBuf::from(env::current_dir().unwrap());
        write(dir.join("canistore_user.did"), export_candid()).expect("Write failed.");
    }
}
