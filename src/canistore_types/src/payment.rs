use candid::{CandidType, Principal};
use icrc_ledger_types::icrc1::account::Account;
use serde::{Deserialize, Serialize};

pub const SPACE_CREATION_PRICE_ICP: u64 = 100_000_000;
pub const SPACE_LICENSE_PRICE_ICP: u64 = 100_000_000;

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum PaymentType {
    Price(SubscriptionPrice),
    LicensePrice(LicensePrice),
    CreationPrice(TokenPrice),
    Verification(bool),
    Award(AwardPrice),
}

#[derive(CandidType, Deserialize, Serialize, Clone, PartialEq, Eq, Debug)]
pub enum PaymentStatus {
    Unpaid,
    Paid,
    Cancelled,
    TimedOut,
    Refunded,
    Verifying,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct PaymentOrder {
    pub id: u64,
    pub payer: Principal,
    pub amount: u64,
    pub payment_type: PaymentType,
    pub source: String,
    pub token: String,
    pub amount_paid: u64,
    pub status: PaymentStatus,
    pub verified_time: Option<u64>,
    pub shared_time: Option<u64>,
    pub created_time: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct PaymentInfo {
    pub id: u64,
    pub recipient: Vec<u8>,
    pub token: String,
    pub amount: u64,
    pub payment_type: PaymentType,
    pub created_time: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum PaymentResponse {
    Ok { invoice: PaymentInfo },
    Err(String),
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum SubscriptionType {
    Free,
    Day30,
    Day90,
    Day180,
    Day360,
    Day1000,
    Permanent,
}

impl SubscriptionType {
    pub fn value(&self) -> u8 {
        match self {
            SubscriptionType::Free => 0,
            SubscriptionType::Day30 => 1,
            SubscriptionType::Day90 => 2,
            SubscriptionType::Day180 => 3,
            SubscriptionType::Day360 => 4,
            SubscriptionType::Day1000 => 5,
            SubscriptionType::Permanent => 6,
        }
    }

    pub fn expire_time(&self) -> u64 {
        match self {
            SubscriptionType::Free => 0,
            SubscriptionType::Day30 => 30 * 86400 * 1_000_000_000,
            SubscriptionType::Day90 => 90 * 86400 * 1_000_000_000,
            SubscriptionType::Day180 => 180 * 86400 * 1_000_000_000,
            SubscriptionType::Day360 => 360 * 86400 * 1_000_000_000,
            SubscriptionType::Day1000 => 1000 * 86400 * 1_000_000_000,
            SubscriptionType::Permanent => 500 * 360 * 86400 * 1_000_000_000,
        }
    }

    pub fn next_subscription_type(a: &Self, b: &Self) -> Self {
        if a.value() < b.value() {
            b.clone()
        } else {
            a.clone()
        }
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct SubscriptionPrice {
    pub sub_type: SubscriptionType,
    pub price: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct LicensePrice {
    pub track_id: u64,
    pub token_name: String,
    pub price: u64,
}

impl LicensePrice {
    pub fn new_for_license(track_id: u64, price: u64) -> Self {
        LicensePrice {
            track_id,
            token_name: "ICP".to_string(),
            price,
        }
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct TokenPrice {
    pub token_name: String,
    pub price: u64,
}

impl TokenPrice {
    pub fn new_for_creation_space() -> Self {
        TokenPrice {
            token_name: "ICP".to_string(),
            price: SPACE_CREATION_PRICE_ICP,
        }
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct Subscriber {
    pub pid: Principal,
    pub created: u64,
    pub sub_type: SubscriptionType,
    pub expire_time: u64,
}

impl Subscriber {
    pub fn update_subscription(&mut self, new_subscriber: &Subscriber) {
        let current_time = ic_cdk::api::time();
        if self.sub_type == SubscriptionType::Free {
            self.sub_type =
                SubscriptionType::next_subscription_type(&self.sub_type, &new_subscriber.sub_type);
            self.expire_time = new_subscriber.expire_time;
        } else if self.expire_time < current_time {
            self.sub_type = new_subscriber.sub_type.clone();
            self.expire_time = new_subscriber.expire_time;
        } else {
            self.sub_type =
                SubscriptionType::next_subscription_type(&self.sub_type, &new_subscriber.sub_type);
            self.expire_time += new_subscriber.expire_time - current_time;
        }
    }

    pub fn new(user: Principal, price: SubscriptionPrice) -> Self {
        let current_time = ic_cdk::api::time();
        Self {
            pid: user,
            created: current_time,
            sub_type: price.sub_type.clone(),
            expire_time: current_time + price.sub_type.expire_time(),
        }
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct AwardPrice {
    pub aid: String,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct AwardOrder {
    pub id: u64,
    pub album_id: String,
    pub contributor: Principal,
    pub token: String,
    pub amount: u64,
    pub created_time: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct RevenueShare {
    pub ratio: u32,
    pub recipient: Principal,
    pub remarks: String,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct SubscriberInfo {
    pub pid: Principal,
    pub sub_type: SubscriptionType,
    pub expire_time: u64,
    pub created: u64,
    pub is_black: bool,
}

#[derive(CandidType, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum QuerySort {
    TimeDesc,
    TimeAsc,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct SubscriberInfoResp {
    pub page: u32,
    pub total: i32,
    pub has_more: bool,
    pub data: Vec<SubscriberInfo>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct QuerySelfSubscriber {
    pub data: Option<SubscriberInfo>,
    pub is_blacklisted: bool,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct QueryCommonSubscriber {
    pub data: Option<SubscriberInfo>,
    pub is_subscriber: bool,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct QueryOrder {
    pub id: u64,
    pub payer: Principal,
    pub recipient: Vec<u8>,
    pub amount: u64,
    pub payment_type: PaymentType,
    pub source: String,
    pub token: String,
    pub created_time: u64,
    pub amount_paid: u64,
    pub status: PaymentStatus,
    pub verified_time: Option<u64>,
    pub shared_time: Option<u64>,
}

impl QueryOrder {
    pub fn from_payment_order(order: PaymentOrder, recipient: Vec<u8>) -> Self {
        QueryOrder {
            id: order.id,
            payer: order.payer,
            recipient,
            amount: order.amount,
            payment_type: order.payment_type.clone(),
            source: order.source.clone(),
            token: order.token.clone(),
            created_time: order.created_time,
            amount_paid: order.amount_paid,
            status: order.status,
            verified_time: order.verified_time,
            shared_time: order.shared_time,
        }
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct QueryCommonReq {
    pub page: usize,
    pub size: usize,
    pub sort: QuerySort,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct QueryOrderResp {
    pub page: usize,
    pub total: usize,
    pub has_more: bool,
    pub data: Vec<QueryOrder>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct QueryAward {
    pub id: u64,
    pub album_id: String,
    pub contributor: Principal,
    pub token: String,
    pub amount: u64,
    pub created: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct QueryAwardReq {
    pub album_id: String,
    pub page: u32,
    pub size: u32,
    pub sort: QuerySort,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct QueryAwardResp {
    pub page: u32,
    pub total: u32,
    pub has_more: bool,
    pub data: Vec<QueryAward>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct TransferArgs {
    pub recipient: Vec<u8>,
    pub memo: u64,
    pub amount: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct ICRC1TransferArgs {
    pub token: String,
    pub amount: u64,
    pub recipient: Account,
    pub memo: Option<Vec<u8>>,
}
