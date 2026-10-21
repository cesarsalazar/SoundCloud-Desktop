use std::net::SocketAddr;
use std::time::Duration;

use chat_core::{Tier, Transport, TransportError, TransportRequest};
use warp::Filter;
use warp::http::{Response, StatusCode};

use super::client::{lost_on_the_way, seconds_to_ms};
use super::hops::{Hop, hops_over, order};
use super::{EdgeTransport, Route};

const CHAT_URL: &str = "https://api.scnative.dev/chat/x";

fn request(tier: Tier, timeout_ms: u64) -> TransportRequest {
    TransportRequest {
        tier,
        envelope: vec![1, 2, 3, 4],
        timeout_ms,
        long_poll: true,
    }
}

async fn serve(status: u16, retry_after: Option<&'static str>, delay: Duration) -> SocketAddr {
    let route = warp::post()
        .and(warp::path!("chat" / "x"))
        .and(warp::body::bytes())
        .then(move |body: warp::hyper::body::Bytes| async move {
            tokio::time::sleep(delay).await;
            let mut answer = Response::builder()
                .status(StatusCode::from_u16(status).unwrap())
                .header("content-type", "application/octet-stream");
            if let Some(seconds) = retry_after {
                answer = answer.header("retry-after", seconds);
            }
            let mut echoed = body.to_vec();
            echoed.reverse();
            answer.body(echoed).unwrap()
        });
    let (addr, server) = warp::serve(route).bind_ephemeral(([127, 0, 0, 1], 0));
    tokio::spawn(server);
    addr
}

fn fixed(addr: SocketAddr) -> EdgeTransport {
    EdgeTransport::new(Route::fixed(&format!("http://{addr}")))
}

#[test]
fn the_edge_level_goes_first_and_the_rest_keep_their_order() {
    assert_eq!(order(Tier::Direct), [Tier::Direct, Tier::Relay, Tier::Pro]);
    assert_eq!(order(Tier::Relay), [Tier::Relay, Tier::Direct, Tier::Pro]);
    assert_eq!(order(Tier::Pro), [Tier::Pro, Tier::Direct, Tier::Relay]);
}

#[test]
fn every_tier_points_at_the_sealed_channel() {
    let relays = vec![
        "api.r1.relay.scnative.dev".to_string(),
        "api.r2.relay.scnative.dev".to_string(),
    ];
    let pros = vec!["p1.pro.scnative.dev".to_string()];
    assert_eq!(
        hops_over(CHAT_URL, Tier::Direct, &relays, &pros),
        [Hop::Http(CHAT_URL.to_string())]
    );
    assert_eq!(
        hops_over(CHAT_URL, Tier::Relay, &relays, &pros),
        [
            Hop::Http("https://api.r1.relay.scnative.dev/chat/x".to_string()),
            Hop::Http("https://api.r2.relay.scnative.dev/chat/x".to_string()),
        ]
    );
    assert_eq!(
        hops_over(CHAT_URL, Tier::Pro, &relays, &pros),
        [Hop::Pro {
            node: "https://p1.pro.scnative.dev".to_string(),
            url: CHAT_URL.to_string(),
        }]
    );
}

#[test]
fn the_edge_route_offers_all_three_tiers_on_the_api_origin() {
    let route = Route::edge();
    let mut tiers = route.tiers();
    tiers.sort_by_key(|tier| *tier as u8);
    assert_eq!(tiers, [Tier::Direct, Tier::Relay, Tier::Pro]);
    assert_eq!(route.hops(Tier::Direct), [Hop::Http(CHAT_URL.to_string())]);
    assert!(!route.hops(Tier::Relay).is_empty());
    assert!(!route.hops(Tier::Pro).is_empty());
}

#[test]
fn a_fixed_dev_origin_is_direct_only() {
    let route = Route::fixed("http://127.0.0.1:3000/");
    assert_eq!(route.tiers(), [Tier::Direct]);
    assert_eq!(
        route.hops(Tier::Direct),
        [Hop::Http("http://127.0.0.1:3000/chat/x".to_string())]
    );
    assert!(route.hops(Tier::Relay).is_empty());
}

#[test]
fn gateway_errors_without_retry_after_mean_the_hop_is_lost() {
    assert!(lost_on_the_way(421, None));
    assert!(lost_on_the_way(502, None));
    assert!(lost_on_the_way(504, None));
    assert!(!lost_on_the_way(503, Some(2_000)));
    assert!(!lost_on_the_way(200, None));
    assert!(!lost_on_the_way(400, None));
    assert_eq!(seconds_to_ms(" 3 "), Some(3_000));
    assert_eq!(seconds_to_ms("soon"), None);
}

#[tokio::test]
async fn an_answer_comes_back_whole_with_its_status() {
    let addr = serve(200, None, Duration::ZERO).await;
    let answer = fixed(addr)
        .exchange(request(Tier::Direct, 5_000))
        .await
        .unwrap();
    assert_eq!(answer.status, 200);
    assert_eq!(answer.body, [4, 3, 2, 1]);
    assert_eq!(answer.retry_after_ms, None);
}

#[tokio::test]
async fn busy_keeps_its_retry_after() {
    let addr = serve(503, Some("2"), Duration::ZERO).await;
    let answer = fixed(addr)
        .exchange(request(Tier::Direct, 5_000))
        .await
        .unwrap();
    assert_eq!(answer.status, 503);
    assert_eq!(answer.retry_after_ms, Some(2_000));
}

#[tokio::test]
async fn a_gateway_page_counts_as_unreachable() {
    let addr = serve(502, None, Duration::ZERO).await;
    let failed = fixed(addr).exchange(request(Tier::Direct, 5_000)).await;
    assert_eq!(failed, Err(TransportError::Unreachable));
}

#[tokio::test]
async fn a_slow_answer_times_out_at_the_deadline() {
    let addr = serve(200, None, Duration::from_secs(5)).await;
    let failed = fixed(addr).exchange(request(Tier::Direct, 300)).await;
    assert_eq!(failed, Err(TransportError::Timeout));
}

#[tokio::test]
async fn a_closed_port_is_unreachable() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    let failed = fixed(addr).exchange(request(Tier::Direct, 5_000)).await;
    assert_eq!(failed, Err(TransportError::Unreachable));
}

#[tokio::test]
async fn a_tier_the_route_lacks_is_unreachable() {
    let addr = serve(200, None, Duration::ZERO).await;
    let failed = fixed(addr).exchange(request(Tier::Relay, 5_000)).await;
    assert_eq!(failed, Err(TransportError::Unreachable));
}
