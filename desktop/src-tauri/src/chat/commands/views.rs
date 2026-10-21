use serde::Deserialize;
use uuid::Uuid;

use chat_core::DmPolicy;

#[derive(Debug, Clone, Deserialize)]
pub struct ConversationView {
    pub id: Uuid,
    pub kind: String,
    #[serde(default)]
    pub peer: Option<String>,
    #[serde(default)]
    pub last_seq: i64,
    #[serde(default)]
    pub last_at: i64,
    #[serde(default)]
    pub member_count: i32,
    #[serde(default)]
    pub ttl_s: Option<i32>,
    #[serde(default)]
    pub room_ref: Option<String>,
    pub me: MembershipView,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MembershipView {
    pub role: String,
    pub state: String,
    #[serde(default)]
    pub accepted: bool,
    #[serde(default)]
    pub last_read_seq: i64,
    #[serde(default)]
    pub muted_until: Option<i64>,
    #[serde(default)]
    pub notify: Option<String>,
    #[serde(default)]
    pub pinned_at: Option<i64>,
    #[serde(default)]
    pub archived: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MemberView {
    pub user_id: String,
    pub role: String,
    pub state: String,
    #[serde(default)]
    pub accepted: bool,
    #[serde(default)]
    pub joined_seq: i64,
    #[serde(default)]
    pub invited_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InviteView {
    pub token: String,
    pub expires_at: i64,
    #[serde(default)]
    pub uses_left: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JoinedView {
    pub conversation_id: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RequestsView {
    #[serde(default)]
    pub requests: Vec<RequestView>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RequestView {
    pub conversation_id: Uuid,
    pub kind: String,
    pub from: String,
    #[serde(default)]
    pub last_seq: i64,
    #[serde(default)]
    pub last_at: i64,
    #[serde(default)]
    pub member_count: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsView {
    pub dm_policy: DmPolicy,
    pub read_receipts: bool,
    pub typing: bool,
    pub presence_share: bool,
    #[serde(default)]
    pub blocked: Vec<PersonView>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PersonView {
    #[serde(alias = "id")]
    pub user_id: String,
    #[serde(default, alias = "name")]
    pub username: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub permalink_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CardView {
    #[serde(flatten)]
    pub person: PersonView,
    #[serde(default)]
    pub blocked: bool,
    #[serde(default)]
    pub contact: bool,
    #[serde(default)]
    pub can_message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RoomChatView {
    pub conversation_id: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PreviewView {
    pub code: String,
    #[serde(default)]
    pub title: Option<String>,
    pub host: PersonView,
    #[serde(default)]
    pub track: Option<PreviewTrack>,
    #[serde(default)]
    pub listeners: u32,
    #[serde(default)]
    pub alive: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PreviewTrack {
    pub urn: String,
    pub title: String,
    #[serde(default)]
    pub artist: Option<String>,
    #[serde(default)]
    pub artwork_url: Option<String>,
    #[serde(default)]
    pub duration_ms: Option<i64>,
}
