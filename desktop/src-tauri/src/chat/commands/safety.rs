use chat_core::api::wire::{FileReport, SettingsChange};
use chat_core::{
    ChatError, ChatResult, ChatSettings, ReportInput, SettingsPatch, UserCard, UserId,
};

use super::reads::blocking;
use crate::chat::state::ChatState;

const COMMENT_MAX: usize = 500;

pub async fn block(state: &ChatState, user_id: UserId) -> ChatResult<()> {
    state.api.block(&user_id).await
}

pub async fn unblock(state: &ChatState, user_id: UserId) -> ChatResult<()> {
    state.api.unblock(&user_id).await
}

pub async fn report(state: &ChatState, report: ReportInput) -> ChatResult<()> {
    let report = checked(report)?;
    state
        .api
        .report(&FileReport::from(&report))
        .await
        .map(|_| ())
}

pub async fn settings(state: &ChatState) -> ChatResult<ChatSettings> {
    Ok(ChatSettings::from(&state.api.settings().await?))
}

pub async fn put_settings(state: &ChatState, patch: SettingsPatch) -> ChatResult<ChatSettings> {
    let view = state
        .api
        .put_settings(&SettingsChange::from(&patch))
        .await?;
    Ok(ChatSettings::from(&view))
}

pub async fn card(state: &ChatState, user_id: UserId) -> ChatResult<UserCard> {
    let card = UserCard::from(&state.api.user_card(&user_id).await?);
    let user = card.user.clone();
    blocking(state, move |core| core.store.put_users(&[user])).await?;
    Ok(card)
}

pub fn checked(report: ReportInput) -> ChatResult<ReportInput> {
    let comment = report
        .comment
        .map(|comment| comment.trim().to_string())
        .filter(|comment| !comment.is_empty());
    let too_long = comment
        .as_ref()
        .is_some_and(|comment| comment.chars().count() > COMMENT_MAX);
    if report.seq_from > report.seq_to || too_long {
        return Err(ChatError::Invalid);
    }
    Ok(ReportInput { comment, ..report })
}
