use std::path::PathBuf;
use std::sync::Arc;

use chat_core::{ChatStatus, Events, KeyStore, Transport};

use super::events::TauriEvents;
use super::keychain::Keychain;
use super::notify::Notifier;
use super::transport::EdgeTransport;
use crate::rt::AppHandle;

pub struct ChatState {
    pub app: AppHandle,
    pub rt: tokio::runtime::Handle,
    pub dir: PathBuf,
    pub keys: Arc<dyn KeyStore>,
    pub transport: Arc<dyn Transport>,
    pub events: Arc<dyn Events>,
    pub notifier: Notifier,
}

impl ChatState {
    pub fn new(app: AppHandle, rt: tokio::runtime::Handle, dir: PathBuf) -> Self {
        Self {
            events: Arc::new(TauriEvents::new(app.clone())),
            app,
            rt,
            dir,
            keys: Arc::new(Keychain),
            transport: Arc::new(EdgeTransport),
            notifier: Notifier::default(),
        }
    }

    pub fn status(&self) -> ChatStatus {
        ChatStatus::off()
    }
}
