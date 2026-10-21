use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use chat_core::ConvId;

use super::local::Core;
use super::{Runtime, counts};

const READ_SETTLE: Duration = Duration::from_millis(300);
const TYPING_EVERY: Duration = Duration::from_secs(3);

impl Runtime {
    pub fn read_soon(self: &Arc<Self>) {
        if self.read_pending.swap(true, Ordering::AcqRel) {
            return;
        }
        let runtime = self.clone();
        self.rt.spawn(async move {
            tokio::time::sleep(READ_SETTLE).await;
            runtime.read_pending.store(false, Ordering::Release);
            runtime.kick();
            if let Some(core) = runtime.core() {
                runtime.recount(&core);
            }
        });
    }

    pub fn may_type(&self, conversation_id: ConvId) -> bool {
        let Ok(mut sent) = self.typing_sent.lock() else {
            return false;
        };
        let now = Instant::now();
        sent.retain(|_, at| now.duration_since(*at) < TYPING_EVERY);
        if sent.contains_key(&conversation_id) {
            return false;
        }
        sent.insert(conversation_id, now);
        true
    }

    pub fn recount(&self, core: &Core) {
        let store = core.store.clone();
        let status = self.status.clone();
        let now = self.clock.now_ms();
        self.rt.spawn_blocking(move || {
            if let Ok(counts) = counts::count(&store, now) {
                status.update(|status| {
                    status.unread = counts.unread;
                    status.mentions = counts.mentions;
                    status.requests = counts.requests;
                });
            }
        });
    }
}
