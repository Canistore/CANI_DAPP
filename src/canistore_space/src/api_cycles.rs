use canistore_types::canister::{StatusRequest, StatusResponse};
use ic_cdk::{query, update};

#[derive(candid::CandidType, candid::Deserialize, Debug)]
pub struct WalletReceiveResult {
    accepted: u64,
}

#[query]
pub fn wallet_balance() -> candid::Nat {
    return candid::Nat::from(ic_cdk::api::canister_balance128());
}

#[update]
pub fn wallet_receive() -> WalletReceiveResult {
    let available = ic_cdk::api::call::msg_cycles_available128();

    if available == 0 {
        return WalletReceiveResult { accepted: 0 };
    }
    let accepted = ic_cdk::api::call::msg_cycles_accept128(available);
    assert!(accepted == available);
    WalletReceiveResult {
        accepted: accepted as u64,
    }
}

#[query]
pub fn canister_get_status(request: StatusRequest) -> StatusResponse {
    let cycles = obtain_value(request.cycles, get_current_cycles);
    let memory_size = obtain_value(request.memory_size, get_current_memory_size);
    let heap_memory_size = obtain_value(request.heap_memory_size, get_current_heap_memory_size);

    StatusResponse {
        cycles,
        memory_size,
        heap_memory_size,
    }
}

fn obtain_value<T, F>(need: bool, supplier: F) -> Option<T>
where
    F: Fn() -> T,
{
    if need {
        Some(supplier())
    } else {
        None
    }
}

fn get_current_cycles() -> u64 {
    crate::utils::get_cycles()
}

fn get_current_memory_size() -> u64 {
    crate::utils::get_stable_memory_size() + crate::utils::get_heap_memory_size()
}

fn get_current_heap_memory_size() -> u64 {
    crate::utils::get_heap_memory_size()
}
