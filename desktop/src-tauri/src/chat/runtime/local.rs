use std::path::{Path, PathBuf};
use std::sync::Arc;

use chat_core::api::Api;
use chat_core::crypto::DeviceKeys;
use chat_core::outbox::Outbox;
use chat_core::search::Search;
use chat_core::store::Store;
use chat_core::sync::SyncEngine;
use chat_core::{ChatError, ChatResult, DeviceId, Events, KeyStore};

use super::Runtime;

const DATABASE: &str = "chat.db";

pub struct Core {
    pub device_id: DeviceId,
    pub keys: Arc<DeviceKeys>,
    pub store: Arc<Store>,
    pub outbox: Outbox,
    pub search: Search,
    pub sync: SyncEngine,
}

impl Core {
    pub fn open(
        dir: &Path,
        keychain: &dyn KeyStore,
        api: Arc<Api>,
        events: Arc<dyn Events>,
    ) -> ChatResult<Self> {
        std::fs::create_dir_all(dir).map_err(|_| ChatError::Store)?;
        let (device_id, keys) = DeviceKeys::load_or_create(keychain)?;
        let store = Arc::new(Store::open_keyed(&database_path(dir), keychain)?);
        Ok(Self {
            device_id,
            keys: Arc::new(keys),
            outbox: Outbox::new(store.clone()),
            search: Search::new(store.clone()),
            sync: SyncEngine::new(api, store.clone(), events),
            store,
        })
    }
}

impl Runtime {
    pub(super) async fn on_store<T, F>(&self, core: &Arc<Core>, work: F) -> ChatResult<T>
    where
        T: Send + 'static,
        F: FnOnce(&Core) -> ChatResult<T> + Send + 'static,
    {
        let core = core.clone();
        self.rt
            .spawn_blocking(move || work(&core))
            .await
            .map_err(|_| ChatError::Internal)?
    }
}

pub fn has_local_state(dir: &Path) -> bool {
    database_path(dir).exists()
}

pub fn database_path(dir: &Path) -> PathBuf {
    dir.join(DATABASE)
}
