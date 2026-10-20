pub mod migrations;

use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use hmac::{Hmac, Mac};
use rusqlite::Connection;
use sha2::Sha256;
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::contracts::{
    ChatError, ChatResult, ClientId, ConvId, ConversationPage, ConversationSummary, DeviceId,
    EmbedPage, EmbedsQuery, GoneMessage, HistoryPage, HistoryQuery, ListQuery, Member,
    MessageClass, Millis, PeerMark, RichText, Seq, SystemEvent, UserId, UserRef,
};
use crate::crypto::Ck;

const BODY_LABEL: &[u8] = b"chat-store body";
const SEARCH_LABEL: &[u8] = b"chat-store fts";

pub struct StoreKeys {
    pub body: Zeroizing<[u8; 32]>,
    pub search: Zeroizing<[u8; 32]>,
}

impl StoreKeys {
    pub fn derive(master: &[u8; 32]) -> ChatResult<Self> {
        Ok(Self {
            body: subkey(master, BODY_LABEL)?,
            search: subkey(master, SEARCH_LABEL)?,
        })
    }
}

fn subkey(master: &[u8; 32], label: &[u8]) -> ChatResult<Zeroizing<[u8; 32]>> {
    let mut mac = Hmac::<Sha256>::new_from_slice(master).map_err(|_| ChatError::Crypto)?;
    mac.update(label);
    Ok(Zeroizing::new(mac.finalize().into_bytes().into()))
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SyncState {
    pub pos: u64,
    pub inc: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoredContent {
    Open(Vec<u8>),
    Sealed(Vec<u8>),
    System(SystemEvent),
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredMessage {
    pub conversation_id: ConvId,
    pub seq: Seq,
    pub client_id: ClientId,
    pub sender: UserId,
    pub sender_device: Option<DeviceId>,
    pub class: MessageClass,
    pub epoch: u32,
    pub target_seq: Option<Seq>,
    pub ts: Millis,
    pub expires_at: Option<Millis>,
    pub local_only: bool,
    pub content: StoredContent,
}

pub struct Store {
    conn: Mutex<Connection>,
    keys: StoreKeys,
}

impl Store {
    pub fn open(path: &Path, master: &[u8; 32]) -> ChatResult<Self> {
        Self::with_connection(Connection::open(path)?, master)
    }

    pub fn open_in_memory(master: &[u8; 32]) -> ChatResult<Self> {
        Self::with_connection(Connection::open_in_memory()?, master)
    }

    fn with_connection(conn: Connection, master: &[u8; 32]) -> ChatResult<Self> {
        migrations::prepare(&conn)?;
        migrations::migrate(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
            keys: StoreKeys::derive(master)?,
        })
    }

    pub fn keys(&self) -> &StoreKeys {
        &self.keys
    }

    pub fn conn(&self) -> ChatResult<MutexGuard<'_, Connection>> {
        self.conn.lock().map_err(|_| ChatError::Store)
    }

    pub fn sync_state(&self) -> ChatResult<SyncState> {
        Err(ChatError::Unimplemented)
    }

    pub fn set_sync_state(&self, _state: SyncState) -> ChatResult<()> {
        Err(ChatError::Unimplemented)
    }

    pub fn known(&self, _limit: usize) -> ChatResult<Vec<(ConvId, Seq)>> {
        Err(ChatError::Unimplemented)
    }

    pub fn put_users(&self, _users: &[UserRef]) -> ChatResult<()> {
        Err(ChatError::Unimplemented)
    }

    pub fn users(&self, _ids: &[UserId]) -> ChatResult<Vec<UserRef>> {
        Err(ChatError::Unimplemented)
    }

    pub fn put_conversations(&self, _items: &[ConversationSummary]) -> ChatResult<()> {
        Err(ChatError::Unimplemented)
    }

    pub fn conversations(&self, _query: &ListQuery) -> ChatResult<ConversationPage> {
        Err(ChatError::Unimplemented)
    }

    pub fn conversation(&self, _id: ConvId) -> ChatResult<Option<ConversationSummary>> {
        Err(ChatError::Unimplemented)
    }

    pub fn put_members(&self, _id: ConvId, _members: &[Member]) -> ChatResult<()> {
        Err(ChatError::Unimplemented)
    }

    pub fn members(&self, _id: ConvId) -> ChatResult<Vec<Member>> {
        Err(ChatError::Unimplemented)
    }

    pub fn put_marks(&self, _id: ConvId, _marks: &[PeerMark]) -> ChatResult<()> {
        Err(ChatError::Unimplemented)
    }

    pub fn marks(&self, _id: ConvId) -> ChatResult<Vec<PeerMark>> {
        Err(ChatError::Unimplemented)
    }

    pub fn put_messages(&self, _items: &[StoredMessage]) -> ChatResult<()> {
        Err(ChatError::Unimplemented)
    }

    pub fn messages(&self, _id: ConvId, _query: &HistoryQuery) -> ChatResult<HistoryPage> {
        Err(ChatError::Unimplemented)
    }

    pub fn embeds(&self, _id: ConvId, _query: &EmbedsQuery) -> ChatResult<EmbedPage> {
        Err(ChatError::Unimplemented)
    }

    pub fn sealed_messages(&self, _id: ConvId, _epoch: u32) -> ChatResult<Vec<StoredMessage>> {
        Err(ChatError::Unimplemented)
    }

    pub fn put_ck(&self, _id: ConvId, _epoch: u32, _ck: &Ck) -> ChatResult<()> {
        Err(ChatError::Unimplemented)
    }

    pub fn ck(&self, _id: ConvId, _epoch: u32) -> ChatResult<Option<Ck>> {
        Err(ChatError::Unimplemented)
    }

    pub fn save_draft(&self, _id: ConvId, _draft: Option<&RichText>) -> ChatResult<()> {
        Err(ChatError::Unimplemented)
    }

    pub fn draft(&self, _id: ConvId) -> ChatResult<Option<RichText>> {
        Err(ChatError::Unimplemented)
    }

    pub fn purge_conversations(&self, _ids: &[ConvId]) -> ChatResult<()> {
        Err(ChatError::Unimplemented)
    }

    pub fn purge_messages(&self, _gone: &[GoneMessage]) -> ChatResult<()> {
        Err(ChatError::Unimplemented)
    }

    pub fn purge_expired(&self, _now: Millis) -> ChatResult<Vec<GoneMessage>> {
        Err(ChatError::Unimplemented)
    }

    pub fn next_expiry(&self) -> ChatResult<Option<Millis>> {
        Err(ChatError::Unimplemented)
    }

    pub fn keep_local_above(&self, _server_last: &[(ConvId, Seq)]) -> ChatResult<()> {
        Err(ChatError::Unimplemented)
    }

    pub fn wipe(&self) -> ChatResult<()> {
        Err(ChatError::Unimplemented)
    }
}
