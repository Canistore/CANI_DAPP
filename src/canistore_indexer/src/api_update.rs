use canistore_types::message::Message;
use ic_cdk::update;

use crate::store;

#[update]
async fn receive_message(msg: Message) -> Result<String, String> {
    store::message::process_message(msg, ic_cdk::caller()).await
}

#[update]
async fn receive_batch_messages(messages: Vec<Message>) -> Result<usize, String> {
    let mut success_count = 0;
    let caller = ic_cdk::caller();

    for msg in messages {
        match store::message::process_message(msg, caller).await {
            Ok(_) => success_count += 1,
            Err(err) => return Err(err),
        }
    }

    Ok(success_count)
}
