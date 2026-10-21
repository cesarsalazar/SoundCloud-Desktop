use serde::Serialize;

use chat_core::api::wire::FileReport;
use chat_core::{
    ChatError, ChatResult, ChatSettings, DmPolicy, ReportInput, ReportReason, SettingsPatch,
    UserCard, UserId,
};

use super::convert;
use super::reads::blocking;
use super::remote::{self, DELETE, GET, NO_BODY, PUT};
use super::views::{CardView, SettingsView};
use crate::chat::state::ChatState;

const COMMENT_MAX: usize = 500;

#[derive(Serialize)]
struct SettingsBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    dm_policy: Option<DmPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    read_receipts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    typing: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_share: Option<bool>,
}

pub async fn block(state: &ChatState, user_id: UserId) -> ChatResult<()> {
    remote::done(state, PUT, &block_path(&user_id), "", NO_BODY).await
}

pub async fn unblock(state: &ChatState, user_id: UserId) -> ChatResult<()> {
    remote::done(state, DELETE, &block_path(&user_id), "", NO_BODY).await
}

pub async fn report(state: &ChatState, report: ReportInput) -> ChatResult<()> {
    let comment = report
        .comment
        .map(|comment| comment.trim().to_string())
        .filter(|comment| !comment.is_empty());
    if report.seq_from > report.seq_to
        || comment
            .as_ref()
            .is_some_and(|comment| comment.chars().count() > COMMENT_MAX)
    {
        return Err(ChatError::Invalid);
    }
    let body = FileReport {
        conversation_id: report.conversation_id,
        seq_from: report.seq_from,
        seq_to: report.seq_to,
        reason: reason(report.reason).to_string(),
        comment,
    };
    state.api.report(&body).await.map(|_| ())
}

pub async fn settings(state: &ChatState) -> ChatResult<ChatSettings> {
    let view: SettingsView = remote::json(state, GET, "/chat/settings", "", NO_BODY).await?;
    Ok(convert::settings(view))
}

pub async fn put_settings(state: &ChatState, patch: SettingsPatch) -> ChatResult<ChatSettings> {
    let body = SettingsBody {
        dm_policy: patch.dm_policy,
        read_receipts: patch.read_receipts,
        typing: patch.typing,
        presence_share: patch.presence_share,
    };
    let view: SettingsView = remote::json(state, PUT, "/chat/settings", "", Some(&body)).await?;
    Ok(convert::settings(view))
}

pub async fn card(state: &ChatState, user_id: UserId) -> ChatResult<UserCard> {
    let path = format!("/chat/users/{}/card", remote::segment(&user_id));
    let view: CardView = remote::json(state, GET, &path, "", NO_BODY).await?;
    let card = convert::card(view);
    let user = card.user.clone();
    blocking(state, move |core| core.store.put_users(&[user])).await?;
    Ok(card)
}

fn block_path(user_id: &str) -> String {
    format!("/chat/blocks/{}", remote::segment(user_id))
}

fn reason(reason: ReportReason) -> &'static str {
    match reason {
        ReportReason::Spam => "spam",
        ReportReason::Harassment => "harassment",
        ReportReason::Doxxing => "doxxing",
        ReportReason::Illegal => "illegal",
        ReportReason::Other => "other",
    }
}
