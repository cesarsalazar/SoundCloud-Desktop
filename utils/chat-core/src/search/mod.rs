use std::sync::Arc;

use crate::contracts::{ChatError, ChatResult, ConvId, SearchHit, SearchQuery, Seq};
use crate::store::Store;

pub const PREFIX_MIN: usize = 2;
pub const PREFIX_MAX: usize = 16;
pub const TOKEN_BYTES: usize = 12;

pub struct Search {
    store: Arc<Store>,
}

impl Search {
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }

    pub fn store(&self) -> &Arc<Store> {
        &self.store
    }

    pub fn index(&self, _conversation_id: ConvId, _seq: Seq, _text: &str) -> ChatResult<()> {
        Err(ChatError::Unimplemented)
    }

    pub fn remove(&self, _conversation_id: ConvId, _seq: Seq) -> ChatResult<()> {
        Err(ChatError::Unimplemented)
    }

    pub fn query(&self, _query: &SearchQuery) -> ChatResult<Vec<SearchHit>> {
        Err(ChatError::Unimplemented)
    }
}
