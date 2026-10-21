use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chat_core::api::Api;
use chat_core::crypto::DeviceKeys;
use chat_core::outbox::Outbox;
use chat_core::search::Search;
use chat_core::store::Store;
use chat_core::sync::SyncEngine;
use chat_core::{ChatError, ChatResult, DeviceId, Events, KeyName, KeyStore, Zeroizing};

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
        let store = Arc::new(open_store(&database_path(dir), keychain)?);
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

pub fn has_local_state(dir: &Path) -> bool {
    database_path(dir).exists()
}

pub fn remove_database(dir: &Path) -> ChatResult<()> {
    let path = database_path(dir);
    for file in [path.clone(), sibling(&path, "-wal"), sibling(&path, "-shm")] {
        match std::fs::remove_file(&file) {
            Err(error) if error.kind() != ErrorKind::NotFound => return Err(ChatError::Store),
            _ => {}
        }
    }
    Ok(())
}

fn open_store(path: &Path, keychain: &dyn KeyStore) -> ChatResult<Store> {
    let (master, fresh) = database_key(keychain)?;
    let dir = path.parent().unwrap_or(path);
    if fresh {
        remove_database(dir)?;
    }
    match Store::open(path, &master) {
        Err(ChatError::Crypto) => {
            remove_database(dir)?;
            Store::open(path, &master)
        }
        opened => opened,
    }
}

fn database_key(keychain: &dyn KeyStore) -> ChatResult<(Zeroizing<[u8; 32]>, bool)> {
    if let Some(saved) = keychain.load(KeyName::Database)?
        && let Ok(master) = <[u8; 32]>::try_from(saved.as_slice())
    {
        return Ok((Zeroizing::new(master), false));
    }
    let master = chat_core::chat_wire::random_bytes::<32>().map_err(|_| ChatError::Crypto)?;
    let master = Zeroizing::new(master);
    keychain.save(KeyName::Database, master.as_slice())?;
    Ok((master, true))
}

fn database_path(dir: &Path) -> PathBuf {
    dir.join(DATABASE)
}

fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(suffix);
    PathBuf::from(name)
}
