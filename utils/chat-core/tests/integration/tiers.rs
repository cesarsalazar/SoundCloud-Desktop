use chat_core::Tier;

use crate::support::{Client, Stack};

const DEAD: &str = "http://127.0.0.1:9";

#[tokio::test]
#[ignore]
async fn a_dead_direct_route_falls_back_to_the_relay_and_the_relay_leads_after() {
    let stack = Stack::from_env();
    let person = stack.person("gus");
    let client = Client::on(
        vec![
            (Tier::Direct, DEAD.into()),
            (Tier::Relay, stack.origin.clone()),
        ],
        &person,
    );
    assert!(client.api.config().await.unwrap().enabled);
    client.register().await.unwrap();
    assert_eq!(client.api.tier_order().first(), Some(&Tier::Relay));
    let tiers: Vec<Tier> = client
        .transport
        .sent()
        .iter()
        .map(|(tier, _)| *tier)
        .collect();
    assert_eq!(tiers, [Tier::Direct, Tier::Relay, Tier::Relay]);
    client
        .sync_until("the sync falls through to the relay", |client| {
            client.engine.status().tier == Some(Tier::Relay)
        })
        .await;
}
