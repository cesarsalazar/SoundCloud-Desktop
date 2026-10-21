use crate::dm::direct;
use crate::support::{Client, Stack};

#[tokio::test]
#[ignore]
async fn a_new_device_sees_the_chat_but_not_the_history_before_it() {
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
    assert_eq!(ann.send(view.id, "before the new device").await, Ok(1));
    bob.sync_until("bob reads it", |bob| {
        bob.texts(view.id) == ["before the new device"]
    })
    .await;

    let laptop = Client::up(&stack, &stack.device_of(&bob.person)).await;
    assert!(laptop.engine.status().history.locked);
    assert!(laptop.engine.status().history.unlocks_at.is_some());
    laptop
        .sync_until("the new device lists the chat", |laptop| {
            laptop.conversation(view.id).is_some()
        })
        .await;
    assert_eq!(ann.send(view.id, "after the new device").await, Ok(1));
    laptop
        .sync_until("the new device reads only what came after it", |laptop| {
            let texts = laptop.texts(view.id);
            texts.last().map(String::as_str) == Some("after the new device")
                && !texts.iter().any(|text| text == "before the new device")
        })
        .await;
}
