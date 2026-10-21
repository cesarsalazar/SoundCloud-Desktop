use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chat_core::api::wire::SealedKey;
use chat_core::crypto::Ck;
use chat_core::{ChatError, ChatResult, ConvId};

use super::Runtime;
use super::local::Core;

#[derive(Default)]
pub struct Epochs(Mutex<HashMap<ConvId, u32>>);

impl Epochs {
    pub fn forget(&self, conversation_id: ConvId) {
        if let Ok(mut known) = self.0.lock() {
            known.remove(&conversation_id);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut known) = self.0.lock() {
            known.clear();
        }
    }

    fn get(&self, conversation_id: ConvId) -> Option<u32> {
        self.0.lock().ok()?.get(&conversation_id).copied()
    }

    fn raise(&self, conversation_id: ConvId, epoch: u32) {
        if let Ok(mut known) = self.0.lock() {
            let current = known.entry(conversation_id).or_insert(epoch);
            *current = (*current).max(epoch);
        }
    }
}

impl Runtime {
    pub(super) async fn current_epoch(
        &self,
        core: &Arc<Core>,
        conversation_id: ConvId,
    ) -> ChatResult<u32> {
        if let Some(epoch) = self.epochs.get(conversation_id) {
            return Ok(epoch);
        }
        let newest = self.fetch_keys(core, conversation_id, None).await?;
        self.epochs.raise(conversation_id, newest);
        Ok(newest)
    }

    pub(super) async fn ck_of(
        &self,
        core: &Arc<Core>,
        conversation_id: ConvId,
        epoch: u32,
    ) -> ChatResult<Ck> {
        let cached = move |core: &Core| core.store.ck(conversation_id, epoch);
        if let Some(ck) = self.on_store(core, cached).await? {
            return Ok(ck);
        }
        self.fetch_keys(core, conversation_id, Some(epoch)).await?;
        self.on_store(core, cached).await?.ok_or(ChatError::Crypto)
    }

    async fn fetch_keys(
        &self,
        core: &Arc<Core>,
        conversation_id: ConvId,
        from_epoch: Option<u32>,
    ) -> ChatResult<u32> {
        let sealed = self.api.keys(conversation_id, from_epoch).await?;
        self.on_store(core, move |core| keep(core, conversation_id, &sealed))
            .await
    }
}

fn keep(core: &Core, conversation_id: ConvId, sealed: &[SealedKey]) -> ChatResult<u32> {
    let mut newest = None;
    for key in sealed {
        let epoch = u32::try_from(key.epoch).map_err(|_| ChatError::Protocol)?;
        let ck = core
            .keys
            .open_ck(conversation_id, epoch, core.device_id, &key.sealed)?;
        core.store.put_ck(conversation_id, epoch, &ck)?;
        newest = newest.max(Some(epoch));
    }
    newest.ok_or(ChatError::NotReady)
}
