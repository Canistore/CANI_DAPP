use candid::Principal;

mod api_admin;
mod api_cycles;
mod api_http;
mod api_init;
mod api_query;
mod api_update;
pub mod candid_file_generator;
mod guards;
mod permission;
mod store;

const MILLISECONDS: u64 = 1_000_000;
const SECONDS: u64 = 1_000_000_000;
const OSS_BUCKET_VERSION: u16 = 2;
static ANONYMOUS: Principal = Principal::anonymous();

ic_cdk::export_candid!();
