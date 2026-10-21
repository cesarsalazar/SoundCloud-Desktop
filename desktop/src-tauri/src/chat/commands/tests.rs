use chat_core::api::wire::DeviceView;
use chat_core::{
    ConversationKind, DEVICE_AGE_RULE_MS, DeviceStatus, DmPolicy, MemberRole, MemberState, Mute,
    MyPatch, NotifyLevel, RichText, Span, SpanKind, TEXT_MAX, UserRef,
};
use serde_json::json;
use uuid::Uuid;

use super::compose::{check_text, mentions};
use super::conversations::my_patch;
use super::convert;
use super::devices::actions;
use super::views::{
    CardView, ConversationView, MemberView, PreviewView, RequestsView, SettingsView,
};

const HOUR_MS: i64 = 3_600_000;

fn device(created_at: i64, current: bool, unlocked: bool) -> DeviceView {
    DeviceView {
        id: Uuid::new_v4(),
        platform: "macos".into(),
        name: "macOS".into(),
        app_version: "8.6.0".into(),
        tier: "relay".into(),
        status: DeviceStatus::Active,
        created_at,
        last_seen_at: created_at,
        history_unlocked: unlocked,
        current,
    }
}

#[test]
fn an_older_device_can_vouch_for_a_new_one() {
    let me = device(0, true, true);
    let newcomer = device(10 * HOUR_MS, false, false);
    let shown = actions(&newcomer, Some(&me), 11 * HOUR_MS);
    assert!(shown.confirm && shown.not_me && shown.remove);
    let mine = actions(&me, Some(&me), 11 * HOUR_MS);
    assert!(mine.remove && !mine.confirm && !mine.not_me);
}

#[test]
fn a_young_device_cannot_touch_an_older_one_for_three_days() {
    let me = device(10 * HOUR_MS, true, false);
    let older = device(0, false, true);
    let young = actions(&older, Some(&me), 20 * HOUR_MS);
    assert!(!young.confirm && !young.not_me && !young.remove);
    let settled = actions(&older, Some(&me), 10 * HOUR_MS + DEVICE_AGE_RULE_MS);
    assert!(settled.not_me && settled.remove && !settled.confirm);
}

#[test]
fn a_revoked_device_offers_nothing() {
    let me = device(0, true, true);
    let revoked = DeviceView {
        status: DeviceStatus::Revoked,
        ..device(HOUR_MS, false, false)
    };
    let shown = actions(&revoked, Some(&me), 2 * HOUR_MS);
    assert!(!shown.confirm && !shown.not_me && !shown.remove);
}

#[test]
fn mentions_come_from_mention_spans_once_each() {
    let span = |kind, user: Option<&str>| Span {
        kind,
        start: 0,
        len: 1,
        url: None,
        user_id: user.map(str::to_string),
    };
    let text = RichText {
        text: "@a @b @a".into(),
        spans: vec![
            span(SpanKind::Mention, Some("1")),
            span(SpanKind::Bold, None),
            span(SpanKind::Mention, Some("2")),
            span(SpanKind::Mention, Some("1")),
        ],
    };
    assert_eq!(mentions(&text), ["1", "2"]);
}

#[test]
fn text_is_capped_in_utf16_units() {
    let long = RichText {
        text: "я".repeat(TEXT_MAX),
        spans: Vec::new(),
    };
    assert!(check_text(&long).is_ok());
    let longer = RichText {
        text: "😀".repeat(TEXT_MAX / 2 + 1),
        spans: Vec::new(),
    };
    assert!(check_text(&longer).is_err());
}

#[test]
fn a_mute_patch_sends_null_to_unmute_and_skips_untouched_fields() {
    let unmute = MyPatch {
        mute: Some(Mute::Off),
        notify: None,
        pinned: None,
        archived: None,
    };
    assert_eq!(
        serde_json::to_value(my_patch(unmute)).unwrap(),
        json!({ "muted_until": null })
    );
    let pin = MyPatch {
        mute: None,
        notify: Some(NotifyLevel::Mentions),
        pinned: Some(true),
        archived: None,
    };
    assert_eq!(
        serde_json::to_value(my_patch(pin)).unwrap(),
        json!({ "notify": "mentions", "pinned": true })
    );
}

fn conversation_view() -> ConversationView {
    serde_json::from_value(json!({
        "id": "0192e5c5-1a2b-7c3d-8e4f-5a6b7c8d9e0f",
        "kind": "direct",
        "peer": "42",
        "created_by": "7",
        "created_at": 1,
        "last_seq": 9,
        "last_at": 1_000,
        "key_epoch": 2,
        "meta_ct": null,
        "meta_epoch": null,
        "settings": {},
        "member_count": 2,
        "ttl_s": 86_400,
        "room_ref": null,
        "me": {
            "role": "owner",
            "state": "active",
            "accepted": true,
            "joined_seq": 0,
            "joined_epoch": 0,
            "last_read_seq": 4,
            "muted_until": null,
            "notify": "mentions",
            "pinned_at": 5,
            "archived": false,
            "hidden_before_seq": 0,
            "invited_by": null
        }
    }))
    .unwrap()
}

#[test]
fn a_server_conversation_becomes_a_summary() {
    let peer = convert::bare("42");
    let summary = convert::summary(&conversation_view(), Some(peer.clone()));
    assert_eq!(summary.kind, ConversationKind::Direct);
    assert_eq!(summary.role, MemberRole::Owner);
    assert_eq!(summary.unread, 5);
    assert_eq!(summary.notify, NotifyLevel::Mentions);
    assert_eq!(summary.ttl_s, Some(86_400));
    assert_eq!(summary.pinned_at, Some(5));
    assert_eq!(summary.peer, Some(peer));
}

#[test]
fn a_merged_summary_keeps_what_only_the_device_knows() {
    let fresh = convert::summary(&conversation_view(), Some(convert::bare("42")));
    let stored = chat_core::ConversationSummary {
        title: Some("Lena".into()),
        unread: 1,
        peer: Some(UserRef {
            id: "42".into(),
            name: "Lena".into(),
            avatar_url: None,
            permalink: None,
        }),
        has_draft: true,
        ..fresh.clone()
    };
    let merged = convert::merged(fresh, Some(stored));
    assert_eq!(merged.title.as_deref(), Some("Lena"));
    assert_eq!(merged.peer.unwrap().name, "Lena");
    assert_eq!(merged.unread, 1);
    assert!(merged.has_draft);
    assert_eq!(merged.ttl_s, Some(86_400));
}

#[test]
fn members_settings_cards_and_requests_parse_from_the_server() {
    let member: MemberView = serde_json::from_value(json!({
        "user_id": "42", "role": "admin", "state": "invited",
        "accepted": false, "joined_seq": 3, "invited_by": "7"
    }))
    .unwrap();
    let member = convert::member(&member, convert::bare("42"));
    assert_eq!(member.role, MemberRole::Admin);
    assert_eq!(member.state, MemberState::Invited);
    assert_eq!(member.invited_by.as_deref(), Some("7"));

    let settings: SettingsView = serde_json::from_value(json!({
        "dm_policy": "everyone", "read_receipts": false, "typing": true,
        "presence_share": false, "restricted_until": null, "restriction_scope": [],
        "blocked": [{ "user_id": "9", "username": "spam", "avatar_url": null, "at": 1 }]
    }))
    .unwrap();
    let settings = convert::settings(settings);
    assert_eq!(settings.dm_policy, DmPolicy::Everyone);
    assert_eq!(settings.blocked[0].name, "spam");

    let card: CardView = serde_json::from_value(json!({
        "user_id": "42", "username": "lena", "avatar_url": null,
        "permalink_url": "https://soundcloud.com/lena", "verified": false,
        "blocked": false, "contact": true, "can_message": "request"
    }))
    .unwrap();
    let card = convert::card(card);
    assert!(card.can_message && card.contact);
    assert_eq!(
        card.user.permalink.as_deref(),
        Some("https://soundcloud.com/lena")
    );
    let closed: CardView = serde_json::from_value(json!({
        "user_id": "42", "can_message": "unavailable"
    }))
    .unwrap();
    assert!(!convert::card(closed).can_message);

    let requests: RequestsView = serde_json::from_value(json!({
        "requests": [{
            "conversation_id": "0192e5c5-1a2b-7c3d-8e4f-5a6b7c8d9e0f",
            "kind": "direct", "from": "42", "since": 1, "last_seq": 7,
            "last_at": 2, "member_count": 2, "meta_ct": null, "meta_epoch": null
        }],
        "count": 1
    }))
    .unwrap();
    let request = convert::request(&requests.requests[0], None, convert::bare("42"));
    assert_eq!(request.messages, 3);
    assert!(!request.conversation.accepted);
    assert_eq!(request.conversation.state, MemberState::Invited);
}

#[test]
fn a_room_preview_keeps_its_host_and_track() {
    let preview: PreviewView = serde_json::from_value(json!({
        "code": "ABCD", "title": "night", "listeners": 3, "capacity": 50,
        "public": true, "alive": true,
        "host": { "id": "7", "name": "dj", "avatar_url": null },
        "track": { "urn": "soundcloud:tracks:1", "title": "Song", "artist": "Band",
                   "artwork_url": null, "duration_ms": 1_000 }
    }))
    .unwrap();
    let preview = convert::preview(preview);
    assert_eq!(preview.host.id, "7");
    assert_eq!(preview.host.name, "dj");
    let track = preview.track.unwrap();
    assert_eq!(track.duration_ms, Some(1_000));
    assert_eq!(track.artist.as_deref(), Some("Band"));
}

#[test]
fn an_invite_link_carries_its_token() {
    let invite = convert::invite(
        serde_json::from_value(json!({
            "token": "tok", "conversation_id": "0192e5c5-1a2b-7c3d-8e4f-5a6b7c8d9e0f",
            "expires_at": 9, "uses_left": 5
        }))
        .unwrap(),
    );
    assert_eq!(invite.url, format!("{}tok", convert::INVITE_BASE));
    assert_eq!(invite.uses_left, Some(5));
}
