use candid::Principal;

use ic_ledger_types::DEFAULT_FEE;

mod api_cycles;
mod api_init;
mod api_query;
mod api_update;
pub mod candid_file_generator;
mod canister_service;
mod guards;
mod pay;
mod store;
mod utils;

const SPACE_VERSION: u16 = 2;
const SPACE_FEE: u64 = DEFAULT_FEE.e8s();
const SYSTEM_LICENSE_USER: Principal = Principal::anonymous();
const SHARE_PLATFORM_CHANNEL_ID: u64 = 1;

ic_cdk::export_candid!();
