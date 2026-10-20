use chat_core::{ChatError, ChatResult, KeyName, KeyStorage, KeyStore, Zeroizing};

pub struct Keychain;

impl KeyStore for Keychain {
    fn storage(&self) -> KeyStorage {
        KeyStorage::Os
    }

    fn load(&self, _name: KeyName) -> ChatResult<Option<Zeroizing<Vec<u8>>>> {
        Err(ChatError::Unimplemented)
    }

    fn save(&self, _name: KeyName, _secret: &[u8]) -> ChatResult<()> {
        Err(ChatError::Unimplemented)
    }

    fn delete(&self, _name: KeyName) -> ChatResult<()> {
        Err(ChatError::Unimplemented)
    }
}
