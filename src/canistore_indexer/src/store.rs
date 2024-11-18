use candid::{CandidType, Decode, Encode, Principal};

use canistore_types::message::Message;
use ciborium::{from_reader, into_writer};
use ic_cdk_timers::TimerId;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    storable::Bound,
    DefaultMemoryImpl, StableBTreeMap, StableCell, Storable,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, cell::RefCell, collections::BTreeSet};
use crate::{ARCHIVE_MESSAGE_MIGRATION_SIZE, ARCHIVE_MESSAGE_THRESHOLD};

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct Indexer {
    pub name: String,
    pub owner: Principal,
    pub user_count: u32,
}

impl Default for Indexer {
    fn default() -> Self {
        Self {
            name: String::from("default_indexer"),
            owner: Principal::anonymous(),
            user_count: 0,
        }
    }
}

impl Storable for Indexer {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> Cow<[u8]> {
        let mut buf = vec![];
        into_writer(self, &mut buf).expect("failed to encode Indexer data");
        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        from_reader(&bytes[..]).expect("failed to decode Indexer data")
    }
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct MessageSetWrapper(pub BTreeSet<(Message, Principal)>);

impl Storable for MessageSetWrapper {
    const BOUND: Bound = Bound::Unbounded;

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }
}

impl MessageSetWrapper {
    pub fn into_inner(self) -> BTreeSet<(Message, Principal)> {
        self.0
    }

    pub fn new() -> Self {
        MessageSetWrapper(BTreeSet::new())
    }

    pub fn add_message(&mut self, message: Message, principal: Principal) {
        self.0.insert((message, principal));
    }

    pub fn remove_message(&mut self, message_id: &str) {
        self.0.retain(|(msg, _)| msg.msg_id != message_id);
    }

    pub fn get_message(&self, message_id: &str) -> Option<(Message, Principal)> {
        self.0
            .iter()
            .find(|(msg, _)| msg.msg_id == message_id)
            .cloned()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

const INDEXER_MEMORY_ID: MemoryId = MemoryId::new(0);
const MESSAGE_MEMORY_ID: MemoryId = MemoryId::new(1);
const HISTORY_MEMORY_ID: MemoryId = MemoryId::new(2);

thread_local! {
    static INDEXER: RefCell<Indexer> = RefCell::new(Indexer::default());

    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static INDEXER_STORE: RefCell<StableCell<Indexer, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(INDEXER_MEMORY_ID)),
            Indexer::default()
        ).expect("failed to init INDEXER_STORE store")
    );

    static MESSAGE_STORE: RefCell<StableBTreeMap<String, MessageSetWrapper, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(MESSAGE_MEMORY_ID)),
        )
    );

    static HISTORY_STORE: RefCell<StableBTreeMap<String, MessageSetWrapper, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(HISTORY_MEMORY_ID)),
        )
    );

    pub static TIMER_IDS: RefCell<Vec<TimerId>> = RefCell::new(Vec::new());
}

pub mod timer {
    use super::*;
    use std::time::Duration;

    pub fn set_message_clean_up_timer() {
        let secs = Duration::from_secs(10);
        let clean_task = async {
            clean_message_store_task().await;
        };
        let timer_id = ic_cdk_timers::set_timer(secs, move || {
            ic_cdk::spawn(clean_task);
        });
        TIMER_IDS.with(|timer_ids| timer_ids.borrow_mut().push(timer_id));
    }
    
    async fn clean_message_store_task() {
        // Collect keys that need migration from MESSAGE_STORE.
        let keys_to_migrate = MESSAGE_STORE.with(|message_store| {
            let message_store = message_store.borrow();
    
            let mut keys = Vec::new();
            for (key, message_set_wrapper) in message_store.iter() {
                if message_set_wrapper.0.len() > ARCHIVE_MESSAGE_THRESHOLD {
                    ic_cdk::println!("clean_message_store_task: key {} exceeds threshold, migrating data to HISTORY_STORE", key);
                    keys.push(key.clone());
                }
            }
            keys
        });
    
        // For each key, perform migration and deletion in sequence.
        for key in keys_to_migrate {
            // Step 1: Migrate data to HISTORY_STORE.
            let messages_to_migrate = MESSAGE_STORE.with(|message_store| {
                let message_store = message_store.borrow_mut();
                if let Some(message_set_wrapper) = message_store.get(&key) {
                    // Collect the oldest 1000 messages.
                    let messages: Vec<_> = message_set_wrapper
                        .0
                        .iter()
                        .take(ARCHIVE_MESSAGE_MIGRATION_SIZE)
                        .cloned()
                        .collect();
                    Some(messages)
                } else {
                    None
                }
            });
    
            // Proceed only if there are messages to migrate.
            if let Some(messages_to_migrate) = messages_to_migrate {
                // Save data in HISTORY_STORE.
                HISTORY_STORE.with(|history_store| {
                    let mut history_store = history_store.borrow_mut();
                    
                    // Retrieve the existing history set or create a new one.
                    let mut history_set = history_store.get(&key).unwrap_or_else(|| MessageSetWrapper::new());
    
                    // Add messages to HISTORY_STORE.
                    for (message, principal) in &messages_to_migrate {
                        history_set.add_message(message.clone(), principal.clone());
                    }
    
                    // Reinsert the modified history set into the HISTORY_STORE.
                    history_store.insert(key.clone(), history_set);
    
                    ic_cdk::println!(
                        "clean_message_store_task: migrated {} messages from key {} to HISTORY_STORE",
                        messages_to_migrate.len(), key
                    );
                });
    
                // Step 2: After successfully saving, delete from MESSAGE_STORE.
                MESSAGE_STORE.with(|message_store| {
                    let mut message_store = message_store.borrow_mut();
                    if let Some(mut message_set_wrapper) = message_store.get(&key) {
                        for message in messages_to_migrate {
                            message_set_wrapper.0.remove(&message);
                        }
    
                        // Reinsert the modified message set back into MESSAGE_STORE.
                        message_store.insert(key.clone(), message_set_wrapper);
                        
                        ic_cdk::println!(
                            "clean_message_store_task: removed {} messages from key {} in MESSAGE_STORE",
                            ARCHIVE_MESSAGE_MIGRATION_SIZE, key
                        );
                    }
                });
            }
        }
    
        ic_cdk::println!("clean_message_store_task: Completed cleanup task");
    
        // Re-schedule the cleanup task.
        set_message_clean_up_timer();
    }
    
    
}

pub mod state {
    use super::*;

    #[allow(dead_code)]
    pub fn with<R>(f: impl FnOnce(&Indexer) -> R) -> R {
        INDEXER.with(|r| f(&r.borrow()))
    }

    pub fn with_mut<R>(f: impl FnOnce(&mut Indexer) -> R) -> R {
        INDEXER.with(|r| f(&mut r.borrow_mut()))
    }

    pub fn load() {
        INDEXER_STORE.with(|r| {
            let s = r.borrow().get().clone();
            INDEXER.with(|h| {
                *h.borrow_mut() = s;
            });
        });
    }

    pub fn save() {
        INDEXER.with(|h| {
            INDEXER_STORE.with(|r| {
                r.borrow_mut()
                    .set(h.borrow().clone())
                    .expect("failed to set INDEXER_STORE data");
            });
        });
    }
}

pub mod message {
    use super::*;
    use canistore_types::message::{MessageType, MsgShareTrack, MsgUserInfo, MsgUserPost};
    use ic_cdk::print;
    pub fn get_message_size() -> (Vec<(String, usize)>, usize) {
        MESSAGE_STORE.with(|store| {
            let store_ref = store.borrow();
            let mut set_sizes = Vec::new();
            let mut total_size = 0;
    
            for (key, wrapper) in store_ref.iter() {
                let set_size = wrapper.into_inner().len();
                total_size += set_size;
                set_sizes.push((key.clone(), set_size));
            }
    
            (set_sizes, total_size)
        })
    }

    // Retrieve a message by its message ID
    pub fn get_message(message_type: &str, message_id: &str) -> Option<(Message, Principal)> {
        MESSAGE_STORE.with(|store| {
            store
                .borrow()
                .get(&message_type.to_string())
                .and_then(|wrapper| wrapper.get_message(message_id))
        })
    }

    pub fn get_message_keys() -> Vec<String> {
        MESSAGE_STORE.with(|store| {
            let store_ref = store.borrow();
            let mut keys_vec = Vec::new();
    
            for (key, _) in store_ref.iter() {
                keys_vec.push(key.clone());
            }
            keys_vec
        })
    }

    // Add a new message to the store
    pub fn create_message(message_type: &str, message: Message, principal: Principal) {
        let mut message_count = 0;

        MESSAGE_STORE.with(|store| {
            let mut store_ref = store.borrow_mut();

            if let Some(wrapper) = store_ref.get(&message_type.to_string()) {
                let mut cloned_wrapper = wrapper.clone();
                message_count = cloned_wrapper.0.len();
                cloned_wrapper.add_message(message, principal);
                store_ref.insert(message_type.to_string(), cloned_wrapper);
            } else {
                let mut new_wrapper = MessageSetWrapper::new();
                new_wrapper.add_message(message, principal);
                store_ref.insert(message_type.to_string(), new_wrapper);
            }
        });

        // Only set the cleanup timer if the number of messages exceeds 5000
        if message_count > ARCHIVE_MESSAGE_THRESHOLD {
            timer::set_message_clean_up_timer();
        }
    }


    // Retrieve a list of messages for a given message type with limit and offset
    pub fn get_message_list(
        message_type: &str,
        limit: usize,
        offset: usize,
    ) -> Vec<(Message, Principal)> {
        MESSAGE_STORE.with(|store| {
            let store_ref = store.borrow();
    
            if let Some(wrapper) = store_ref.get(&message_type.to_string()) {
                let mut messages: Vec<_> = wrapper.0.iter().cloned().collect();
                messages.sort_by(|a, b| b.0.timestamp.cmp(&a.0.timestamp));
                messages.into_iter().skip(offset).take(limit).collect()
            } else {
                Vec::new()
            }
        })
    }

    pub fn get_message_list_by_pid(
        message_type: &str,
        pid: Principal,
        limit: usize,
        offset: usize,
    ) -> Vec<(Message, Principal)> {
        MESSAGE_STORE.with(|store| {
            let store_ref = store.borrow();

            if let Some(wrapper) = store_ref.get(&message_type.to_string()) {
                let filtered_iter = wrapper
                    .into_inner()
                    .iter()
                    .filter(|(_, p)| p == &pid) // Filter by the given Principal (pid)
                    .skip(offset) // Skip the offset
                    .take(limit) // Take the limited number of messages
                    .cloned() // Clone the filtered results
                    .collect(); // Collect into a Vec

                filtered_iter
            } else {
                Vec::new()
            }
        })
    }

    // Delete a message by its message ID and message type
    pub fn delete_message(message_type: &str, message_id: &str) -> Result<(), String> {
        let message_delete_result = MESSAGE_STORE.with(|store| {
            let mut store_ref = store.borrow_mut();

            // Attempt to retrieve the wrapper for the given message_type
            if let Some(wrapper) = store_ref.get(&message_type.to_string()) {
                let mut cloned_wrapper = wrapper.clone();
                cloned_wrapper.remove_message(message_id);
    
                if cloned_wrapper.is_empty() {
                    store_ref.remove(&message_type.to_string());
                } else {
                    store_ref.insert(message_type.to_string(), cloned_wrapper);
                }
                Ok(())
            } else {
                Err(format!("Message not found in MESSAGE_STORE for type '{}'", message_type))
            }
        });
    
        if let Err(_) = message_delete_result {
            HISTORY_STORE.with(|store| {
                let mut store_ref = store.borrow_mut();
    
                if let Some(wrapper) = store_ref.get(&message_type.to_string()) {
                    let mut cloned_wrapper = wrapper.clone();
                    cloned_wrapper.remove_message(message_id);
    
                    if cloned_wrapper.is_empty() {
                        store_ref.remove(&message_type.to_string());
                    } else {
                        store_ref.insert(message_type.to_string(), cloned_wrapper);
                    }
                    Ok(())
                } else {
                    Err(format!(
                        "Message not found in both MESSAGE_STORE and HISTORY_STORE for type '{}'",
                        message_type
                    ))
                }
            })
        } else {
            message_delete_result
        }
    }

    // Process the incoming message based on its payload type and message type
    pub async fn process_message(msg: Message, caller: Principal) -> Result<String, String> {
        let msg_id = msg.msg_id.clone();
    
        // Match on the payload type and decode accordingly
        match msg.payload_type.as_str() {
            "MsgUserInfo" => {
                let user_info: MsgUserInfo = msg.decode_payload()?;
                print(format!(
                    "Received user info for id {}: {:?}",
                    &msg_id, user_info
                ));
                // Use the utility function to handle operations
                handle_message_operation(&msg.msg_type, "MsgUserInfo", &msg_id, &msg, caller).await?;
            }
            "MsgUserPost" => {
                let user_post: MsgUserPost = msg.decode_payload()?;
                print(format!(
                    "Received user post for id {}: {:?}",
                    &msg_id, user_post
                ));
                // Use the utility function to handle operations
                handle_message_operation(&msg.msg_type, "MsgUserPost", &msg_id, &msg, caller).await?;
            }
            "MsgShareTrack" => {
                let share_track: MsgShareTrack = msg.decode_payload()?;
                print(format!(
                    "Received shared track for id {}: {:?}",
                    &msg_id, share_track
                ));
                // Use the utility function to handle operations
                handle_message_operation(&msg.msg_type, "MsgShareTrack", &msg_id, &msg, caller).await?;
            }
            _ => {
                return Err(format!(
                    "Unknown payload_type for id {}: {}",
                    &msg_id, msg.payload_type
                ));
            }
        }
    
        Ok(msg_id)
    }
    
    // Utility function to handle message operations (Create, Delete, Update)
    async fn handle_message_operation(
        msg_type: &MessageType,
        msg_name: &str,
        msg_id: &String,
        msg: &Message,
        caller: Principal,
    ) -> Result<(), String> {
        match msg_type {
            MessageType::Create => {
                create_message(msg_name, msg.clone(), caller);  // Clone msg when needed
            }
            MessageType::Delete => {
                delete_message(msg_name, msg_id)?;
            }
            MessageType::Update => {
                // For Update, delete the existing message first, then create a new one
                delete_message(msg_name, msg_id)?;
                create_message(msg_name, msg.clone(), caller);  // Clone msg when needed
            }
            _ => {
                return Err(format!(
                    "Unsupported message type for id {}: {:?}",
                    msg_id, msg_type
                ));
            }
        }
        Ok(())
    }

    
    
}

pub mod history {
    use super::*;

    // Retrieve a list of messages for a given message type with limit and offset
    pub fn get_history_message_list(
        message_type: &str,
        limit: usize,
        offset: usize,
    ) -> Vec<(Message, Principal)> {
        HISTORY_STORE.with(|store| {
            let store_ref = store.borrow();

            if let Some(wrapper) = store_ref.get(&message_type.to_string()) {
                wrapper
                    .into_inner()
                    .iter()
                    .skip(offset)
                    .take(limit)
                    .cloned()
                    .collect()
            } else {
                Vec::new()
            }
        })
    }
}