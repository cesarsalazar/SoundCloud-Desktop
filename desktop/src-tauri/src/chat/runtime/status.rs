use std::sync::{Arc, Mutex};

use chat_core::{ChatStatus, CoreEvent, Events};

pub struct StatusCell {
    status: Mutex<ChatStatus>,
    events: Arc<dyn Events>,
}

impl StatusCell {
    pub fn new(events: Arc<dyn Events>) -> Self {
        Self {
            status: Mutex::new(ChatStatus::off()),
            events,
        }
    }

    pub fn get(&self) -> ChatStatus {
        self.status
            .lock()
            .map(|status| status.clone())
            .unwrap_or_else(|_| ChatStatus::off())
    }

    pub fn update(&self, change: impl FnOnce(&mut ChatStatus)) {
        let changed = {
            let Ok(mut status) = self.status.lock() else {
                return;
            };
            let before = status.clone();
            change(&mut status);
            (*status != before).then(|| status.clone())
        };
        if let Some(status) = changed {
            self.events.emit(CoreEvent::Status(status));
        }
    }
}
