use chat_core::Tier;
use chat_core::channel::CHANNEL_PATH;

use crate::network::edge;

const API_ORIGIN: &str = "https://api.scnative.dev";
const ORIGIN_ENV: &str = "SC_CHAT_ORIGIN";
const NATURAL: [Tier; 3] = [Tier::Direct, Tier::Relay, Tier::Pro];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hop {
    Http(String),
    Pro { node: String, url: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    Edge(String),
    Fixed(String),
}

impl Route {
    pub fn edge() -> Self {
        Self::Edge(channel_url(API_ORIGIN))
    }

    pub fn fixed(origin: &str) -> Self {
        Self::Fixed(channel_url(origin))
    }

    pub fn from_env() -> Self {
        match std::env::var(ORIGIN_ENV) {
            Ok(origin) if cfg!(debug_assertions) && !origin.trim().is_empty() => {
                Self::fixed(origin.trim())
            }
            _ => Self::edge(),
        }
    }

    pub fn tiers(&self) -> Vec<Tier> {
        match self {
            Self::Fixed(_) => vec![Tier::Direct],
            Self::Edge(url) => order(chat_tier(edge::current_tier(url))),
        }
    }

    pub fn hops(&self, tier: Tier) -> Vec<Hop> {
        match self {
            Self::Fixed(url) if tier == Tier::Direct => vec![Hop::Http(url.clone())],
            Self::Fixed(_) => Vec::new(),
            Self::Edge(url) => {
                let relays = edge::relay_hosts(&host_of(url));
                hops_over(url, tier, &relays, &edge::pro_hosts())
            }
        }
    }
}

pub fn order(level: Tier) -> Vec<Tier> {
    std::iter::once(level)
        .chain(NATURAL.into_iter().filter(|tier| *tier != level))
        .collect()
}

pub fn hops_over(url: &str, tier: Tier, relays: &[String], pros: &[String]) -> Vec<Hop> {
    match tier {
        Tier::Direct => vec![Hop::Http(url.to_string())],
        Tier::Relay => relays
            .iter()
            .filter_map(|relay| swap_host(url, relay))
            .map(Hop::Http)
            .collect(),
        Tier::Pro => pros
            .iter()
            .map(|node| Hop::Pro {
                node: format!("https://{node}"),
                url: url.to_string(),
            })
            .collect(),
    }
}

fn chat_tier(tier: edge::Tier) -> Tier {
    match tier {
        edge::Tier::Direct => Tier::Direct,
        edge::Tier::Relay => Tier::Relay,
        edge::Tier::Pro => Tier::Pro,
    }
}

fn channel_url(origin: &str) -> String {
    format!("{}{CHANNEL_PATH}", origin.trim_end_matches('/'))
}

fn host_of(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|url| url.host_str().map(str::to_ascii_lowercase))
        .unwrap_or_default()
}

fn swap_host(url: &str, host: &str) -> Option<String> {
    let mut url = url::Url::parse(url).ok()?;
    url.set_host(Some(host)).ok()?;
    Some(url.to_string())
}
