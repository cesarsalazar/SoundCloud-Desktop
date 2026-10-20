use std::sync::Arc;

use crate::contracts::{
    ChatError, ChatResult, ClientId, ConvId, Message, MessageClass, Millis, Seq,
};
use crate::store::Store;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxItem {
    pub client_id: ClientId,
    pub conversation_id: ConvId,
    pub class: MessageClass,
    pub target_seq: Option<Seq>,
    pub envelope: Vec<u8>,
    pub created_at: Millis,
    pub attempts: u32,
    pub next_at: Millis,
}

pub struct Outbox {
    store: Arc<Store>,
}

impl Outbox {
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }

    pub fn store(&self) -> &Arc<Store> {
        &self.store
    }

    pub fn push(&self, _item: OutboxItem) -> ChatResult<Message> {
        Err(ChatError::Unimplemented)
    }

    pub fn due(&self, _now: Millis, _limit: usize) -> ChatResult<Vec<OutboxItem>> {
        Err(ChatError::Unimplemented)
    }

    pub fn sent(&self, _client_id: ClientId, _seq: Seq, _ts: Millis) -> ChatResult<()> {
        Err(ChatError::Unimplemented)
    }

    pub fn failed(&self, _client_id: ClientId, _retry_at: Option<Millis>) -> ChatResult<()> {
        Err(ChatError::Unimplemented)
    }

    pub fn retry(&self, _client_id: ClientId, _now: Millis) -> ChatResult<()> {
        Err(ChatError::Unimplemented)
    }

    pub fn discard(&self, _client_id: ClientId) -> ChatResult<()> {
        Err(ChatError::Unimplemented)
    }

    pub fn pending(&self, _conversation_id: ConvId) -> ChatResult<Vec<Message>> {
        Err(ChatError::Unimplemented)
    }
}
