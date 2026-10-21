use serde::Serialize;

use chat_core::envelope::Content;
use chat_core::{
    ChatError, ChatResult, ConvId, ConversationKind, ConversationMeta, ConversationPatch,
    ConversationSummary, CoreEvent, CreateConversation, DeleteScope, GoneEvent, Mute, MyPatch,
    NotifyLevel, UserId, is_disappearing_ttl,
};

use super::compose::queue;
use super::convert;
use super::reads::blocking;
use super::remote::{self, DELETE, GET, NO_BODY, POST, PUT};
use super::views::ConversationView;
use crate::chat::state::ChatState;

const MUTE_FOREVER_MS: i64 = 253_402_300_799_000;

#[derive(Serialize)]
struct CreateBody {
    kind: ConversationKind,
    members: Vec<UserId>,
}

#[derive(Serialize)]
pub struct MyPatchBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    muted_until: Option<Option<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notify: Option<NotifyLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pinned: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    archived: Option<bool>,
}

pub async fn create(
    state: &ChatState,
    input: CreateConversation,
) -> ChatResult<ConversationSummary> {
    let body = CreateBody {
        kind: input.kind,
        members: input.members,
    };
    let view: ConversationView =
        remote::json(state, POST, "/chat/conversations", "", Some(&body)).await?;
    let title = input.title.filter(|title| !title.trim().is_empty());
    if let (ConversationKind::Group, Some(title)) = (input.kind, title) {
        let meta = ConversationMeta {
            title,
            description: String::new(),
            avatar_media_id: None,
        };
        queue(state, view.id, None, Content::Meta(meta)).await?;
    }
    adopt(state, view).await
}

pub async fn update(
    state: &ChatState,
    conversation_id: ConvId,
    patch: ConversationPatch,
) -> ChatResult<ConversationSummary> {
    if let Some(meta) = patch.meta {
        let current = blocking(state, move |core| core.store.conversation(conversation_id)).await?;
        let meta = ConversationMeta {
            title: meta
                .title
                .or_else(|| current.and_then(|summary| summary.title))
                .unwrap_or_default(),
            description: meta.description.unwrap_or_default(),
            avatar_media_id: None,
        };
        queue(state, conversation_id, None, Content::Meta(meta)).await?;
    }
    let view = match patch.me {
        Some(mine) => {
            let path = format!("{}/me", remote::conversation(conversation_id));
            remote::json(state, PUT, &path, "", Some(&my_patch(mine))).await?
        }
        None => fetch(state, conversation_id).await?,
    };
    adopt(state, view).await
}

pub async fn delete(
    state: &ChatState,
    conversation_id: ConvId,
    scope: DeleteScope,
) -> ChatResult<()> {
    let scope = match scope {
        DeleteScope::Me => "for=me",
        DeleteScope::Everyone => "for=everyone",
    };
    remote::done(
        state,
        DELETE,
        &remote::conversation(conversation_id),
        scope,
        NO_BODY,
    )
    .await?;
    forget(state, conversation_id).await
}

pub async fn set_ttl(
    state: &ChatState,
    conversation_id: ConvId,
    ttl_s: Option<u32>,
) -> ChatResult<ConversationSummary> {
    if ttl_s.is_some_and(|ttl| !is_disappearing_ttl(ttl)) {
        return Err(ChatError::Invalid);
    }
    state.api.set_ttl(conversation_id, ttl_s).await?;
    refreshed(state, conversation_id).await
}

pub async fn accept(state: &ChatState, conversation_id: ConvId) -> ChatResult<ConversationSummary> {
    let path = format!("{}/accept", remote::conversation(conversation_id));
    remote::done(state, POST, &path, "", NO_BODY).await?;
    refreshed(state, conversation_id).await
}

pub async fn decline(state: &ChatState, conversation_id: ConvId) -> ChatResult<()> {
    let path = format!("{}/decline", remote::conversation(conversation_id));
    remote::done(state, POST, &path, "", NO_BODY).await?;
    forget(state, conversation_id).await
}

pub async fn refreshed(
    state: &ChatState,
    conversation_id: ConvId,
) -> ChatResult<ConversationSummary> {
    let view = fetch(state, conversation_id).await?;
    adopt(state, view).await
}

async fn fetch(state: &ChatState, conversation_id: ConvId) -> ChatResult<ConversationView> {
    remote::json(
        state,
        GET,
        &remote::conversation(conversation_id),
        "",
        NO_BODY,
    )
    .await
}

async fn adopt(state: &ChatState, view: ConversationView) -> ChatResult<ConversationSummary> {
    state.runtime.kick();
    blocking(state, move |core| {
        let peer = view.peer.as_ref().and_then(|peer| {
            convert::people(&core.store, std::slice::from_ref(peer))
                .into_iter()
                .next()
        });
        let stored = core.store.conversation(view.id)?;
        let summary = convert::merged(convert::summary(&view, peer), stored);
        core.store
            .put_conversations(std::slice::from_ref(&summary))?;
        Ok(summary)
    })
    .await
}

async fn forget(state: &ChatState, conversation_id: ConvId) -> ChatResult<()> {
    blocking(state, move |core| {
        core.store.purge_conversations(&[conversation_id])
    })
    .await?;
    state.events.emit(CoreEvent::Gone(GoneEvent {
        conversations: vec![conversation_id],
        messages: Vec::new(),
    }));
    Ok(())
}

pub fn my_patch(mine: MyPatch) -> MyPatchBody {
    MyPatchBody {
        muted_until: mine.mute.map(|mute| match mute {
            Mute::Off => None,
            Mute::Until { at } => Some(at),
            Mute::Forever => Some(MUTE_FOREVER_MS),
        }),
        notify: mine.notify,
        pinned: mine.pinned,
        archived: mine.archived,
    }
}
