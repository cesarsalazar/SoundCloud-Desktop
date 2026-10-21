use std::collections::HashMap;
use std::sync::Mutex;

use chat_core::api::Api;
use chat_core::crypto::Ck;
use chat_core::{ChatError, ChatResult, ConvId};

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

pub async fn current(
    api: &Api,
    core: &Core,
    epochs: &Epochs,
    conversation_id: ConvId,
) -> ChatResult<u32> {
    if let Some(epoch) = epochs.get(conversation_id) {
        return Ok(epoch);
    }
    let newest = fetch(api, core, conversation_id, None).await?;
    epochs.raise(conversation_id, newest);
    Ok(newest)
}

pub async fn ck(api: &Api, core: &Core, conversation_id: ConvId, epoch: u32) -> ChatResult<Ck> {
    if let Some(ck) = core.store.ck(conversation_id, epoch)? {
        return Ok(ck);
    }
    fetch(api, core, conversation_id, Some(epoch)).await?;
    core.store
        .ck(conversation_id, epoch)?
        .ok_or(ChatError::Crypto)
}

async fn fetch(
    api: &Api,
    core: &Core,
    conversation_id: ConvId,
    from_epoch: Option<u32>,
) -> ChatResult<u32> {
    let mut newest = None;
    for key in api.keys(conversation_id, from_epoch).await? {
        let epoch = u32::try_from(key.epoch).map_err(|_| ChatError::Protocol)?;
        let ck = core
            .keys
            .open_ck(conversation_id, epoch, core.device_id, &key.sealed)?;
        core.store.put_ck(conversation_id, epoch, &ck)?;
        newest = newest.max(Some(epoch));
    }
    newest.ok_or(ChatError::NotReady)
}
