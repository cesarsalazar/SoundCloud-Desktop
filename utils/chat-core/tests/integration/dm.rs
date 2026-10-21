use chat_core::ConversationKind;
use chat_core::DeliveryState;
use chat_core::api::wire::NewConversation;

use crate::support::{Client, Stack};

pub fn direct(with: &str) -> NewConversation {
    NewConversation {
        kind: ConversationKind::Direct,
        members: vec![with.to_owned()],
        meta_ct: None,
        meta_epoch: None,
    }
}

#[tokio::test]
#[ignore]
async fn a_dm_carries_text_both_ways_and_marks_it_read() {
    let stack = Stack::from_env();
    let ann = Client::up(&stack, &stack.person("ann")).await;
    let bob = Client::up(&stack, &stack.person("bob")).await;
    stack.follow(&bob.person, &ann.person);
    let view = ann
        .api
        .create_conversation(&direct(&bob.person.user_id))
        .await
        .unwrap();
    ann.engine.adopt(&view).await.unwrap();
    ann.send(view.id, "hi bob").await.unwrap();

    bob.sync_until("bob gets the chat", |bob| bob.texts(view.id) == ["hi bob"])
        .await;
    bob.send(view.id, "hi ann").await.unwrap();
    ann.sync_until("ann gets the answer", |ann| {
        ann.texts(view.id) == ["hi bob", "hi ann"]
    })
    .await;

    let seen = bob.messages(view.id);
    assert!(!seen[0].mine && seen[1].mine);
    bob.engine.queue_read(view.id, seen[1].seq.unwrap());
    bob.sync().await.unwrap();
    ann.sync_until("ann sees bob read her message", |ann| {
        ann.messages(view.id)[0].state == DeliveryState::Read
    })
    .await;
}
