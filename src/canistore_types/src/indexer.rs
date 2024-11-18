use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum CanisterArgs {
    Init(IndexerInitArgs),
    Upgrade(IndexerUpgradeArgs),
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct IndexerInitArgs {
    pub name: String,     // Indexer name
    pub owner: Principal, // Owner
    pub user_count: u32,  // User count
}

impl Default for IndexerInitArgs {
    fn default() -> Self {
        IndexerInitArgs {
            name: String::from("Canistore Indexer"), // Default indexer name
            owner: Principal::anonymous(),           // Default to anonymous principal
            user_count: 0,                           // Default to User count
        }
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct IndexerUpgradeArgs {
    pub owner: Option<Principal>,
    pub user_count: Option<u32>,
}
