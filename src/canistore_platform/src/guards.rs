use candid::Principal;

use crate::store;

#[inline(always)]
pub fn owner_guard() -> Result<(), String> {
    let rest = store::state::with(|s| s.owner_permission(ic_cdk::caller()));
    rest
}

#[inline(always)]
pub fn controller_guard() -> Result<(), String> {
    let rest = store::state::with(|s| s.controller_or_owner_permission(ic_cdk::caller()));
    rest
}

#[inline(always)]
pub fn anonymous_guard() -> Result<(), String> {
    if ic_cdk::caller() == Principal::anonymous() {
        Err(String::from("Error: Anonymous principal is not allowed"))
    } else {
        Ok(())
    }
}
