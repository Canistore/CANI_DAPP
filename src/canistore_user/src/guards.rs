use crate::store::state;
use candid::Principal;

#[inline(always)]
pub fn anonymous_guard() -> Result<(), String> {
    if ic_cdk::caller() == Principal::anonymous() {
        Err(String::from("Error: Anonymous principal is not allowed"))
    } else {
        Ok(())
    }
}

#[inline(always)]
pub fn owner_guard() -> Result<(), String> {
    let owner = state::with(|state| state.owner);

    if ic_cdk::caller() == owner {
        Ok(())
    } else {
        Err("Error: Only the owner can call this action.".to_string())
    }
}
