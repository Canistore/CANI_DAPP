use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub enum CanisterType {
    Dao,
    User,
    Platform,
    Indexer,
    Ledger,
    CkBTCLedger,
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub enum Environment {
    Test,
    Production,
}

impl Environment {
    pub fn get_canister_pid(&self, canister_type: CanisterType) -> &'static str {
        match self {
            Environment::Test => match canister_type {
                CanisterType::Dao => TEST_DAO_CANISTER_PID,
                CanisterType::User => TEST_USER_CANISTER_PID,
                CanisterType::Platform => TEST_PLATFORM_CANISTER_PID,
                CanisterType::Indexer => TEST_INDEXER_CANISTER_PID,
                CanisterType::Ledger => TEST_LEDGER_CANISTER_ID,
                CanisterType::CkBTCLedger => TEST_CKBTC_LEDGER_CANISTER_ID,
            },
            Environment::Production => match canister_type {
                CanisterType::Dao => DAO_CANISTER_PID,
                CanisterType::User => USER_CANISTER_PID,
                CanisterType::Platform => PLATFORM_CANISTER_PID,
                CanisterType::Indexer => INDEXER_CANISTER_PID,
                CanisterType::Ledger => LEDGER_CANISTER_ID,
                CanisterType::CkBTCLedger => CKBTC_LEDGER_CANISTER_ID,
            },
        }
    }
}

pub const TEST_DAO_CANISTER_PID: &str = "vght4-jyaaa-aaaag-aceyq-cai";
pub const TEST_USER_CANISTER_PID: &str = "rzw6u-kqaaa-aaaag-al5ga-cai";
pub const TEST_PLATFORM_CANISTER_PID: &str = "myniu-wqaaa-aaaah-advna-cai";
pub const TEST_INDEXER_CANISTER_PID: &str = "ikdny-nyaaa-aaaag-ab2za-cai";
pub const TEST_LEDGER_CANISTER_ID: &str = "s57im-oyaaa-aaaas-akwma-cai";
pub const TEST_CKBTC_LEDGER_CANISTER_ID: &str = "s57im-oyaaa-aaaas-akwma-cai";

pub const DAO_CANISTER_PID: &str = "tlcef-5iaaa-aaaas-akjmq-cai";
pub const USER_CANISTER_PID: &str = "tcbpz-laaaa-aaaas-akjna-cai";
pub const PLATFORM_CANISTER_PID: &str = "tfajn-gyaaa-aaaas-akjnq-cai";
pub const INDEXER_CANISTER_PID: &str = "tqhya-hqaaa-aaaas-akjoa-cai";
pub const LEDGER_CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
pub const CKBTC_LEDGER_CANISTER_ID: &str = "mxzaz-hqaaa-aaaar-qaada-cai";
