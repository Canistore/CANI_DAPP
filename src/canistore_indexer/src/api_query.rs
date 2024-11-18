use candid::Principal;
use canistore_types::message::Message;
use ic_cdk::query;

use crate::store;

#[query]
fn get_message_size() -> (Vec<(String, usize)>, usize) {
    store::message::get_message_size()
}

#[query]
fn get_message(message_type: String, message_id: String) -> Option<(Message, Principal)> {
    store::message::get_message(&message_type, &message_id)
}

#[query]
fn get_message_keys() -> Vec<String> {
    store::message::get_message_keys()
}

#[query]
fn get_message_list(
    message_type: String,
    limit: usize,
    offset: usize,
) -> Vec<(Message, Principal)> {
    store::message::get_message_list(message_type.as_str(), limit, offset)
}

#[query]
fn get_message_list_by_pid(
    message_type: String,
    pid: Principal,
    limit: usize,
    offset: usize,
) -> Vec<(Message, Principal)> {
    store::message::get_message_list_by_pid(message_type.as_str(), pid, limit, offset)
}

#[query]
fn get_history_message_list(
    message_type: String,
    limit: usize,
    offset: usize,
) -> Vec<(Message, Principal)> {
    store::history::get_history_message_list(message_type.as_str(), limit, offset)
}
