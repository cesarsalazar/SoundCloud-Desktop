use chat_wire::proto::{Bind, Envelope, Inner};
use zeroize::Zeroizing;

use crate::contracts::{ChatError, ChatResult, ClientId, ConvId, DeviceId, KeyStore, MessageClass};

pub struct DeviceKeys {
    auth_seed: Zeroizing<[u8; 32]>,
    box_secret: Zeroizing<[u8; 32]>,
}

impl DeviceKeys {
    pub fn from_seeds(auth_seed: [u8; 32], box_secret: [u8; 32]) -> Self {
        Self {
            auth_seed: Zeroizing::new(auth_seed),
            box_secret: Zeroizing::new(box_secret),
        }
    }

    pub fn load_or_create(_store: &dyn KeyStore) -> ChatResult<(DeviceId, Self)> {
        Err(ChatError::Unimplemented)
    }

    pub fn auth_public(&self) -> [u8; 32] {
        chat_wire::sign::public_key(&self.auth_seed)
    }

    pub fn box_public(&self) -> ChatResult<[u8; 32]> {
        Ok(chat_wire::ck::box_public(&self.box_secret)?)
    }

    pub fn bind(&self, _device_id: DeviceId, _ts_ms: u64) -> ChatResult<Bind> {
        Err(ChatError::Unimplemented)
    }

    pub fn sign(&self, _inner: &mut Inner, _enc: &[u8; 32]) -> ChatResult<()> {
        Err(ChatError::Unimplemented)
    }

    pub fn open_ck(
        &self,
        _conversation_id: ConvId,
        _epoch: u32,
        _device_id: DeviceId,
        _sealed: &[u8],
    ) -> ChatResult<Ck> {
        Err(ChatError::Unimplemented)
    }
}

pub struct Ck(Zeroizing<[u8; 32]>);

impl Ck {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(Zeroizing::new(bytes))
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageAt {
    pub conversation_id: ConvId,
    pub epoch: u32,
    pub sender_device: DeviceId,
    pub client_id: ClientId,
    pub class: MessageClass,
}

pub fn seal_message(_ck: &Ck, _at: &MessageAt, _envelope: &Envelope) -> ChatResult<Vec<u8>> {
    Err(ChatError::Unimplemented)
}

pub fn open_message(_ck: &Ck, _at: &MessageAt, _body: &[u8]) -> ChatResult<Envelope> {
    Err(ChatError::Unimplemented)
}

pub fn open_meta(
    _ck: &Ck,
    _conversation_id: ConvId,
    _epoch: u32,
    _meta_ct: &[u8],
) -> ChatResult<Envelope> {
    Err(ChatError::Unimplemented)
}
