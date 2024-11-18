use candid::{CandidType, Decode, Encode, Principal};
use canistore_types::{
    constant::Environment,
    error::{CustomError, ErrorCode},
    payment::PaymentOrder,
    user::User,
};
use ciborium::{from_reader, into_writer};
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    storable::Bound,
    DefaultMemoryImpl, StableBTreeMap, StableCell, Storable,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct State {
    pub name: String,
    pub owner: Principal,
    pub dao_canister_id: Principal,
    pub indexer_canister_id: Principal,
    pub user_count: u128,
    pub next_order_id: u64,
    pub total_orders: u64,
    pub invite_codes: Vec<String>,
    pub env: Environment,
}

impl Default for State {
    fn default() -> Self {
        Self {
            name: String::from("User Center"),
            owner: Principal::anonymous(),
            dao_canister_id: Principal::anonymous(),
            indexer_canister_id: Principal::anonymous(),
            user_count: 0 as u128,
            next_order_id: 0,
            total_orders: 0,
            invite_codes: vec![],
            env: Environment::Test,
        }
    }
}

impl Storable for State {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> Cow<[u8]> {
        let mut buf = vec![];
        into_writer(self, &mut buf).expect("failed to encode User Center data");
        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        from_reader(&bytes[..]).expect("failed to decode User Center data")
    }
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct UserWrapper(pub User);

impl Storable for UserWrapper {
    const BOUND: Bound = Bound::Unbounded;

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }
}

impl UserWrapper {
    pub fn into_inner(self) -> User {
        self.0
    }
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct PaymentOrderWrapper(pub PaymentOrder);

impl Storable for PaymentOrderWrapper {
    const BOUND: Bound = Bound::Unbounded;

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }
}

impl PaymentOrderWrapper {
    pub fn into_inner(self) -> PaymentOrder {
        self.0
    }
}

const STATE_MEMORY_ID: MemoryId = MemoryId::new(0);
const USER_MEMORY_ID: MemoryId = MemoryId::new(1);
const PAYMENT_MEMORY_ID: MemoryId = MemoryId::new(2);

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());

    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static STATE_STORE: RefCell<StableCell<State, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(STATE_MEMORY_ID)),
            State::default()
        ).expect("failed to init STATE_STORE")
    );

    static USER_STORE: RefCell<StableBTreeMap<Principal, UserWrapper, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(USER_MEMORY_ID)),
        )
    );

    static PAYMENT_STORE: RefCell<StableBTreeMap<u64, PaymentOrderWrapper, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(PAYMENT_MEMORY_ID)),
        )
    );
}

pub mod state {
    use super::*;

    pub fn with<R>(f: impl FnOnce(&State) -> R) -> R {
        STATE.with(|r| f(&r.borrow()))
    }

    pub fn with_mut<R>(f: impl FnOnce(&mut State) -> R) -> R {
        STATE.with(|r| f(&mut r.borrow_mut()))
    }

    pub fn load() {
        STATE_STORE.with(|r| {
            let s = r.borrow().get().clone();
            STATE.with(|h| {
                *h.borrow_mut() = s;
            });
        });
    }

    pub fn save() {
        STATE.with(|h| {
            STATE_STORE.with(|r| {
                r.borrow_mut()
                    .set(h.borrow().clone())
                    .expect("failed to save User Center data");
            });
        });
    }

    pub fn add_invite_code(invite_code: String) -> Result<String, String> {
        state::with_mut(|r| {
            if r.invite_codes.contains(&invite_code) {
                return Ok(invite_code.clone());
            }

            if r.invite_codes.len() >= 20 {
                return Err(
                    CustomError::new(ErrorCode::MaximumRecords, Some("invite codes")).to_string(),
                );
            }

            r.invite_codes.push(invite_code.clone());
            Ok(invite_code.clone())
        })
    }

    pub fn check_invite_code(invite_code: &String) -> bool {
        state::with(|r| {
            if r.invite_codes.contains(invite_code) {
                return true;
            } else {
                return false;
            }
        })
    }

    pub fn delete_invite_code(invite_code: String) -> Result<String, String> {
        state::with_mut(|r| {
            if let Some(pos) = r.invite_codes.iter().position(|s| *s == invite_code) {
                r.invite_codes.remove(pos);
            }
            Ok(invite_code.clone())
        })
    }

    pub fn get_env() -> Environment {
        state::with(|r| r.env.clone())
    }
}

pub mod user {
    use canistore_types::user::{UserInfo, UserSpaceInfo};

    use super::*;

    pub fn get_user(pid: Principal) -> Option<UserWrapper> {
        USER_STORE.with(|r| r.borrow().get(&pid))
    }

    pub fn get_user_infos(user_pids: Vec<Principal>) -> Vec<UserInfo> {
        USER_STORE.with(|r| {
            let store = r.borrow();
            user_pids
                .into_iter()
                .filter_map(|pid| {
                    store
                        .get(&pid)
                        .map(|user_wrapper| user_wrapper.into_inner().to_user_info(pid))
                })
                .collect()
        })
    }

    pub fn get_user_count() -> u64 {
        USER_STORE.with(|r| r.borrow().len())
    }

    pub fn get_user_pids() -> Vec<Principal> {
        USER_STORE.with(|r| {
            let user_store = r.borrow();
            user_store.iter().map(|(principal, _)| principal).collect()
        })
    }

    pub fn get_user_spaces_count(pid: Principal) -> u8 {
        let user = USER_STORE.with(|r| r.borrow().get(&pid));
        match user {
            Some(user) => user.into_inner().spaces.len() as u8,
            None => 0,
        }
    }

    pub fn add_user(pid: Principal, user: User) {
        USER_STORE.with(|r| r.borrow_mut().insert(pid, UserWrapper(user)));
    }

    pub fn update_user(pid: Principal, update_fn: impl FnOnce(&mut User)) -> Result<(), String> {
        USER_STORE.with(|store| {
            let mut users = store.borrow_mut();

            // Fetch the existing user
            if let Some(user_wrapper) = users.get(&pid) {
                let mut user = user_wrapper.into_inner();
                update_fn(&mut user);
                users.insert(pid, UserWrapper(user));
                Ok(())
            } else {
                Err(format!("User with principal {} not found", pid))
            }
        })
    }

    pub fn add_space_to_user(pid: Principal, space_info: UserSpaceInfo) -> Result<(), String> {
        update_user(pid, |user| {
            user.spaces.push(space_info);
        })
    }
}

pub mod payment {
    use std::time::Duration;

    use crate::{
        pay::{default_account_id, token_balance, token_fee, token_transfer},
        utils::generate_order_subaccount,
    };

    use super::*;
    use crate::utils::check_page_size;
    use canistore_types::{
        payment::{PaymentInfo, PaymentStatus, PaymentType, QueryCommonReq, QueryOrder, QuerySort},
        space::SPACE_FEE,
    };
    use ic_cdk::api::time;
    use icrc_ledger_types::icrc1::account::Account;

    pub fn get_payment_order(id: u64) -> Option<PaymentOrder> {
        PAYMENT_STORE.with(|r| {
            let order_wrapper = r.borrow().get(&id);
            match order_wrapper {
                Some(order) => Some(order.into_inner()),
                None => None,
            }
        })
    }

    pub fn check_payment_order(payer: Principal, order_id: u64) -> bool {
        let order = self::get_payment_order(order_id);
        match order {
            Some(order) => {
                if order.status == PaymentStatus::Paid && order.payer == payer {
                    return true;
                } else {
                    return false;
                }
            }
            None => {
                return false;
            }
        }
    }

    pub fn create_payment_order(
        order_id: u64,
        payer: Principal,
        source: String,
        token: String,
        amount: u64,
        payment_type: PaymentType,
    ) -> PaymentInfo {
        let payment_order = PaymentOrder {
            id: order_id,
            payer,
            amount,
            payment_type: payment_type.clone(),
            source: source.clone(),
            token: token.clone(),
            amount_paid: 0,
            status: PaymentStatus::Unpaid,
            verified_time: None,
            shared_time: None,
            created_time: time(),
        };

        PAYMENT_STORE.with(|store| {
            store
                .borrow_mut()
                .insert(order_id, PaymentOrderWrapper(payment_order.clone()));
        });

        let recipient_subaccount = generate_order_subaccount(payer, order_id);

        PaymentInfo {
            id: order_id,
            recipient: recipient_subaccount,
            token,
            amount,
            payment_type,
            created_time: time(),
        }
    }

    pub fn limit_orders(caller: Principal, req: &QueryCommonReq) -> (usize, bool, Vec<QueryOrder>) {
        let mut data = Vec::new();
        let (page, size) = check_page_size(req.page, req.size);
        let start = (page - 1) * size;
        let mut has_more = false;
        let mut total = 0;

        // Collect all entries in a Vec
        let mut orders: Vec<(u64, PaymentOrderWrapper)> = PAYMENT_STORE
            .with(|store| store.borrow().iter().map(|(k, v)| (k, v.clone())).collect());

        // Sort or reverse based on the sort order
        match req.sort {
            QuerySort::TimeAsc => {}
            QuerySort::TimeDesc => {
                orders.reverse();
            }
        }

        // Iterate through the orders
        for (_idx, (_key, order_wrapper)) in orders.iter().enumerate() {
            let order = &order_wrapper.0;

            if order.payer != caller {
                continue;
            }

            if total >= start && total < start + size {
                data.push(QueryOrder::from_payment_order(
                    order.clone(),
                    generate_order_subaccount(order.payer, order.id),
                ));
            }
            total += 1;
        }

        if total > start + size {
            has_more = true;
        }

        (total, has_more, data)
    }

    pub async fn confirm_payment_order(order_id: u64) -> Result<bool, String> {
        let order = get_payment_order(order_id);

        let mut check_order = match order {
            Some(order) => order,
            None => return Err(format!("Order with id {} not found", order_id)),
        };

        // Verify the payment order
        let is_verified = verify_payment_order(&mut check_order).await;

        if is_verified {
            share_pay(&mut check_order).await;
        }

        PAYMENT_STORE.with(|store| {
            let mut store = store.borrow_mut();
            // store.remove(&order_id);
            store.insert(order_id, PaymentOrderWrapper(check_order));
        });

        if is_verified {
            Ok(true)
        } else {
            Err("Order verification failed".to_string())
        }
    }

    pub async fn refund_payment_order(
        order_id: u64,
        caller: Principal,
        to: Vec<u8>,
    ) -> Result<bool, String> {
        // Fetch the payment order
        let order = get_payment_order(order_id);
        let mut check_order = match order {
            Some(order) => order,
            None => return Err(format!("Order with id {} not found", order_id)),
        };

        // Check ownership
        if check_order.payer != caller {
            return Err("Caller is not the owner of the order".to_string());
        }

        // Perform the refund
        let is_refunded = process_refund_payment_order(&mut check_order, to).await;

        // Update the payment store
        PAYMENT_STORE.with(|store| {
            let mut store = store.borrow_mut();
            store.insert(order_id, PaymentOrderWrapper(check_order));
        });

        if is_refunded {
            Ok(true)
        } else {
            Err("Refund process failed".to_string())
        }
    }

    async fn verify_payment_order(order: &mut PaymentOrder) -> bool {
        // If order is already paid, return true
        if order.status == PaymentStatus::Paid {
            return true;
        }

        // If order is neither unpaid nor verifying, return false
        if order.status != PaymentStatus::Unpaid {
            return false;
        }

        // Update the status to verifying
        order.status = PaymentStatus::Verifying;

        // Set timeout for 15 minutes (converted to nanoseconds)
        let fifteen_minutes = Duration::from_secs(15 * 60).as_nanos() as u64;
        if order.created_time + fifteen_minutes < time() {
            order.status = PaymentStatus::TimedOut;
            return false;
        }

        // Check the account balance based on the generated subaccount
        let subaccount: Option<[u8; 32]> = generate_order_subaccount(order.payer, order.id)
            .try_into()
            .ok();
        let payment_account = Account {
            owner: ic_cdk::id(),
            subaccount,
        };
        let amount_paid = token_balance(order.token.as_str(), payment_account).await;

        // Update the order with the paid amount
        order.amount_paid = amount_paid;

        // If the amount paid is less than the required amount, mark it as unpaid and return false
        if amount_paid == 0 || amount_paid < order.amount {
            order.status = PaymentStatus::Unpaid;
            return false;
        }

        // Mark the order as paid
        order.status = PaymentStatus::Paid;
        order.verified_time = Some(time());

        true
    }

    async fn share_pay(order: &mut PaymentOrder) -> bool {
        if order.status != PaymentStatus::Paid {
            return false;
        }

        if order.shared_time.is_some() {
            return false;
        }

        let subaccount = generate_order_subaccount(order.payer, order.id)
            .try_into()
            .ok();
        let mut amount = order.amount_paid;
        let mut shared_amount = amount;

        if amount <= SPACE_FEE {
            order.shared_time = Some(ic_cdk::api::time());
            return true;
        }

        amount -= SPACE_FEE;
        if amount < shared_amount {
            shared_amount = amount;
        }

        let share_to = default_account_id().as_ref().to_vec();
        match token_transfer(&order.token, subaccount, share_to, shared_amount).await {
            Ok(_) => {
                order.shared_time = Some(time());
                true
            }
            Err(e) => {
                ic_cdk::println!("share_pay error:{:?}", e);
                false
            }
        }
    }

    async fn process_refund_payment_order(order: &mut PaymentOrder, refund_to: Vec<u8>) -> bool {
        // If the order is already paid or refunded, return false
        if order.status == PaymentStatus::Paid || order.status == PaymentStatus::Refunded {
            return false;
        }

        if order.amount == 0 {
            return false;
        }

        // Generate subaccount for the payer
        let subaccount: Option<[u8; 32]> = generate_order_subaccount(order.payer, order.id)
            .try_into()
            .ok();

        let payer_account = Account {
            owner: ic_cdk::id(),
            subaccount,
        };

        let balance = token_balance(&order.token, payer_account).await;
        order.amount_paid = balance;

        if balance < token_fee(&order.token) {
            return false;
        }

        // Calculate the refundable amount (balance minus fee)
        let amount_to_refund = balance - token_fee(&order.token);

        // Attempt to transfer the refund
        match token_transfer(&order.token, subaccount, refund_to, amount_to_refund).await {
            Ok(_) => {
                // Update order status to refunded
                order.status = PaymentStatus::Refunded;
                order.verified_time = Some(time());
                true
            }
            Err(_) => false, // If the transfer fails, return false
        }
    }
}
