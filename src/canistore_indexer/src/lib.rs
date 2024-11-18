use ic_cdk::export_candid;

mod api_cycles;
mod api_init;
mod api_query;
mod api_update;
pub mod candid_file_generator;
mod store;

export_candid!();

pub const MAX_MESSAGE_COUNT: u64 = 2000;
pub const MAX_HISTORY_MESSAGE_COUNT: u64 = 20000;
pub const ARCHIVE_MESSAGE_DEFAULT_CYCLES: u128 = 1_000_000_000_000;
pub const ARCHIVE_MESSAGE_THRESHOLD: usize = 5000;
pub const ARCHIVE_MESSAGE_MIGRATION_SIZE: usize = 500;
