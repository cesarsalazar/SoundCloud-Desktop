use std::sync::Arc;
use std::time::Duration;

use chat_core::api::wire::{SendAnswer, SendMessage};
use chat_core::chat_wire::prost::Message as _;
use chat_core::chat_wire::proto::Envelope;
use chat_core::crypto::{self, MessageAt};
use chat_core::outbox::OutboxItem;
use chat_core::{
    ChatError, ChatResult, ClientId, ConvId, CoreEvent, Events, HistoryQuery, MessagesEvent,
    MessagesOrigin, Seq,
};

use super::Runtime;
use super::keys;
use super::local::Core;

const BATCH: usize = 16;
const IDLE: Duration = Duration::from_secs(30);
const RETRY_BASE: Duration = Duration::from_secs(2);
const RETRY_CAP: Duration = Duration::from_secs(300);
const LIMITED: Duration = Duration::from_secs(30);
const PAUSED: Duration = Duration::from_secs(60);
const SLOWED_MIN: Duration = Duration::from_secs(1);

pub async fn run(runtime: Arc<Runtime>, core: Arc<Core>, generation: u64) {
    loop {
        let wait = runtime.drain(&core).await;
        tokio::select! {
            _ = tokio::time::sleep(wait) => {}
            _ = runtime.signals.outbox_filled() => {}
            _ = runtime.signals.restarted(generation) => return,
        }
    }
}

pub fn retry_in(error: &ChatError, attempts: u32) -> Option<Duration> {
    match error {
        ChatError::NotMember
        | ChatError::Muted { .. }
        | ChatError::Unavailable
        | ChatError::RequestLimit
        | ChatError::TooYoung
        | ChatError::Invalid
        | ChatError::NotFound => None,
        ChatError::SlowMode { retry_ms } | ChatError::Busy { retry_ms } => {
            Some(Duration::from_millis(*retry_ms).max(SLOWED_MIN))
        }
        ChatError::SendLimited => Some(LIMITED),
        ChatError::SendPaused | ChatError::Disabled => Some(PAUSED),
        ChatError::Clock { .. } | ChatError::EpochStale => Some(SLOWED_MIN),
        _ => Some(
            RETRY_BASE
                .saturating_mul(1 << attempts.min(8))
                .min(RETRY_CAP),
        ),
    }
}

impl Runtime {
    async fn drain(&self, core: &Core) -> Duration {
        let Ok(due) = core.outbox.due(self.clock.now_ms(), BATCH) else {
            return IDLE;
        };
        let full = due.len() == BATCH;
        let mut soonest = IDLE;
        for item in due {
            if let Some(retry) = self.deliver(core, item).await {
                soonest = soonest.min(retry);
            }
        }
        if full { Duration::ZERO } else { soonest }
    }

    async fn deliver(&self, core: &Core, item: OutboxItem) -> Option<Duration> {
        let mut sent = self.send_once(core, &item).await;
        if matches!(sent, Err(ChatError::EpochStale)) {
            self.epochs.forget(item.conversation_id);
            sent = self.send_once(core, &item).await;
        }
        match sent {
            Ok(answer) => {
                let _ = core.outbox.sent(item.client_id, answer.seq, answer.ts);
                self.announce_sent(core, item.conversation_id, answer.seq);
                None
            }
            Err(error) => {
                if let ChatError::Clock { server_now } = error {
                    self.api.channel().note_server_now(server_now);
                }
                let retry = retry_in(&error, item.attempts);
                let retry_at = retry.map(|retry| {
                    self.clock.now_ms() + i64::try_from(retry.as_millis()).unwrap_or(i64::MAX)
                });
                let _ = core.outbox.failed(item.client_id, retry_at);
                self.announce_pending(core, item.conversation_id, item.client_id);
                retry
            }
        }
    }

    async fn send_once(&self, core: &Core, item: &OutboxItem) -> ChatResult<SendAnswer> {
        let conversation_id = item.conversation_id;
        let epoch = keys::current(&self.api, core, &self.epochs, conversation_id).await?;
        let ck = keys::ck(&self.api, core, conversation_id, epoch).await?;
        let envelope =
            Envelope::decode(item.envelope.as_slice()).map_err(|_| ChatError::Protocol)?;
        let at = MessageAt {
            conversation_id,
            epoch,
            sender_device: core.device_id,
            client_id: item.client_id,
            class: item.class,
        };
        let body = crypto::seal_message(&ck, &at, &envelope)?;
        let message = SendMessage {
            client_id: item.client_id,
            class: item.class.as_str().to_string(),
            epoch: i32::try_from(epoch).map_err(|_| ChatError::Protocol)?,
            target_seq: item.target_seq,
            body,
        };
        self.api.send(conversation_id, &message).await
    }

    fn announce_sent(&self, core: &Core, conversation_id: ConvId, seq: Seq) {
        let query = HistoryQuery {
            before: None,
            after: Some(seq - 1),
            limit: 1,
        };
        if let Ok(page) = core.store.messages(conversation_id, &query) {
            let messages = page
                .messages
                .into_iter()
                .filter(|message| message.seq == Some(seq))
                .collect();
            self.announce(conversation_id, messages);
        }
    }

    fn announce_pending(&self, core: &Core, conversation_id: ConvId, client_id: ClientId) {
        if let Ok(pending) = core.outbox.pending(conversation_id) {
            let messages = pending
                .into_iter()
                .filter(|message| message.client_id == client_id)
                .collect();
            self.announce(conversation_id, messages);
        }
    }

    pub(super) fn announce(&self, conversation_id: ConvId, messages: Vec<chat_core::Message>) {
        if messages.is_empty() {
            return;
        }
        self.events.emit(CoreEvent::Messages(MessagesEvent {
            conversation_id,
            origin: MessagesOrigin::Local,
            messages,
        }));
    }
}
