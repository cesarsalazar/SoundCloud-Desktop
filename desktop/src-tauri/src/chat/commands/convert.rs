use chat_core::store::Store;
use chat_core::{
    ChatRequest, ChatSettings, ConversationKind, ConversationSummary, Embed, EmbedKind, Invite,
    Member, MemberRole, MemberState, NotifyLevel, RoomPreview, UserCard, UserId, UserRef,
};

use super::views::{
    CardView, ConversationView, InviteView, MemberView, PersonView, PreviewView, RequestView,
    SettingsView,
};

pub const INVITE_BASE: &str = "https://getsoundcloud.app/chat/join/";
const REQUEST_MESSAGES_MAX: i64 = 3;

pub fn summary(view: &ConversationView, peer: Option<UserRef>) -> ConversationSummary {
    let me = &view.me;
    ConversationSummary {
        id: view.id,
        kind: kind(&view.kind),
        title: None,
        avatar_url: None,
        peer,
        role: role(&me.role),
        state: member_state(&me.state),
        accepted: me.accepted,
        member_count: count(view.member_count),
        last_seq: view.last_seq,
        last_at: view.last_at,
        last_read_seq: me.last_read_seq,
        unread: count(view.last_seq - me.last_read_seq),
        mentions: 0,
        preview: None,
        muted_until: me.muted_until,
        notify: notify(me.notify.as_deref()),
        pinned_at: me.pinned_at,
        archived: me.archived,
        ttl_s: view.ttl_s.and_then(|ttl| u32::try_from(ttl).ok()),
        room_code: view.room_ref.clone(),
        history_locked: false,
        has_draft: false,
    }
}

pub fn merged(
    fresh: ConversationSummary,
    stored: Option<ConversationSummary>,
) -> ConversationSummary {
    let Some(stored) = stored else {
        return fresh;
    };
    ConversationSummary {
        title: stored.title,
        avatar_url: stored.avatar_url,
        peer: fresh
            .peer
            .filter(|peer| peer.name != peer.id)
            .or(stored.peer),
        unread: stored.unread,
        mentions: stored.mentions,
        preview: stored.preview,
        history_locked: stored.history_locked,
        has_draft: stored.has_draft,
        ..fresh
    }
}

pub fn request(
    view: &RequestView,
    stored: Option<ConversationSummary>,
    from: UserRef,
) -> ChatRequest {
    let conversation = stored.unwrap_or_else(|| ConversationSummary {
        id: view.conversation_id,
        kind: kind(&view.kind),
        title: None,
        avatar_url: None,
        peer: Some(from.clone()),
        role: MemberRole::Member,
        state: MemberState::Invited,
        accepted: false,
        member_count: count(view.member_count),
        last_seq: view.last_seq,
        last_at: view.last_at,
        last_read_seq: 0,
        unread: count(view.last_seq),
        mentions: 0,
        preview: None,
        muted_until: None,
        notify: NotifyLevel::All,
        pinned_at: None,
        archived: false,
        ttl_s: None,
        room_code: None,
        history_locked: false,
        has_draft: false,
    });
    ChatRequest {
        conversation,
        from,
        messages: count(view.last_seq.clamp(0, REQUEST_MESSAGES_MAX)),
    }
}

pub fn member(view: &MemberView, user: UserRef) -> Member {
    Member {
        user,
        role: role(&view.role),
        state: member_state(&view.state),
        accepted: view.accepted,
        joined_seq: view.joined_seq,
        invited_by: view.invited_by.clone(),
    }
}

pub fn invite(view: InviteView) -> Invite {
    Invite {
        url: format!("{INVITE_BASE}{}", view.token),
        token: view.token,
        expires_at: view.expires_at,
        uses_left: view.uses_left.and_then(|uses| u32::try_from(uses).ok()),
    }
}

pub fn settings(view: SettingsView) -> ChatSettings {
    ChatSettings {
        dm_policy: view.dm_policy,
        read_receipts: view.read_receipts,
        typing: view.typing,
        presence_share: view.presence_share,
        blocked: view.blocked.into_iter().map(person).collect(),
    }
}

pub fn card(view: CardView) -> UserCard {
    UserCard {
        can_message: !view.can_message.is_empty() && view.can_message != "unavailable",
        user: person(view.person),
        contact: view.contact,
        blocked: view.blocked,
        presence: None,
    }
}

pub fn preview(view: PreviewView) -> RoomPreview {
    RoomPreview {
        code: view.code,
        title: view.title,
        host: person(view.host),
        track: view.track.map(|track| Embed {
            kind: EmbedKind::Track,
            urn: track.urn,
            permalink: String::new(),
            secret_token: None,
            title: track.title,
            artist: track.artist,
            artwork_url: track.artwork_url,
            duration_ms: track.duration_ms.and_then(|ms| u64::try_from(ms).ok()),
            access: None,
            at_ms: None,
            tracks: Vec::new(),
            room: None,
        }),
        listeners: view.listeners,
        alive: view.alive,
    }
}

pub fn person(view: PersonView) -> UserRef {
    UserRef {
        name: view.username.unwrap_or_else(|| view.user_id.clone()),
        id: view.user_id,
        avatar_url: view.avatar_url,
        permalink: view.permalink_url,
    }
}

pub fn people(store: &Store, ids: &[UserId]) -> Vec<UserRef> {
    let known = store.users(ids).unwrap_or_default();
    ids.iter()
        .map(|id| {
            known
                .iter()
                .find(|user| &user.id == id)
                .cloned()
                .unwrap_or_else(|| bare(id))
        })
        .collect()
}

pub fn bare(id: &str) -> UserRef {
    UserRef {
        id: id.to_string(),
        name: id.to_string(),
        avatar_url: None,
        permalink: None,
    }
}

pub fn kind(raw: &str) -> ConversationKind {
    match raw {
        "direct" => ConversationKind::Direct,
        "room" => ConversationKind::Room,
        "self" => ConversationKind::Saved,
        _ => ConversationKind::Group,
    }
}

pub fn role(raw: &str) -> MemberRole {
    match raw {
        "owner" => MemberRole::Owner,
        "admin" => MemberRole::Admin,
        _ => MemberRole::Member,
    }
}

pub fn member_state(raw: &str) -> MemberState {
    match raw {
        "invited" => MemberState::Invited,
        "left" => MemberState::Left,
        "kicked" => MemberState::Kicked,
        "banned" => MemberState::Banned,
        _ => MemberState::Active,
    }
}

fn notify(raw: Option<&str>) -> NotifyLevel {
    match raw {
        Some("mentions") => NotifyLevel::Mentions,
        Some("none") => NotifyLevel::None,
        _ => NotifyLevel::All,
    }
}

fn count(value: impl TryInto<u32>) -> u32 {
    value.try_into().unwrap_or(0)
}
