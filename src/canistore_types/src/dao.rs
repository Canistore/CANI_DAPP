use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteArray;

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct DaoStateInfo {
    pub name: String,
    pub user_canister_id: Principal,
    pub platform_canister_id: Principal,
    pub is_open: bool,
    pub sub_canisters: Vec<CanisterDeploy>,
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct CanisterDeploy {
    pub deploy_at: u64,
    pub canister: Principal,
    pub wasm_name: String,
    pub wasm_hash: ByteArray<32>,
}
