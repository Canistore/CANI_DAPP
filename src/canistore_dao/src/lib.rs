use ic_cdk::export_candid;

mod api_init;
mod api_update;
mod api_query;
mod api_cycles;
pub mod candid_file_generator;
mod store;
mod guards;

const CERTIFIED_HEADER: &str = "cani_cert";
const DEPLOY_THRESHOLD: u128 = 4_000_000_000_000;

export_candid!();
