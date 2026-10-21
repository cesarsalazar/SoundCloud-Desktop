mod activity;
mod bringup;
mod clock;
mod counts;
mod drive;
mod expiry;
mod keys;
mod ladder;
mod lifecycle;
mod local;
mod sender;
mod session;
mod signals;
mod status;
mod tap;
mod visibility;
mod wake;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;

use chat_core::api::Api;
use chat_core::api::wire::DeviceView;
use chat_core::{ChatConfig, ChatError, ChatResult, ChatStatus, Clock, ConvId, UserId};
use tauri::Listener;

use super::keychain::Keychain;
use super::state::ChatState;
use crate::rt::AppHandle;
use keys::Epochs;
use session::Session;
use signals::Signals;
use status::StatusCell;

pub use bringup::config_of;
pub use clock::SystemClock;
pub use local::Core;
pub use tap::Tap;

const AUTH_CHANGED: &str = "auth:changed";
const EDGE_CHANGED: &str = "edge:config";

pub struct Runtime {
    app: AppHandle,
    rt: tokio::runtime::Handle,
    dir: PathBuf,
    keychain: Arc<Keychain>,
    api: Arc<Api>,
    clock: Arc<dyn Clock>,
    events: Arc<Tap>,
    signals: Signals,
    status: Arc<StatusCell>,
    lifecycle: tokio::sync::Mutex<()>,
    core: RwLock<Option<Arc<Core>>>,
    config: RwLock<Option<ChatConfig>>,
    epochs: Epochs,
    session: Session,
    woke: AtomicBool,
    read_pending: AtomicBool,
    typing_sent: Mutex<HashMap<ConvId, Instant>>,
}

pub struct Parts {
    pub app: AppHandle,
    pub rt: tokio::runtime::Handle,
    pub dir: PathBuf,
    pub keychain: Arc<Keychain>,
    pub api: Arc<Api>,
    pub clock: Arc<dyn Clock>,
    pub events: Arc<Tap>,
}

pub fn start(state: &ChatState) {
    let runtime = state.runtime.clone();
    visibility::watch(&state.app, runtime.clone(), &state.rt);
    wake::watch(runtime.clone(), &state.rt);
    let watcher = runtime.clone();
    state.app.listen_any(AUTH_CHANGED, move |_| {
        let runtime = watcher.clone();
        watcher
            .rt
            .spawn(async move { runtime.auth_changed().await });
    });
    let rerouted = runtime.clone();
    state
        .app
        .listen_any(EDGE_CHANGED, move |_| rerouted.reroute());
    state.rt.spawn(runtime.run());
}

impl Runtime {
    pub fn new(parts: Parts) -> Self {
        Self {
            status: Arc::new(StatusCell::new(parts.events.clone())),
            app: parts.app,
            rt: parts.rt,
            dir: parts.dir,
            keychain: parts.keychain,
            api: parts.api,
            clock: parts.clock,
            events: parts.events,
            signals: Signals::new(),
            lifecycle: tokio::sync::Mutex::new(()),
            core: RwLock::new(None),
            config: RwLock::new(None),
            epochs: Epochs::default(),
            session: Session::default(),
            woke: AtomicBool::new(false),
            read_pending: AtomicBool::new(false),
            typing_sent: Mutex::new(HashMap::new()),
        }
    }

    pub fn status(&self) -> ChatStatus {
        self.status.get()
    }

    pub fn core(&self) -> Option<Arc<Core>> {
        self.core.read().ok().and_then(|core| core.clone())
    }

    pub fn ready(&self) -> ChatResult<Arc<Core>> {
        self.core().ok_or(ChatError::NotReady)
    }

    pub fn api(&self) -> &Arc<Api> {
        &self.api
    }

    pub fn config(&self) -> Option<ChatConfig> {
        self.config.read().ok().and_then(|config| config.clone())
    }

    pub fn now_ms(&self) -> i64 {
        self.clock.now_ms()
    }

    pub fn typing(&self, conversation_id: ConvId) -> Vec<UserId> {
        self.events.typing(conversation_id)
    }

    pub fn kick(&self) {
        self.signals.kick();
    }

    pub fn outbox_filled(&self) {
        self.signals.outbox();
    }

    pub fn restart(&self) {
        self.signals.restart();
    }

    pub fn adopt_device(&self, device: &DeviceView) {
        self.status.update(|status| {
            status.device_id = Some(device.id);
            status.history = bringup::history_gate(device);
        });
    }

    pub fn set_visible(&self, visible: bool) {
        if self.signals.set_visible(visible) {
            self.status.update(|status| status.visible = visible);
        }
    }

    pub fn woke_up(&self) {
        self.woke.store(true, Ordering::Relaxed);
        self.signals.wake();
    }

    fn take_wake(&self) -> bool {
        self.woke.swap(false, Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests;
