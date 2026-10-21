use chat_core::api::wire::DeviceView;
use chat_core::{
    ChatConfig, ChatResult, DEVICE_AGE_RULE_MS, Device, DeviceActions, DeviceId, DeviceStatus,
    Millis, Tier,
};

use super::remote::{self, DELETE, NO_BODY, POST};
use crate::chat::state::ChatState;

pub async fn config(state: &ChatState) -> ChatResult<ChatConfig> {
    match state.api.config().await {
        Ok(view) => Ok(crate::chat::runtime::config_of(view)),
        Err(error) => state.runtime.config().ok_or(error),
    }
}

pub async fn list(state: &ChatState) -> ChatResult<Vec<Device>> {
    let views = state.api.devices().await?;
    let now = state.runtime.now_ms();
    let me = views.iter().find(|view| view.current).cloned();
    if let Some(me) = &me {
        state.runtime.adopt_device(me);
    }
    Ok(views
        .into_iter()
        .map(|view| device(&view, me.as_ref(), now))
        .collect())
}

pub async fn confirm(state: &ChatState, device_id: DeviceId) -> ChatResult<()> {
    let path = format!("/chat/devices/{device_id}/confirm");
    remote::done(state, POST, &path, "", NO_BODY).await
}

pub async fn not_me(state: &ChatState, device_id: DeviceId) -> ChatResult<()> {
    let path = format!("/chat/devices/{device_id}/not-me");
    remote::done(state, POST, &path, "", NO_BODY).await
}

pub async fn remove(state: &ChatState, device_id: DeviceId) -> ChatResult<()> {
    if state.status().device_id == Some(device_id) {
        return logout(state).await;
    }
    let path = format!("/chat/devices/{device_id}");
    remote::done(state, DELETE, &path, "", NO_BODY).await
}

pub async fn logout(state: &ChatState) -> ChatResult<()> {
    let _ = remote::done(state, DELETE, "/chat/devices/me", "", NO_BODY).await;
    state.log_out().await
}

pub fn device(view: &DeviceView, me: Option<&DeviceView>, now: Millis) -> Device {
    Device {
        id: view.id,
        platform: view.platform.clone(),
        name: view.name.clone(),
        app_version: view.app_version.clone(),
        tier: tier(&view.tier),
        status: view.status,
        created_at: view.created_at,
        last_seen_at: view.last_seen_at,
        history_unlocked: view.history_unlocked,
        current: view.current,
        actions: actions(view, me, now),
    }
}

pub fn actions(view: &DeviceView, me: Option<&DeviceView>, now: Millis) -> DeviceActions {
    if view.current {
        return DeviceActions {
            confirm: false,
            not_me: false,
            remove: true,
        };
    }
    let Some(me) = me else {
        return DeviceActions::default();
    };
    if view.status == DeviceStatus::Revoked {
        return DeviceActions::default();
    }
    let trusted = me.created_at <= view.created_at || now - me.created_at >= DEVICE_AGE_RULE_MS;
    DeviceActions {
        confirm: trusted && me.history_unlocked && !view.history_unlocked,
        not_me: trusted,
        remove: trusted,
    }
}

fn tier(raw: &str) -> Tier {
    match raw {
        "relay" => Tier::Relay,
        "pro" => Tier::Pro,
        _ => Tier::Direct,
    }
}
