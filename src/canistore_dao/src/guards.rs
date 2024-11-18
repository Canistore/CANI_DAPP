use crate::store::state;

#[inline(always)]
pub fn owner_guard() -> Result<(), String> {
    let owner = state::with(|state| state.owner);

    if ic_cdk::caller() == owner {
        Ok(())
    } else {
        Err("Error: Only the owner can call this action.".to_string())
    }
}

#[inline(always)]
pub fn dao_guard() -> Result<(), String> {
    let caller = ic_cdk::caller();

    // Fetch the owner and user_canister_id from the state
    let (owner, user_canister_id) =
        state::with(|state| (state.owner.clone(), state.user_canister_id.clone()));

    // Check if the caller is either the owner or the user_canister_id
    if caller == owner || caller == user_canister_id {
        Ok(())
    } else {
        Err("Error: Only the owner or the user_canister_id can call this action.".to_string())
    }
}

#[inline(always)]
pub fn space_guard() -> Result<(), String> {
    let caller = ic_cdk::caller();

    let (owner, user_space_infos) =
        state::with(|state| (state.owner.clone(), state.user_space_infos.clone()));

    if caller == owner || user_space_infos.contains_key(&caller) {
        Ok(())
    } else {
        Err("Error: Only the owner or a user with space info can call this action.".to_string())
    }
}
