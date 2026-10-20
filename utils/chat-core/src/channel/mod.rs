use std::sync::Arc;
use std::sync::atomic::{AtomicI64, AtomicU8, Ordering};

use chat_wire::seal::{ENC_LEN, ResponseKey};

use crate::contracts::{ChatError, ChatResult, Clock, DeviceId, Millis, Tier};
use crate::crypto::DeviceKeys;

pub const CHANNEL_PATH: &str = "/chat/x";
const FIRST_OCK: u8 = chat_wire::ock::OCK_PINS[0].0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InnerCall {
    pub method: String,
    pub path: String,
    pub query: String,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InnerAnswer {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

pub struct Caller<'a> {
    pub session_id: &'a str,
    pub device_id: DeviceId,
    pub keys: &'a DeviceKeys,
}

pub struct Sealed {
    pub envelope: Vec<u8>,
    pub enc: [u8; ENC_LEN],
    pub response_key: ResponseKey,
}

impl Sealed {
    pub fn open(&self, _body: &[u8]) -> ChatResult<InnerAnswer> {
        Err(ChatError::Unimplemented)
    }
}

pub struct Channel {
    clock: Arc<dyn Clock>,
    offset_ms: AtomicI64,
    ock_version: AtomicU8,
}

impl Channel {
    pub fn new(clock: Arc<dyn Clock>) -> Self {
        Self {
            clock,
            offset_ms: AtomicI64::new(0),
            ock_version: AtomicU8::new(FIRST_OCK),
        }
    }

    pub fn now_ms(&self) -> Millis {
        self.clock.now_ms() + self.offset_ms.load(Ordering::Relaxed)
    }

    pub fn note_server_now(&self, server_now: Millis) {
        self.offset_ms
            .store(server_now - self.clock.now_ms(), Ordering::Relaxed);
    }

    pub fn ock_version(&self) -> u8 {
        self.ock_version.load(Ordering::Relaxed)
    }

    pub fn use_ock(&self, version: u8) -> bool {
        let pinned = chat_wire::ock::pinned(version).is_some();
        if pinned {
            self.ock_version.store(version, Ordering::Relaxed);
        }
        pinned
    }

    pub fn seal(&self, _call: &InnerCall, _caller: &Caller<'_>, _tier: Tier) -> ChatResult<Sealed> {
        Err(ChatError::Unimplemented)
    }
}
