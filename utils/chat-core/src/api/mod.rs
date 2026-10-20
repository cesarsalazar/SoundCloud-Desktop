mod routes;
pub mod wire;

use std::sync::{Arc, RwLock};

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::channel::{Channel, InnerAnswer, InnerCall};
use crate::contracts::{ChatError, ChatResult, DeviceId, Tier, Transport};
use crate::crypto::DeviceKeys;

pub const ANSWER_SLACK_MS: u64 = 15_000;
pub const CALL_TIMEOUT_MS: u64 = 20_000;

#[derive(Clone)]
pub struct Identity {
    pub session_id: String,
    pub device_id: DeviceId,
    pub keys: Arc<DeviceKeys>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallOptions {
    pub tier: Option<Tier>,
    pub timeout_ms: u64,
    pub long_poll: bool,
}

impl Default for CallOptions {
    fn default() -> Self {
        Self {
            tier: None,
            timeout_ms: CALL_TIMEOUT_MS,
            long_poll: false,
        }
    }
}

pub struct Api {
    channel: Arc<Channel>,
    transport: Arc<dyn Transport>,
    identity: RwLock<Option<Identity>>,
}

impl Api {
    pub fn new(channel: Arc<Channel>, transport: Arc<dyn Transport>) -> Self {
        Self {
            channel,
            transport,
            identity: RwLock::new(None),
        }
    }

    pub fn channel(&self) -> &Arc<Channel> {
        &self.channel
    }

    pub fn transport(&self) -> &Arc<dyn Transport> {
        &self.transport
    }

    pub fn identity(&self) -> Option<Identity> {
        self.identity
            .read()
            .ok()
            .and_then(|identity| identity.clone())
    }

    pub fn set_identity(&self, identity: Option<Identity>) {
        if let Ok(mut current) = self.identity.write() {
            *current = identity;
        }
    }

    pub async fn call(&self, _call: InnerCall, _options: CallOptions) -> ChatResult<InnerAnswer> {
        Err(ChatError::Unimplemented)
    }

    pub async fn json<T, B>(
        &self,
        _method: &str,
        _path: &str,
        _query: &str,
        _body: Option<&B>,
        _options: CallOptions,
    ) -> ChatResult<T>
    where
        T: DeserializeOwned,
        B: Serialize + Sync,
    {
        Err(ChatError::Unimplemented)
    }
}
