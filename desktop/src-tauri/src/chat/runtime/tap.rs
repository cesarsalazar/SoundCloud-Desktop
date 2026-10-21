use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use chat_core::{ConvId, CoreEvent, Events, UserId};

const TYPING_FOR: Duration = Duration::from_secs(6);

pub struct Tap {
    inner: Arc<dyn Events>,
    typing: Mutex<HashMap<ConvId, (Vec<UserId>, Instant)>>,
}

impl Tap {
    pub fn new(inner: Arc<dyn Events>) -> Self {
        Self {
            inner,
            typing: Mutex::new(HashMap::new()),
        }
    }

    pub fn typing(&self, conversation_id: ConvId) -> Vec<UserId> {
        let Ok(typing) = self.typing.lock() else {
            return Vec::new();
        };
        typing
            .get(&conversation_id)
            .filter(|(_, at)| at.elapsed() < TYPING_FOR)
            .map(|(users, _)| users.clone())
            .unwrap_or_default()
    }
}

impl Events for Tap {
    fn emit(&self, event: CoreEvent) {
        if let CoreEvent::Typing(typing) = &event
            && let Ok(mut known) = self.typing.lock()
        {
            known.retain(|_, (_, at)| at.elapsed() < TYPING_FOR);
            known.insert(
                typing.conversation_id,
                (typing.users.clone(), Instant::now()),
            );
        }
        self.inner.emit(event);
    }
}
