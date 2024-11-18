use candid::Principal;
use ic_ledger_types::{AccountIdentifier, Subaccount};
use sha2::{Digest, Sha224};

#[cfg(target_arch = "wasm32")]
const WASM_PAGE_SIZE: u64 = 65536;

pub fn get_cycles() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        ic_cdk::api::canister_balance()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0
    }
}

pub fn get_stable_memory_size() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        (ic_cdk::api::stable::stable_size() as u64) * WASM_PAGE_SIZE
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0
    }
}

pub fn get_heap_memory_size() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        (core::arch::wasm32::memory_size(0) as u64) * WASM_PAGE_SIZE
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        0
    }
}

pub fn check_page_size(page: usize, size: usize) -> (usize, usize) {
    let page = if page > 0 { page } else { 1 };
    let size = if size > 0 && size <= 100 { size } else { 10 };
    (page, size)
}

pub fn new_subaccount(opt_vec: Option<[u8; 32]>) -> Option<Subaccount> {
    match opt_vec {
        Some(vec) => Some(Subaccount(vec)),
        None => None,
    }
}

pub fn account_id(principal: Principal, subaccount: Option<[u8; 32]>) -> AccountIdentifier {
    let subaccount = subaccount
        .map(Subaccount)
        .unwrap_or_else(|| Subaccount([0; 32]));
    AccountIdentifier::new(&principal, &subaccount)
}

pub fn generate_order_subaccount(caller: Principal, payid: u64) -> Vec<u8> {
    // Create SHA-224 hasher
    let mut hasher = Sha224::new();

    // Length of domain separator (0x0A)
    hasher.update([0x0A]);

    // Domain separator ("payid")
    hasher.update("payid".as_bytes());

    // Counter (payid) as nonce, encoded as big-endian bytes
    let payid_bytes = payid.to_be_bytes();
    hasher.update(payid_bytes);

    // Principal of caller (blob representation)
    hasher.update(caller.as_slice());

    // Finalize the hash and get the result
    let hash_sum = hasher.finalize();

    // Calculate CRC32 checksum of the hash sum
    let crc32 = crc32fast::hash(&hash_sum);

    // Prepare buffer for the final result (CRC32 + hash sum)
    let mut buf = Vec::with_capacity(32);

    // Add CRC32 (big-endian 4 bytes) to the buffer
    buf.extend_from_slice(&crc32.to_be_bytes());

    // Add the hash sum to the buffer
    buf.extend_from_slice(&hash_sum);

    buf
}
