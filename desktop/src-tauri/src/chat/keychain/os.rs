use chat_core::{ChatError, ChatResult, KeyName, Zeroizing};
use keyring::{Entry, Error};

use crate::app::diagnostics;

pub struct OsVault {
    service: String,
}

impl OsVault {
    pub fn new(service: &str) -> Self {
        Self {
            service: service.to_string(),
        }
    }

    pub fn load(&self, name: KeyName) -> ChatResult<Option<Zeroizing<Vec<u8>>>> {
        match self.entry(name)?.get_secret() {
            Ok(secret) => Ok(Some(Zeroizing::new(secret))),
            Err(Error::NoEntry) => Ok(None),
            Err(error) => Err(failed(name, error)),
        }
    }

    pub fn save(&self, name: KeyName, secret: &[u8]) -> ChatResult<()> {
        self.entry(name)?
            .set_secret(secret)
            .map_err(|error| failed(name, error))
    }

    pub fn delete(&self, name: KeyName) -> ChatResult<()> {
        match self.entry(name)?.delete_credential() {
            Ok(()) | Err(Error::NoEntry) => Ok(()),
            Err(error) => Err(failed(name, error)),
        }
    }

    fn entry(&self, name: KeyName) -> ChatResult<Entry> {
        Entry::new(&self.service, name.as_str()).map_err(|error| failed(name, error))
    }
}

fn failed(name: KeyName, error: Error) -> ChatError {
    diagnostics::warn(format!("[chat] keychain {}: {error}", name.as_str()));
    ChatError::Keychain
}
