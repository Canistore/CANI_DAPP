use crate::api_cycles::WalletReceiveResult;
use crate::api_init::CanisterArgs;
use candid::{export_service, Principal};
use canistore_types::{
    certificate::{CertificateInfo, MusicCertificate, MusicCertificateResp, MusicCopyright},
    dao::DaoStateInfo,
    space::{CanisterArgs as SpaceCanisterArgs, OssInitArgs, SpaceOssCanisterArgs},
};
use ic_cdk::query;
use std::collections::HashMap;

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
        write(dir.join("canistore_dao.did"), export_candid()).expect("Write failed.");
    }
}
