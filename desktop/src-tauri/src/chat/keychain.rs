mod file;
mod os;

use std::path::Path;
use std::sync::Mutex;

use chat_core::{ChatResult, KeyName, KeyStorage, KeyStore, Zeroizing};

use crate::app::diagnostics;

use file::FileVault;
use os::OsVault;

const FALL_BACK_AFTER: u32 = 5;

pub struct Keychain {
    os: OsVault,
    file: FileVault,
    storage: Mutex<KeyStorage>,
}

impl Keychain {
    pub fn new(service: &str, dir: &Path) -> Self {
        let file = FileVault::new(dir);
        let storage = if file.exists() {
            KeyStorage::File
        } else {
            KeyStorage::Os
        };
        Self {
            os: OsVault::new(service),
            file,
            storage: Mutex::new(storage),
        }
    }

    pub fn fall_back(&self) -> ChatResult<()> {
        self.file.create()?;
        self.set_storage(KeyStorage::File);
        diagnostics::warn("[chat] the os keychain is unavailable, chat keys moved to a 0600 file");
        Ok(())
    }

    pub fn wipe(&self) -> ChatResult<()> {
        if self.storage() == KeyStorage::Os {
            for name in KeyName::ALL {
                self.os.delete(name)?;
            }
        }
        self.file.remove()?;
        self.set_storage(KeyStorage::Os);
        Ok(())
    }

    fn set_storage(&self, storage: KeyStorage) {
        if let Ok(mut current) = self.storage.lock() {
            *current = storage;
        }
    }
}

impl KeyStore for Keychain {
    fn storage(&self) -> KeyStorage {
        self.storage
            .lock()
            .map(|storage| *storage)
            .unwrap_or(KeyStorage::Os)
    }

    fn load(&self, name: KeyName) -> ChatResult<Option<Zeroizing<Vec<u8>>>> {
        match self.storage() {
            KeyStorage::Os => self.os.load(name),
            KeyStorage::File => self.file.load(name),
        }
    }

    fn save(&self, name: KeyName, secret: &[u8]) -> ChatResult<()> {
        match self.storage() {
            KeyStorage::Os => self.os.save(name, secret),
            KeyStorage::File => self.file.save(name, secret),
        }
    }

    fn delete(&self, name: KeyName) -> ChatResult<()> {
        match self.storage() {
            KeyStorage::Os => self.os.delete(name),
            KeyStorage::File => self.file.delete(name),
        }
    }
}

pub fn should_fall_back(storage: KeyStorage, has_local_state: bool, failures: u32) -> bool {
    storage == KeyStorage::Os && (!has_local_state || failures >= FALL_BACK_AFTER)
}

#[cfg(test)]
mod tests;
