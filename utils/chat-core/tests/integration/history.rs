use chat_core::{HistoryQuery, MessageBody};

use crate::dm::direct;
use crate::support::{Client, Stack};

#[tokio::test]
#[ignore]
async fn a_new_device_reads_old_history_only_after_an_older_device_says_it_is_me() {
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
    stack.age_device(&bob.person, 73);
    bob.register().await.unwrap();

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

    assert!(
        laptop
            .api
            .confirm_device(bob.person.device_id)
            .await
            .is_err()
    );
    bob.api
        .confirm_device(laptop.person.device_id)
        .await
        .expect("a seasoned device confirms the new one");
    laptop
        .sync_until("the history gate opens on the new device", |laptop| {
            !laptop.engine.status().history.locked
        })
        .await;
    let page = laptop
        .engine
        .fetch_history(
            view.id,
            &HistoryQuery {
                before: None,
                after: None,
                limit: 50,
            },
        )
        .await
        .unwrap();
    assert!(page.messages.iter().any(|message| matches!(
        &message.body,
        MessageBody::Text(rich) if rich.text == "before the new device"
    )));
    assert_eq!(
        laptop.texts(view.id),
        ["before the new device", "after the new device"]
    );
}
