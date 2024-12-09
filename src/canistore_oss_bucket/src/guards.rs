use crate::store::state;
use candid::Principal;

#[inline(always)]
#[allow(dead_code)]
pub fn is_controller() -> Result<(), String> {
    let caller = ic_cdk::caller();
    if ic_cdk::api::is_controller(&caller) {
        Ok(())
    } else {
        Err("user is not a controller".to_string())
    }
}

#[inline(always)]
pub fn admin_guard() -> Result<(), String> {
    let caller = ic_cdk::caller();
    let manager_canister = state::with(|state| state.manager_canister);

    if ic_cdk::api::is_controller(&caller)
        || (manager_canister != Principal::anonymous() && caller == manager_canister)
    {
        Ok(())
    } else {
        Err("Error: Only the owner can call this action.".to_string())
    }
}
