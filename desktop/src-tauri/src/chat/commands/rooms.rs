use serde::Serialize;

use chat_core::{
    ChatResult, ConversationSummary, Embed, RoomModeration, RoomPreview, RoomSettingsPatch,
};

use super::conversations::refreshed;
use super::convert;
use super::remote::{self, DELETE, GET, NO_BODY, POST, PUT};
use super::views::{PreviewView, RoomChatView};
use crate::chat::state::ChatState;

#[derive(Serialize)]
struct MuteBody {
    seconds: u32,
}

#[derive(Serialize)]
struct ModeratorBody {
    on: bool,
}

#[derive(Serialize)]
struct SettingsBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    slow_mode_s: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    chat_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    links: Option<bool>,
}

#[derive(Serialize)]
struct SuggestBody {
    track: Embed,
}

pub async fn chat(state: &ChatState, code: String) -> ChatResult<ConversationSummary> {
    let opened: RoomChatView = remote::json(state, GET, &room(&code, "chat"), "", NO_BODY).await?;
    refreshed(state, opened.conversation_id).await
}

pub async fn preview(state: &ChatState, code: String) -> ChatResult<RoomPreview> {
    let view: PreviewView = remote::json(state, GET, &room(&code, "preview"), "", NO_BODY).await?;
    Ok(convert::preview(view))
}

pub async fn moderate(state: &ChatState, code: String, action: RoomModeration) -> ChatResult<()> {
    match action {
        RoomModeration::Kick { user_id } => {
            let path = room(&code, &format!("members/{}", remote::segment(&user_id)));
            remote::done(state, DELETE, &path, "", NO_BODY).await
        }
        RoomModeration::Ban { user_id } => {
            let path = room(&code, &format!("members/{}", remote::segment(&user_id)));
            remote::done(state, DELETE, &path, "ban=true", NO_BODY).await
        }
        RoomModeration::Mute { user_id, seconds } => {
            let path = room(
                &code,
                &format!("members/{}/mute", remote::segment(&user_id)),
            );
            remote::done(state, PUT, &path, "", Some(&MuteBody { seconds })).await
        }
        RoomModeration::Moderator { user_id, on } => {
            let path = room(&code, &format!("moderators/{}", remote::segment(&user_id)));
            remote::done(state, PUT, &path, "", Some(&ModeratorBody { on })).await
        }
    }
}

pub async fn settings(state: &ChatState, code: String, patch: RoomSettingsPatch) -> ChatResult<()> {
    let body = SettingsBody {
        title: patch.title,
        slow_mode_s: patch.slow_mode_s,
        chat_enabled: patch.chat_enabled,
        links: patch.links,
    };
    remote::done(state, PUT, &room(&code, "settings"), "", Some(&body)).await
}

pub async fn suggest(state: &ChatState, code: String, track: Embed) -> ChatResult<()> {
    let body = SuggestBody { track };
    remote::done(state, POST, &room(&code, "suggest"), "", Some(&body)).await
}

fn room(code: &str, rest: &str) -> String {
    format!("/chat/rooms/{}/{rest}", remote::segment(code))
}
