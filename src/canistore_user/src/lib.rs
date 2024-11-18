mod api_cycles;
mod api_init;
mod api_query;
mod api_update;
pub mod candid_file_generator;
mod guards;
mod pay;
mod store;
mod utils;

const MAX_CREATE_SPACE_SIZE: u8 = 1;

ic_cdk::export_candid!();
