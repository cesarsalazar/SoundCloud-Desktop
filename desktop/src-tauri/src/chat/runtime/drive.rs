use std::sync::Arc;

use super::Runtime;
use super::bringup::Halt;
use super::local::Core;

const FIRST_FAILURE: u32 = 1;

impl Runtime {
    pub(super) async fn drive(&self, core: Arc<Core>, generation: u64) -> Halt {
        core.sync.set_visible(self.signals.visible());
        if let Some(cap) = self
            .config()
            .and_then(|config| hold_cap(config.hold_max_ms))
        {
            core.sync.set_hold_cap(cap);
        }
        tokio::select! {
            ran = core.sync.run(self.clock.as_ref()) => match ran {
                Ok(()) => Halt::Restart,
                Err(error) => self.halt(&error, FIRST_FAILURE),
            },
            _ = self.signals.restarted(generation) => Halt::Restart,
        }
    }
}

pub fn hold_cap(hold_max_ms: u64) -> Option<u64> {
    (hold_max_ms > 0).then_some(hold_max_ms)
}
