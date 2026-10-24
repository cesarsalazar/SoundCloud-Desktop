use std::time::Duration;

use serde::Deserialize;
use serde_json::Value;

use super::model::{AppProbe, RelayRoute, Upstream, UpstreamState};
use super::targets::app_probe;
use crate::network::edge;

const ROUTE_TIMEOUT: Duration = Duration::from_secs(10);
const ORIGINS: [&str; 4] = ["main", "star", "s3", "images"];
const MAIN: &str = "main";

#[derive(Deserialize)]
struct Seen {
    direct: u8,
    #[serde(default)]
    backup: u8,
}

pub async fn of(node: &str, client: Option<&wreq::Client>) -> Option<RelayRoute> {
    let client = client?;
    let through = format!("https://api.{node}.{}/health", edge::relay_zone());
    let (through, seen) = tokio::join!(
        app_probe(client, &through, ROUTE_TIMEOUT),
        seen(client, node)
    );
    Some(route(through, seen.as_ref()))
}

pub async fn seen(client: &wreq::Client, node: &str) -> Option<Value> {
    let url = format!("https://{node}.{}/upstreams", edge::relay_zone());
    let answer = async {
        let response = client.get(&url).send().await.ok()?;
        if !response.status().is_success() {
            return None;
        }
        response.json::<Value>().await.ok()
    };
    tokio::time::timeout(ROUTE_TIMEOUT, answer)
        .await
        .ok()
        .flatten()
}

pub fn route(through: AppProbe, seen: Option<&Value>) -> RelayRoute {
    let mut upstreams: Vec<Upstream> = ORIGINS
        .iter()
        .filter_map(|origin| {
            let seen: Seen = serde_json::from_value(seen?.get(*origin)?.clone()).ok()?;
            Some(Upstream {
                origin: origin.to_string(),
                state: state_of(&seen),
            })
        })
        .collect();
    if upstreams.first().is_none_or(|first| first.origin != MAIN) {
        upstreams.insert(
            0,
            Upstream {
                origin: MAIN.to_string(),
                state: UpstreamState::Ok,
            },
        );
    }
    if !through.ok {
        upstreams[0].state = UpstreamState::Down;
    }
    RelayRoute {
        through: Some(through),
        upstreams,
    }
}

pub fn lost_main(seen: &Value) -> bool {
    seen.get(MAIN)
        .and_then(|main| serde_json::from_value::<Seen>(main.clone()).ok())
        .is_some_and(|main| state_of(&main) == UpstreamState::Down)
}

fn state_of(seen: &Seen) -> UpstreamState {
    if seen.direct > 0 {
        UpstreamState::Ok
    } else if seen.backup > 0 {
        UpstreamState::Backup
    } else {
        UpstreamState::Down
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{lost_main, route};
    use crate::network::netcheck::model::{AppProbe, RelayRoute, UpstreamState};

    fn answered(ok: bool) -> AppProbe {
        AppProbe {
            ok,
            status: Some(if ok { 200 } else { 503 }),
            ms: Some(120),
            fail: None,
        }
    }

    fn states(route: &RelayRoute) -> Vec<(&str, UpstreamState)> {
        route
            .upstreams
            .iter()
            .map(|upstream| (upstream.origin.as_str(), upstream.state))
            .collect()
    }

    #[test]
    fn a_relay_on_its_backup_path_reads_as_backup_not_down() {
        let seen = json!({
            "node": "r2",
            "main": {"direct": 0, "backup": 1},
            "star": {"direct": 1, "backup": 1},
            "s3": {"direct": 1},
            "images": {"direct": 0}
        });
        assert_eq!(
            states(&route(answered(true), Some(&seen))),
            [
                ("main", UpstreamState::Backup),
                ("star", UpstreamState::Ok),
                ("s3", UpstreamState::Ok),
                ("images", UpstreamState::Down),
            ]
        );
    }

    #[test]
    fn main_that_did_not_answer_through_the_relay_is_down_whatever_the_relay_says() {
        let seen = json!({"main": {"direct": 1}});
        assert_eq!(
            states(&route(answered(false), Some(&seen))),
            [("main", UpstreamState::Down)]
        );
    }

    #[test]
    fn only_a_relay_with_no_path_at_all_has_lost_main() {
        assert!(lost_main(&json!({"main": {"direct": 0, "backup": 0}})));
        assert!(lost_main(&json!({"main": {"direct": 0}})));
        assert!(!lost_main(&json!({"main": {"direct": 0, "backup": 1}})));
        assert!(!lost_main(&json!({"star": {"direct": 0}})));
    }

    #[test]
    fn a_relay_without_upstreams_still_tells_about_main() {
        assert_eq!(
            states(&route(answered(true), None)),
            [("main", UpstreamState::Ok)]
        );
    }
}
