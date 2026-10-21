use std::path::PathBuf;
use std::sync::Arc;

use chat_core::api::Api;
use chat_core::channel::Channel;
use chat_core::{ChatResult, ChatStatus, Clock, Events, KeyStore, Transport};

use super::events::TauriEvents;
use super::keychain::Keychain;
use super::notify::Notifier;
use super::runtime::{Core, Parts, Runtime, StatusCell, SystemClock, Tap};
use super::transport::{EdgeTransport, Route};
use crate::rt::AppHandle;

const KEYCHAIN_SUFFIX: &str = ".chat";

pub struct ChatState {
    pub app: AppHandle,
    pub rt: tokio::runtime::Handle,
    pub dir: PathBuf,
    pub keys: Arc<dyn KeyStore>,
    pub keychain: Arc<Keychain>,
    pub transport: Arc<dyn Transport>,
    pub events: Arc<dyn Events>,
    pub clock: Arc<dyn Clock>,
    pub channel: Arc<Channel>,
    pub api: Arc<Api>,
    pub runtime: Arc<Runtime>,
    pub notifier: Notifier,
}

impl ChatState {
    pub fn new(app: AppHandle, rt: tokio::runtime::Handle, dir: PathBuf) -> Self {
        let service = format!("{}{KEYCHAIN_SUFFIX}", app.config().identifier);
        let keychain = Arc::new(Keychain::new(&service, &dir));
        let raw: Arc<dyn Events> = Arc::new(TauriEvents::new(app.clone()));
        let status = Arc::new(StatusCell::new(raw.clone()));
        let tap = Arc::new(Tap::new(raw, status.clone()));
        let clock: Arc<dyn Clock> = Arc::new(SystemClock);
        let transport: Arc<dyn Transport> = Arc::new(EdgeTransport::new(Route::from_env()));
        let channel = Arc::new(Channel::new(clock.clone()));
        let api = Arc::new(Api::new(channel.clone(), transport.clone()));
        let runtime = Arc::new(Runtime::new(Parts {
            app: app.clone(),
            rt: rt.clone(),
            dir: dir.clone(),
            keychain: keychain.clone(),
            api: api.clone(),
            clock: clock.clone(),
            events: tap.clone(),
            status,
        }));
        Self {
            app,
            rt,
            dir,
            keys: keychain.clone(),
            keychain,
            transport,
            events: tap,
            clock,
            channel,
            api,
            runtime,
            notifier: Notifier::default(),
        }
    }

    pub fn status(&self) -> ChatStatus {
        self.runtime.status()
    }

    pub fn core(&self) -> ChatResult<Arc<Core>> {
        self.runtime.ready()
    }

    pub async fn local_core(&self) -> ChatResult<Arc<Core>> {
        self.runtime.local_core().await
    }

    pub async fn wipe(&self, forget_device: bool) -> ChatResult<()> {
        self.runtime.wipe(forget_device).await
    }

    pub async fn log_out(&self) -> ChatResult<()> {
        self.runtime.log_out().await
    }

    pub fn relogged(&self) {
        self.runtime.relogged();
    }
}
