use std::sync::Arc;

use chat_core::api::wire::LinkStart;
use chat_core::api::{BindSigner, Signer};
use chat_core::crypto::DeviceKeys;
use chat_core::{ChatError, LinkClaimed, LinkCreated, LinkMode, LinkState};
use uuid::Uuid;

use crate::support::{Client, Person, Stack};

fn fresh_device() -> BindSigner {
    BindSigner {
        device_id: Uuid::new_v4(),
        keys: Arc::new(DeviceKeys::generate().unwrap()),
        target_session: None,
    }
}

fn source_of(client: &Client) -> BindSigner {
    BindSigner {
        device_id: client.person.device_id,
        keys: client.person.keys.clone(),
        target_session: Some(client.person.session_id.to_string()),
    }
}

fn linked(owner: &Person, device: &BindSigner, session: &str) -> Person {
    Person {
        session_id: Uuid::parse_str(session).unwrap(),
        device_id: device.device_id,
        keys: device.keys.clone(),
        ..owner.clone()
    }
}

#[tokio::test]
#[ignore]
async fn an_unbound_session_reads_config_and_is_sent_to_relogin() {
    let stack = Stack::from_env();
    let owner = stack.person("old");
    let unbound = Person {
        session_id: stack.unbound_session(&owner),
        ..owner
    };
    let client = Client::on(
        vec![(chat_core::Tier::Direct, stack.origin.clone())],
        &unbound,
    );
    assert!(client.api.config().await.unwrap().enabled);
    assert_eq!(
        client.register().await.err(),
        Some(ChatError::ReloginRequired)
    );
}

#[tokio::test]
#[ignore]
async fn a_session_signed_by_another_device_is_bound_other() {
    let stack = Stack::from_env();
    let owner = stack.person("twin");
    let other = fresh_device();
    let impostor = Person {
        device_id: other.device_id,
        keys: other.keys.clone(),
        ..owner
    };
    let client = Client::on(
        vec![(chat_core::Tier::Direct, stack.origin.clone())],
        &impostor,
    );
    assert_eq!(
        client.register().await.err(),
        Some(ChatError::SessionBoundOther)
    );
}

#[tokio::test]
#[ignore]
async fn a_pull_link_claimed_by_the_signed_in_device_binds_the_new_one() {
    let stack = Stack::from_env();
    let source = Client::up(&stack, &stack.person("source")).await;
    let fresh = fresh_device();
    let created = source.api.link_create_pull(&fresh).await.unwrap();
    let claimed = source
        .api
        .link_claim_push(&source_of(&source), &created.claim_token)
        .await
        .unwrap();
    assert_eq!(claimed.mode, LinkMode::Pull);
    let polled = source
        .api
        .link_status(&fresh, &created.link_request_id)
        .await
        .unwrap();
    assert_eq!(polled.status, LinkState::Claimed);
    let session = polled
        .session_id
        .expect("the new device learns its session");

    let new_device = Client::up(&stack, &linked(&source.person, &fresh, &session)).await;
    assert_eq!(new_device.engine.devices().await.unwrap().len(), 2);
}

#[tokio::test]
#[ignore]
async fn a_push_link_from_the_signed_in_device_binds_the_new_one() {
    let stack = Stack::from_env();
    let source = Client::up(&stack, &stack.person("pusher")).await;
    let start = LinkStart {
        mode: LinkMode::Push,
    };
    let created: LinkCreated = source
        .api
        .json_as(
            &Signer::Bind(source_of(&source)),
            "POST",
            "/auth/link/create",
            "",
            Some(&start),
        )
        .await
        .unwrap();
    let fresh = fresh_device();
    let claimed: LinkClaimed = source
        .api
        .link_claim_push(&fresh, &created.claim_token)
        .await
        .unwrap();
    assert_eq!(claimed.mode, LinkMode::Push);

    let new_device = Client::up(&stack, &linked(&source.person, &fresh, &claimed.session_id)).await;
    assert_eq!(new_device.engine.devices().await.unwrap().len(), 2);
}
