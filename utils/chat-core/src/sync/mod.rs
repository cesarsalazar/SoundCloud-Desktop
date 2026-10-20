use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::api::Api;
use crate::contracts::{ChatError, ChatResult, ConvId, Events, HIDDEN_SYNC_EVERY_MS, Seq, Tier};
use crate::store::Store;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PollPlan {
    pub tier: Tier,
    pub hold_ms: u64,
    pub delay_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncOutcome {
    Changed,
    Idle,
    Reset,
    Retry { after_ms: u64 },
}

pub struct SyncEngine {
    api: Arc<Api>,
    store: Arc<Store>,
    events: Arc<dyn Events>,
    visible: AtomicBool,
    focus: Mutex<Option<ConvId>>,
}

impl SyncEngine {
    pub fn new(api: Arc<Api>, store: Arc<Store>, events: Arc<dyn Events>) -> Self {
        Self {
            api,
            store,
            events,
            visible: AtomicBool::new(false),
            focus: Mutex::new(None),
        }
    }

    pub fn api(&self) -> &Arc<Api> {
        &self.api
    }

    pub fn store(&self) -> &Arc<Store> {
        &self.store
    }

    pub fn events(&self) -> &Arc<dyn Events> {
        &self.events
    }

    pub fn visible(&self) -> bool {
        self.visible.load(Ordering::Relaxed)
    }

    pub fn set_visible(&self, visible: bool) -> bool {
        let was = self.visible.swap(visible, Ordering::Relaxed);
        visible && !was
    }

    pub fn focus(&self) -> Option<ConvId> {
        self.focus.lock().ok().and_then(|focus| *focus)
    }

    pub fn set_focus(&self, conversation_id: Option<ConvId>) {
        if let Ok(mut focus) = self.focus.lock() {
            *focus = conversation_id;
        }
    }

    pub fn plan(&self, tier: Tier) -> PollPlan {
        if self.visible() {
            PollPlan {
                tier,
                hold_ms: tier.hold_max_ms(),
                delay_ms: 0,
            }
        } else {
            PollPlan {
                tier,
                hold_ms: 0,
                delay_ms: HIDDEN_SYNC_EVERY_MS,
            }
        }
    }

    pub fn queue_read(&self, _conversation_id: ConvId, _seq: Seq) {}

    pub async fn step(&self, _plan: PollPlan) -> ChatResult<SyncOutcome> {
        Err(ChatError::Unimplemented)
    }
}
