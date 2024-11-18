use candid::Nat;
use ciborium::into_writer;
use num_traits::cast::ToPrimitive;
use serde::Serialize;
use std::collections::BTreeMap;

pub mod bucket;
pub mod canister;
pub mod certificate;
pub mod constant;
pub mod cose;
pub mod dao;
pub mod file;
pub mod folder;
pub mod indexer;
pub mod license;
pub mod message;
pub mod oss_permission;
pub mod payment;
pub mod platform;
pub mod space;
pub mod user;
pub mod error;

mod bytes;
pub use bytes::*;

pub const MILLISECONDS: u64 = 1_000_000;
pub const SECONDS: u64 = 1_000_000_000;

pub type MapValue =
    BTreeMap<String, icrc_ledger_types::icrc::generic_metadata_value::MetadataValue>;

pub fn format_error<T>(err: T) -> String
where
    T: std::fmt::Debug,
{
    format!("{:?}", err)
}

pub fn crc32(data: &[u8]) -> u32 {
    let mut h = crc32fast::Hasher::new();
    h.update(data);
    h.finalize()
}

pub fn nat_to_u64(nat: &Nat) -> u64 {
    nat.0.to_u64().unwrap_or(0)
}

// to_cbor_bytes returns the CBOR encoding of the given object that implements the Serialize trait.
pub fn to_cbor_bytes(obj: &impl Serialize) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    into_writer(obj, &mut buf).expect("failed to encode in CBOR format");
    buf
}
