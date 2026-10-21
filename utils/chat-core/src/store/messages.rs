use rusqlite::{Connection, OptionalExtension, params};

use super::items::{self, Item};
use super::project::{self, Context};
use super::{Store, StoreKeys, StoredContent, StoredMessage, content, embeds, purge};
use crate::contracts::{ChatResult, ConvId, MessageClass, Seq};
use crate::envelope::Content;
use crate::search::index;

const UPSERT: &str = "INSERT INTO messages (conversation_id, seq, client_id, sender, sender_device, class, epoch, target_seq, ts, expires_at, local_only, content, nonce, body) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14) ON CONFLICT (conversation_id, seq) DO UPDATE SET client_id = excluded.client_id, sender = excluded.sender, sender_device = excluded.sender_device, class = excluded.class, epoch = excluded.epoch, target_seq = excluded.target_seq, ts = excluded.ts, expires_at = excluded.expires_at, local_only = excluded.local_only, content = excluded.content, nonce = excluded.nonce, body = excluded.body";

impl Store {
    pub fn put_messages(&self, items: &[StoredMessage]) -> ChatResult<()> {
        let mut conn = self.conn()?;
        let tx = conn.transaction()?;
        let mut removed = false;
        for item in items {
            removed |= write(&tx, &self.keys, item)?;
        }
        tx.commit()?;
        if removed {
            purge::finish(&conn)?;
        }
        Ok(())
    }

    pub fn sealed_messages(&self, id: ConvId, epoch: u32) -> ChatResult<Vec<StoredMessage>> {
        let conn = self.conn()?;
        let items = items::load(
            &conn,
            &self.keys,
            "conversation_id = ?1 AND epoch = ?2 AND content = 'sealed' ORDER BY seq",
            params![id.as_bytes(), epoch],
        )?;
        Ok(items.into_iter().map(|item| item.message).collect())
    }

    pub fn set_key_epoch(&self, id: ConvId, epoch: u32) -> ChatResult<()> {
        self.conn()?.execute(
            "UPDATE conversations SET key_epoch = ?2 WHERE id = ?1",
            params![id.as_bytes(), epoch],
        )?;
        Ok(())
    }

    pub fn key_epoch(&self, id: ConvId) -> ChatResult<Option<u32>> {
        Ok(self
            .conn()?
            .query_row(
                "SELECT key_epoch FROM conversations WHERE id = ?1",
                [id.as_bytes()],
                |row| row.get(0),
            )
            .optional()?)
    }
}

pub fn write(tx: &Connection, keys: &StoreKeys, item: &StoredMessage) -> ChatResult<bool> {
    let conv = item.conversation_id;
    if !item.local_only && item.seq <= hidden_before(tx, conv)? {
        return Ok(false);
    }
    let (kind, sealed) = items::seal_content(keys, conv, item.seq, &item.content)?;
    let (nonce, body) = match sealed {
        Some(sealed) => (Some(sealed.nonce), Some(sealed.body)),
        None => (None, None),
    };
    let previous: Option<String> = tx
        .prepare_cached("SELECT content FROM messages WHERE conversation_id = ?1 AND seq = ?2")?
        .query_row(params![conv.as_bytes(), item.seq], |row| row.get(0))
        .optional()?;
    tx.prepare_cached(UPSERT)?.execute(params![
        conv.as_bytes(),
        item.seq,
        item.client_id.as_bytes(),
        item.sender,
        item.sender_device.map(|id| *id.as_bytes()),
        item.class.as_str(),
        item.epoch,
        item.target_seq,
        item.ts,
        item.expires_at,
        item.local_only as i64,
        kind,
        nonce,
        body,
    ])?;
    tx.prepare_cached("DELETE FROM outbox WHERE client_id = ?1 AND conversation_id = ?2")?
        .execute(params![item.client_id.as_bytes(), conv.as_bytes()])?;
    reindex(tx, keys, conv, item.seq, previous.is_none())?;
    let mut cleared = previous.is_some_and(|old| old != "deleted") && kind == "deleted";
    if item.class == MessageClass::Edit
        && let Some(target) = item.target_seq
    {
        refresh(tx, keys, conv, target)?;
        cleared |= deletes_target(&item.content);
    }
    Ok(cleared)
}

pub fn refresh(tx: &Connection, keys: &StoreKeys, conv: ConvId, seq: Seq) -> ChatResult<()> {
    reindex(tx, keys, conv, seq, false)
}

fn reindex(
    tx: &Connection,
    keys: &StoreKeys,
    conv: ConvId,
    seq: Seq,
    fresh: bool,
) -> ChatResult<()> {
    let Some(item) = items::by_seq(tx, keys, conv, seq)? else {
        return Ok(());
    };
    if !project::visible(item.message.class) {
        return Ok(());
    }
    let edits = items::edits(tx, keys, conv, seq)?;
    let modifiers: Vec<&Item> = edits.iter().collect();
    let message = project::message(&item, &modifiers, Context::default());
    let text = project::searchable(&message);
    if fresh {
        index::insert(tx, &keys.search, item.id, text.as_deref())?;
    } else {
        index::replace(tx, &keys.search, item.id, text.as_deref())?;
    }
    embeds::replace(tx, keys, item.id, conv, &message.embeds)
}

fn deletes_target(stored: &StoredContent) -> bool {
    match stored {
        StoredContent::Open(bytes) => matches!(content::decode(bytes), Content::Delete { .. }),
        _ => false,
    }
}

pub fn hidden_before(conn: &Connection, conv: ConvId) -> ChatResult<Seq> {
    Ok(conn
        .prepare_cached("SELECT hidden_before_seq FROM conversations WHERE id = ?1")?
        .query_row([conv.as_bytes()], |row| row.get(0))
        .optional()?
        .unwrap_or_default())
}
