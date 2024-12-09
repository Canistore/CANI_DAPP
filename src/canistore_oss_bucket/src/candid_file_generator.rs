use serde_bytes::ByteBuf;
use std::collections::BTreeSet;

use crate::{api_cycles::WalletReceiveResult, api_init::CanisterArgs};
use candid::export_service;
use candid::Principal;
use canistore_types::{bucket::*, file::*, folder::*, ByteN};
use ic_cdk::query;

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
        write(dir.join("canistore_oss_bucket.did"), export_candid()).expect("Write failed.");
    }
}
