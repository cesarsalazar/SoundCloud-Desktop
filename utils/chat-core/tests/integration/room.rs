use serde_json::json;

use crate::support::{Client, Stack};

#[tokio::test]
#[ignore]
async fn a_room_chat_lives_while_people_listen_and_is_purged_locally_when_the_room_ends() {
    let stack = Stack::from_env();
    let ann = Client::up(&stack, &stack.person("ann")).await;
    let bob = Client::up(&stack, &stack.person("bob")).await;
    let created = stack
        .as_session(
            ann.person.session_id,
            "POST",
            "/rooms",
            Some(json!({"name": "ann", "public": false})),
        )
        .await;
    assert_eq!(
        created.status,
        201,
        "{}",
        String::from_utf8_lossy(&created.body)
    );
    let code = created.json()["code"].as_str().unwrap().to_owned();
    let joined = stack
        .as_session(
            bob.person.session_id,
            "POST",
            &format!("/rooms/{code}/members"),
            Some(json!({"name": "bob"})),
        )
        .await;
    assert_eq!(
        joined.status,
        200,
        "{}",
        String::from_utf8_lossy(&joined.body)
    );

    let room = ann.api.room_chat(&code).await.unwrap();
    assert_eq!(
        bob.api.room_chat(&code).await.unwrap().conversation_id,
        room.conversation_id
    );
    let chat = room.conversation_id;
    for client in [&ann, &bob] {
        client
            .engine
            .adopt_room(chat, &code, room.from_seq)
            .await
            .unwrap();
    }
    assert_eq!(ann.send(chat, "room hello").await, Ok(1));
    bob.sync_until("bob reads the room", |bob| {
        bob.texts(chat).iter().any(|text| text == "room hello")
    })
    .await;

    for client in [&bob, &ann] {
        let left = stack
            .as_session(
                client.person.session_id,
                "DELETE",
                &format!("/rooms/{code}/members/me"),
                None,
            )
            .await;
        assert!(left.status < 300, "{}", left.status);
    }
    for client in [&ann, &bob] {
        client
            .sync_until("the ended room chat is purged on the device", |client| {
                client.conversation(chat).is_none() && client.messages(chat).is_empty()
            })
            .await;
        assert!(client.events.gone().contains(&chat));
    }
}
