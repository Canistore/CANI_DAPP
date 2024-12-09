use candid::Principal;
use canistore_types::bucket::UpdateBucketInput;
use std::collections::BTreeSet;

use crate::guards::admin_guard;
use crate::{store, ANONYMOUS};

#[ic_cdk::update(guard = "admin_guard")]
fn admin_set_managers(args: BTreeSet<Principal>) -> Result<(), String> {
    validate_admin_set_managers(args.clone())?;
    store::state::with_mut(|r| {
        r.managers = args;
    });
    Ok(())
}

#[ic_cdk::update]
fn validate_admin_set_managers(args: BTreeSet<Principal>) -> Result<(), String> {
    if args.is_empty() {
        return Err("managers cannot be empty".to_string());
    }
    if args.contains(&ANONYMOUS) {
        return Err("anonymous user is not allowed".to_string());
    }
    Ok(())
}

#[ic_cdk::update(guard = "admin_guard")]
fn admin_set_auditors(args: BTreeSet<Principal>) -> Result<(), String> {
    validate_admin_set_auditors(args.clone())?;
    store::state::with_mut(|r| {
        r.auditors = args;
    });
    Ok(())
}

#[ic_cdk::update]
fn validate_admin_set_auditors(args: BTreeSet<Principal>) -> Result<(), String> {
    if args.is_empty() {
        return Err("auditors cannot be empty".to_string());
    }
    if args.contains(&ANONYMOUS) {
        return Err("anonymous user is not allowed".to_string());
    }
    Ok(())
}

#[ic_cdk::update(guard = "admin_guard")]
fn admin_update_bucket(args: UpdateBucketInput) -> Result<(), String> {
    args.validate()?;
    store::state::with_mut(|s| {
        if let Some(name) = args.name {
            s.name = name;
        }
        if let Some(max_file_size) = args.max_file_size {
            s.max_file_size = max_file_size;
        }
        if let Some(max_folder_depth) = args.max_folder_depth {
            s.max_folder_depth = max_folder_depth;
        }
        if let Some(max_children) = args.max_children {
            s.max_children = max_children;
        }
        if let Some(max_custom_data_size) = args.max_custom_data_size {
            s.max_custom_data_size = max_custom_data_size;
        }
        if let Some(enable_hash_index) = args.enable_hash_index {
            s.enable_hash_index = enable_hash_index;
        }
        if let Some(status) = args.status {
            s.status = status;
        }
        if let Some(visibility) = args.visibility {
            s.visibility = visibility;
        }
        if let Some(trusted_ecdsa_pub_keys) = args.trusted_ecdsa_pub_keys {
            s.trusted_ecdsa_pub_keys = trusted_ecdsa_pub_keys;
        }
        if let Some(trusted_eddsa_pub_keys) = args.trusted_eddsa_pub_keys {
            s.trusted_eddsa_pub_keys = trusted_eddsa_pub_keys;
        }
    });
    Ok(())
}

#[ic_cdk::update]
fn validate_admin_update_bucket(args: UpdateBucketInput) -> Result<(), String> {
    args.validate()
}
