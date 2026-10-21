use chat_wire::prost::Message as _;
use chat_wire::proto::{self, envelope::Body};

use crate::contracts::{
    ConversationMeta, Embed, EmbedKind, EmbedTrack, Forwarded, RichText, RoomInvite, Span, SpanKind,
};
use crate::envelope::{Content, Post};

pub fn decode(bytes: &[u8]) -> Content {
    match proto::Envelope::decode(bytes) {
        Ok(envelope) => content(envelope),
        Err(_) => Content::Unknown,
    }
}

fn content(envelope: proto::Envelope) -> Content {
    match envelope.body {
        Some(Body::Text(text)) => Content::Post(Post {
            text: rich(text),
            reply_to: positive(envelope.reply_to),
            mentions: envelope.mentions,
            embeds: envelope.embeds.into_iter().filter_map(embed).collect(),
            forwarded_from: envelope.forwarded_from.map(forwarded),
        }),
        Some(Body::Edit(edit)) => Content::Edit {
            target: edit.target as i64,
            text: edit.text.map(rich).unwrap_or_default(),
        },
        Some(Body::Delete(delete)) => Content::Delete {
            target: delete.target as i64,
        },
        Some(Body::Reaction(reaction)) => Content::Reaction {
            target: reaction.target as i64,
            emoji: reaction.emoji,
            on: reaction.on,
        },
        Some(Body::Meta(meta)) => Content::Meta(ConversationMeta {
            title: meta.title,
            description: meta.description,
            avatar_media_id: filled(meta.avatar_media_id),
        }),
        Some(Body::Pin(pin)) => Content::Pin {
            target: pin.target as i64,
            on: pin.on,
        },
        Some(Body::Suggest(suggest)) => suggest
            .track
            .and_then(|track| embed(*track))
            .map(Content::Suggest)
            .unwrap_or(Content::Unknown),
        None => Content::Unknown,
    }
}

fn rich(text: proto::Text) -> RichText {
    RichText {
        text: text.text,
        spans: text.spans.into_iter().filter_map(span).collect(),
    }
}

fn span(span: proto::Span) -> Option<Span> {
    let kind = match proto::SpanKind::try_from(span.kind).ok()? {
        proto::SpanKind::Bold => SpanKind::Bold,
        proto::SpanKind::Italic => SpanKind::Italic,
        proto::SpanKind::Code => SpanKind::Code,
        proto::SpanKind::Spoiler => SpanKind::Spoiler,
        proto::SpanKind::Link => SpanKind::Link,
        proto::SpanKind::Mention => SpanKind::Mention,
        proto::SpanKind::Unspecified => return None,
    };
    Some(Span {
        kind,
        start: span.start,
        len: span.len,
        url: filled(span.url),
        user_id: filled(span.user_id),
    })
}

fn embed(embed: proto::Embed) -> Option<Embed> {
    let kind = match proto::EmbedKind::try_from(embed.kind).ok()? {
        proto::EmbedKind::Track => EmbedKind::Track,
        proto::EmbedKind::Playlist => EmbedKind::Playlist,
        proto::EmbedKind::User => EmbedKind::User,
        proto::EmbedKind::RoomInvite => EmbedKind::RoomInvite,
        proto::EmbedKind::Unspecified => return None,
    };
    Some(Embed {
        kind,
        urn: embed.urn,
        permalink: embed.permalink,
        secret_token: filled(embed.secret_token),
        title: embed.title,
        artist: filled(embed.artist),
        artwork_url: filled(embed.artwork_url),
        duration_ms: nonzero(embed.duration_ms),
        access: filled(embed.access),
        at_ms: nonzero(embed.at_ms),
        tracks: embed.tracks.into_iter().map(track).collect(),
        room: embed.room.map(room),
    })
}

fn track(track: proto::EmbedTrack) -> EmbedTrack {
    EmbedTrack {
        urn: track.urn,
        title: track.title,
        artist: track.artist,
        duration_ms: track.duration_ms,
        artwork_url: filled(track.artwork_url),
    }
}

fn room(room: proto::RoomInvite) -> RoomInvite {
    RoomInvite {
        code: room.code,
        title: filled(room.title),
        host: room.host,
    }
}

fn forwarded(from: proto::Forwarded) -> Forwarded {
    Forwarded {
        user_id: from.user_id,
        name: from.name,
        sent_at_ms: from.sent_at_ms as i64,
    }
}

fn filled(text: String) -> Option<String> {
    (!text.is_empty()).then_some(text)
}

fn nonzero(value: u64) -> Option<u64> {
    (value != 0).then_some(value)
}

fn positive(seq: u64) -> Option<i64> {
    (seq != 0).then_some(seq as i64)
}
