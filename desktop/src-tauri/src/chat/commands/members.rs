use serde::Serialize;

use chat_core::{
    ChatError, ChatRequest, ChatResult, ConvId, ConversationSummary, Invite, InviteOptions, Member,
    MemberRole, UserId,
};

use super::conversations::refreshed;
use super::convert;
use super::reads::blocking;
use super::remote::{self, DELETE, GET, NO_BODY, POST, PUT};
use super::views::{InviteView, JoinedView, MemberView, RequestsView};
use crate::chat::state::ChatState;

#[derive(Serialize)]
struct AddBody {
    users: Vec<UserId>,
}

#[derive(Serialize)]
struct RoleBody {
    role: MemberRole,
}

#[derive(Serialize)]
struct InviteBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_in_s: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    uses: Option<u32>,
}

pub async fn list(state: &ChatState, conversation_id: ConvId) -> ChatResult<Vec<Member>> {
    let path = format!("{}/members", remote::conversation(conversation_id));
    match remote::json::<Vec<MemberView>, ()>(state, GET, &path, "", NO_BODY).await {
        Ok(views) => adopt(state, conversation_id, views).await,
        Err(ChatError::Offline) => {
            blocking(state, move |core| core.store.members(conversation_id)).await
        }
        Err(error) => Err(error),
    }
}

pub async fn add(
    state: &ChatState,
    conversation_id: ConvId,
    users: Vec<UserId>,
) -> ChatResult<Vec<Member>> {
    if users.is_empty() {
        return Err(ChatError::Invalid);
    }
    let path = format!("{}/members", remote::conversation(conversation_id));
    remote::done(state, POST, &path, "", Some(&AddBody { users })).await?;
    state.runtime.kick();
    list(state, conversation_id).await
}

pub async fn remove(state: &ChatState, conversation_id: ConvId, user_id: UserId) -> ChatResult<()> {
    let path = member_path(conversation_id, &user_id);
    remote::done(state, DELETE, &path, "", NO_BODY).await?;
    state.runtime.kick();
    Ok(())
}

pub async fn set_role(
    state: &ChatState,
    conversation_id: ConvId,
    user_id: UserId,
    role: MemberRole,
) -> ChatResult<Member> {
    let path = member_path(conversation_id, &user_id);
    let views: Vec<MemberView> =
        remote::json(state, PUT, &path, "", Some(&RoleBody { role })).await?;
    adopt(state, conversation_id, views)
        .await?
        .into_iter()
        .find(|member| member.user.id == user_id)
        .ok_or(ChatError::NotFound)
}

pub async fn requests(state: &ChatState) -> ChatResult<Vec<ChatRequest>> {
    let answer: RequestsView = remote::json(state, GET, "/chat/requests", "", NO_BODY).await?;
    blocking(state, move |core| {
        let senders: Vec<UserId> = answer
            .requests
            .iter()
            .map(|view| view.from.clone())
            .collect();
        let people = convert::people(&core.store, &senders);
        answer
            .requests
            .iter()
            .zip(people)
            .map(|(view, from)| {
                let stored = core.store.conversation(view.conversation_id)?;
                Ok(convert::request(view, stored, from))
            })
            .collect()
    })
    .await
}

pub async fn invite_create(
    state: &ChatState,
    conversation_id: ConvId,
    options: InviteOptions,
) -> ChatResult<Invite> {
    let path = format!("{}/invites", remote::conversation(conversation_id));
    let body = InviteBody {
        expires_in_s: options.expires_in_s,
        uses: options.uses,
    };
    let view: InviteView = remote::json(state, POST, &path, "", Some(&body)).await?;
    Ok(convert::invite(view))
}

pub async fn invite_revoke(state: &ChatState, token: String) -> ChatResult<()> {
    let path = format!("/chat/invites/{}", remote::segment(&token));
    remote::done(state, DELETE, &path, "", NO_BODY).await
}

pub async fn invite_join(state: &ChatState, token: String) -> ChatResult<ConversationSummary> {
    let path = format!("/chat/invites/{}/join", remote::segment(&token));
    let joined: JoinedView = remote::json(state, POST, &path, "", NO_BODY).await?;
    refreshed(state, joined.conversation_id).await
}

async fn adopt(
    state: &ChatState,
    conversation_id: ConvId,
    views: Vec<MemberView>,
) -> ChatResult<Vec<Member>> {
    blocking(state, move |core| {
        let ids: Vec<UserId> = views.iter().map(|view| view.user_id.clone()).collect();
        let members: Vec<Member> = views
            .iter()
            .zip(convert::people(&core.store, &ids))
            .map(|(view, user)| convert::member(view, user))
            .collect();
        core.store.put_members(conversation_id, &members)?;
        Ok(members)
    })
    .await
}

fn member_path(conversation_id: ConvId, user_id: &str) -> String {
    format!(
        "{}/members/{}",
        remote::conversation(conversation_id),
        remote::segment(user_id)
    )
}
