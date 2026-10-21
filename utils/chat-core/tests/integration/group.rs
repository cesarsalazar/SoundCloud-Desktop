use chat_core::api::wire::{MemberChange, NewConversation};
use chat_core::{ConversationKind, MemberRole, MessageBody, SystemKind};

use crate::support::{Client, Stack};

#[tokio::test]
#[ignore]
async fn a_group_reaches_every_member_and_a_promotion_shows_as_a_role_row() {
    let stack = Stack::from_env();
    let ann = Client::up(&stack, &stack.person("ann")).await;
    let bob = Client::up(&stack, &stack.person("bob")).await;
    let cid = Client::up(&stack, &stack.person("cid")).await;
    stack.follow(&bob.person, &ann.person);
    stack.follow(&cid.person, &ann.person);
    let view = ann
        .api
        .create_conversation(&NewConversation {
            kind: ConversationKind::Group,
            members: vec![bob.person.user_id.clone(), cid.person.user_id.clone()],
            meta_ct: None,
            meta_epoch: None,
        })
        .await
        .unwrap();
    ann.engine.adopt(&view).await.unwrap();
    assert_eq!(ann.send(view.id, "welcome").await, Ok(1));

    for member in [&bob, &cid] {
        member
            .sync_until("a member gets the first post", |client| {
                client.texts(view.id) == ["welcome"]
            })
            .await;
    }
    assert_eq!(cid.send(view.id, "thanks").await, Ok(1));
    for member in [&ann, &bob] {
        member
            .sync_until("the answer reaches the rest", |client| {
                client.texts(view.id) == ["welcome", "thanks"]
            })
            .await;
    }

    let promoted = MemberChange {
        role: Some(MemberRole::Admin),
        unban: false,
    };
    ann.api
        .update_member(view.id, &bob.person.user_id, &promoted)
        .await
        .unwrap();
    let bob_id = bob.person.user_id.clone();
    cid.sync_until("cid sees bob promoted", |cid| {
        cid.messages(view.id)
            .iter()
            .any(|message| match &message.body {
                MessageBody::System(event) => {
                    event.kind == SystemKind::Role
                        && event.role == Some(MemberRole::Admin)
                        && event.target.as_deref() == Some(bob_id.as_str())
                }
                _ => false,
            })
    })
    .await;
}
