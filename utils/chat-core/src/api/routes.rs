use super::wire::{
    ConfigView, DeviceView, FileReport, MessagesPage, RegisterDevice, SealedKey, SendAnswer,
    SendMessage, SyncAnswer, SyncRequest,
};
use super::{Api, CallOptions};
use crate::contracts::{ChatError, ChatResult, ConvId, HistoryQuery, Seq, Tier};

impl Api {
    pub async fn config(&self) -> ChatResult<ConfigView> {
        Err(ChatError::Unimplemented)
    }

    pub async fn register_device(&self, _body: &RegisterDevice) -> ChatResult<DeviceView> {
        Err(ChatError::Unimplemented)
    }

    pub async fn devices(&self) -> ChatResult<Vec<DeviceView>> {
        Err(ChatError::Unimplemented)
    }

    pub async fn keys(
        &self,
        _conversation_id: ConvId,
        _from_epoch: Option<u32>,
    ) -> ChatResult<Vec<SealedKey>> {
        Err(ChatError::Unimplemented)
    }

    pub async fn sync(&self, _request: &SyncRequest, _tier: Tier) -> ChatResult<SyncAnswer> {
        Err(ChatError::Unimplemented)
    }

    pub async fn send(
        &self,
        _conversation_id: ConvId,
        _body: &SendMessage,
    ) -> ChatResult<SendAnswer> {
        Err(ChatError::Unimplemented)
    }

    pub async fn history(
        &self,
        _conversation_id: ConvId,
        _query: &HistoryQuery,
    ) -> ChatResult<MessagesPage> {
        Err(ChatError::Unimplemented)
    }

    pub async fn delete_message(&self, _conversation_id: ConvId, _seq: Seq) -> ChatResult<()> {
        Err(ChatError::Unimplemented)
    }

    pub async fn set_ttl(&self, _conversation_id: ConvId, _ttl_s: Option<u32>) -> ChatResult<()> {
        Err(ChatError::Unimplemented)
    }

    pub async fn report(&self, _body: &FileReport) -> ChatResult<()> {
        Err(ChatError::Unimplemented)
    }

    pub fn sync_options(tier: Tier, hold_ms: u64) -> CallOptions {
        CallOptions {
            tier: Some(tier),
            timeout_ms: hold_ms + super::ANSWER_SLACK_MS,
            long_poll: hold_ms > 0,
        }
    }
}
