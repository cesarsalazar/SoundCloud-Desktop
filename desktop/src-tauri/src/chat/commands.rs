#![allow(unused_variables)]

use chat_core::{
    ChatConfig, ChatError, ChatRequest, ChatSettings, ChatStatus, ConvId, ConversationPage,
    ConversationPatch, ConversationSummary, CreateConversation, DeleteScope, Device, DeviceId,
    Embed, EmbedPage, EmbedsQuery, HistoryPage, HistoryQuery, Invite, InviteOptions, LinkClaimed,
    LinkCreated, LinkPoll, ListQuery, LoginBegin, LoginPoll, LoginTarget, Member, MemberRole,
    Message, MessageTarget, NotifyPrefs, ReportInput, RichText, RoomModeration, RoomPreview,
    RoomSettingsPatch, SearchHit, SearchQuery, SendInput, Seq, SettingsPatch, ThreadSnapshot,
    UserCard, UserId,
};
use tauri::State;

use super::login;
use super::state::ChatState;

#[tauri::command]
pub async fn chat_status(state: State<'_, ChatState>) -> Result<ChatStatus, ChatError> {
    Ok(state.status())
}

#[tauri::command]
pub async fn chat_config(state: State<'_, ChatState>) -> Result<ChatConfig, ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_login_begin(
    state: State<'_, ChatState>,
    target: LoginTarget,
) -> Result<LoginBegin, ChatError> {
    login::begin(&state, target).await
}

#[tauri::command]
pub async fn chat_login_poll(
    state: State<'_, ChatState>,
    login_request_id: String,
) -> Result<LoginPoll, ChatError> {
    login::poll(&state, &login_request_id).await
}

#[tauri::command]
pub async fn chat_link_create_pull(state: State<'_, ChatState>) -> Result<LinkCreated, ChatError> {
    login::link_create_pull(&state).await
}

#[tauri::command]
pub async fn chat_link_claim_push(
    state: State<'_, ChatState>,
    claim_token: String,
) -> Result<LinkClaimed, ChatError> {
    login::link_claim_push(&state, &claim_token).await
}

#[tauri::command]
pub async fn chat_link_poll(
    state: State<'_, ChatState>,
    link_request_id: String,
) -> Result<LinkPoll, ChatError> {
    login::link_poll(&state, &link_request_id).await
}

#[tauri::command]
pub async fn chat_list(
    state: State<'_, ChatState>,
    query: ListQuery,
) -> Result<ConversationPage, ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_open(
    state: State<'_, ChatState>,
    conversation_id: ConvId,
) -> Result<ThreadSnapshot, ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_history(
    state: State<'_, ChatState>,
    conversation_id: ConvId,
    query: HistoryQuery,
) -> Result<HistoryPage, ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_create(
    state: State<'_, ChatState>,
    input: CreateConversation,
) -> Result<ConversationSummary, ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_conversation_update(
    state: State<'_, ChatState>,
    conversation_id: ConvId,
    patch: ConversationPatch,
) -> Result<ConversationSummary, ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_conversation_delete(
    state: State<'_, ChatState>,
    conversation_id: ConvId,
    scope: DeleteScope,
) -> Result<(), ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_set_ttl(
    state: State<'_, ChatState>,
    conversation_id: ConvId,
    ttl_s: Option<u32>,
) -> Result<ConversationSummary, ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_send(
    state: State<'_, ChatState>,
    input: SendInput,
) -> Result<Message, ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_edit(
    state: State<'_, ChatState>,
    conversation_id: ConvId,
    seq: Seq,
    text: RichText,
) -> Result<Message, ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_delete(
    state: State<'_, ChatState>,
    conversation_id: ConvId,
    target: MessageTarget,
    scope: DeleteScope,
) -> Result<(), ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_react(
    state: State<'_, ChatState>,
    conversation_id: ConvId,
    seq: Seq,
    emoji: String,
    on: bool,
) -> Result<(), ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_pin(
    state: State<'_, ChatState>,
    conversation_id: ConvId,
    seq: Seq,
    on: bool,
) -> Result<(), ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_mark_read(
    state: State<'_, ChatState>,
    conversation_id: ConvId,
    seq: Seq,
) -> Result<(), ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_typing(
    state: State<'_, ChatState>,
    conversation_id: ConvId,
) -> Result<(), ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_focus(
    state: State<'_, ChatState>,
    conversation_id: Option<ConvId>,
) -> Result<(), ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_draft_save(
    state: State<'_, ChatState>,
    conversation_id: ConvId,
    draft: Option<RichText>,
) -> Result<(), ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_search(
    state: State<'_, ChatState>,
    query: SearchQuery,
) -> Result<Vec<SearchHit>, ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_embeds(
    state: State<'_, ChatState>,
    conversation_id: ConvId,
    query: EmbedsQuery,
) -> Result<EmbedPage, ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_members(
    state: State<'_, ChatState>,
    conversation_id: ConvId,
) -> Result<Vec<Member>, ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_member_add(
    state: State<'_, ChatState>,
    conversation_id: ConvId,
    user_ids: Vec<UserId>,
) -> Result<Vec<Member>, ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_member_remove(
    state: State<'_, ChatState>,
    conversation_id: ConvId,
    user_id: UserId,
) -> Result<(), ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_member_role(
    state: State<'_, ChatState>,
    conversation_id: ConvId,
    user_id: UserId,
    role: MemberRole,
) -> Result<Member, ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_accept(
    state: State<'_, ChatState>,
    conversation_id: ConvId,
) -> Result<ConversationSummary, ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_decline(
    state: State<'_, ChatState>,
    conversation_id: ConvId,
) -> Result<(), ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_requests(state: State<'_, ChatState>) -> Result<Vec<ChatRequest>, ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_invite_create(
    state: State<'_, ChatState>,
    conversation_id: ConvId,
    options: InviteOptions,
) -> Result<Invite, ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_invite_revoke(
    state: State<'_, ChatState>,
    token: String,
) -> Result<(), ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_invite_join(
    state: State<'_, ChatState>,
    token: String,
) -> Result<ConversationSummary, ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_block(state: State<'_, ChatState>, user_id: UserId) -> Result<(), ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_unblock(state: State<'_, ChatState>, user_id: UserId) -> Result<(), ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_report(
    state: State<'_, ChatState>,
    report: ReportInput,
) -> Result<(), ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_settings_get(state: State<'_, ChatState>) -> Result<ChatSettings, ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_settings_put(
    state: State<'_, ChatState>,
    patch: SettingsPatch,
) -> Result<ChatSettings, ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_user_card(
    state: State<'_, ChatState>,
    user_id: UserId,
) -> Result<UserCard, ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_devices(state: State<'_, ChatState>) -> Result<Vec<Device>, ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_device_confirm(
    state: State<'_, ChatState>,
    device_id: DeviceId,
) -> Result<(), ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_device_not_me(
    state: State<'_, ChatState>,
    device_id: DeviceId,
) -> Result<(), ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_device_remove(
    state: State<'_, ChatState>,
    device_id: DeviceId,
) -> Result<(), ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_room_chat(
    state: State<'_, ChatState>,
    code: String,
) -> Result<ConversationSummary, ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_room_preview(
    state: State<'_, ChatState>,
    code: String,
) -> Result<RoomPreview, ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_room_moderate(
    state: State<'_, ChatState>,
    code: String,
    action: RoomModeration,
) -> Result<(), ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_room_settings(
    state: State<'_, ChatState>,
    code: String,
    patch: RoomSettingsPatch,
) -> Result<(), ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_room_suggest(
    state: State<'_, ChatState>,
    code: String,
    track: Embed,
) -> Result<(), ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_logout(state: State<'_, ChatState>) -> Result<(), ChatError> {
    Err(ChatError::Unimplemented)
}

#[tauri::command]
pub async fn chat_notify_prefs(
    state: State<'_, ChatState>,
    prefs: Option<NotifyPrefs>,
) -> Result<NotifyPrefs, ChatError> {
    Ok(state.notifier.prefs(prefs))
}
