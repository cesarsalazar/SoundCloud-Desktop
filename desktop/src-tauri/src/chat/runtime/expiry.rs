use std::sync::{Arc, Weak};
use std::time::Duration;

use chat_core::{CoreEvent, Events, GoneEvent, Millis};

use super::Runtime;
use super::local::Core;

const LOOK_AGAIN: Duration = Duration::from_secs(15);
const SOONEST: Duration = Duration::from_secs(1);

impl Runtime {
    pub(super) fn watch_expiry(self: &Arc<Self>, core: &Arc<Core>) {
        let runtime = self.clone();
        let core = Arc::downgrade(core);
        self.rt.spawn(async move {
            while let Some(current) = runtime.current(&core) {
                let wait = runtime.expire(&current);
                drop(current);
                tokio::time::sleep(wait).await;
            }
        });
    }

    fn current(&self, core: &Weak<Core>) -> Option<Arc<Core>> {
        let core = core.upgrade()?;
        let open = self.core().is_some_and(|open| Arc::ptr_eq(&open, &core));
        open.then_some(core)
    }

    fn expire(&self, core: &Core) -> Duration {
        let now = self.clock.now_ms();
        if let Ok(gone) = core.store.purge_expired(now)
            && !gone.is_empty()
        {
            self.events.emit(CoreEvent::Gone(GoneEvent {
                conversations: Vec::new(),
                messages: gone,
            }));
            self.recount(core);
        }
        let next = core.store.next_expiry().ok().flatten();
        until(next, now)
    }
}

pub fn until(next: Option<Millis>, now: Millis) -> Duration {
    let Some(next) = next else {
        return LOOK_AGAIN;
    };
    let left = u64::try_from(next.saturating_sub(now)).unwrap_or(0);
    Duration::from_millis(left).clamp(SOONEST, LOOK_AGAIN)
}
