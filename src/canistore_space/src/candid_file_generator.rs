use candid::{export_service, Principal};
use ic_cdk::query;

use crate::api_cycles::WalletReceiveResult;
use crate::store::SpaceInfo;

use canistore_types::{
    canister::{StatusRequest, StatusResponse},
    certificate::MusicCertificateResp,
    dao::DaoStateInfo,
    license::{CreateTrackLicenseArg, LicenseListEntry, LicenseTrackListEntry, QueryLicenseResp},
    payment::{PaymentInfo, PaymentType, QueryCommonReq, QueryOrderResp},
    space::{
        Album, AlbumListEntry, CanisterArgs, CreateAlbumArg, CreateTrackArg, EditAlbumArg,
        EditTrackArg, QueryTrackResp, SharedTrackListResp, Track, UserPost,
    },
    user::Attribute,
};
use ic_ledger_types::{AccountIdentifier, Tokens, TransferArgs};

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
        write(dir.join("canistore_space.did"), export_candid()).expect("Write failed.");
    }
}
