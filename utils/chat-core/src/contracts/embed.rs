use serde::{Deserialize, Serialize};

pub const EMBED_BYTES_MAX: usize = 3 * 1024;
pub const EMBED_ARTWORK_HOST_SUFFIX: &str = ".sndcdn.com";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmbedKind {
    Track,
    Playlist,
    User,
    RoomInvite,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbedTrack {
    pub urn: String,
    pub title: String,
    pub artist: String,
    pub duration_ms: u64,
    pub artwork_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomInvite {
    pub code: String,
    pub title: Option<String>,
    pub host: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Embed {
    pub kind: EmbedKind,
    pub urn: String,
    pub permalink: String,
    pub secret_token: Option<String>,
    pub title: String,
    pub artist: Option<String>,
    pub artwork_url: Option<String>,
    pub duration_ms: Option<u64>,
    pub access: Option<String>,
    pub at_ms: Option<u64>,
    pub tracks: Vec<EmbedTrack>,
    pub room: Option<RoomInvite>,
}
