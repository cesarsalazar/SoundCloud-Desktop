use std::sync::Arc;
use std::time::{Duration, Instant};

use chat_core::sync::{PollPlan, SyncOutcome};
use chat_core::{ChatError, ChatPhase, ChatResult, Tier};

use super::Runtime;
use super::bringup::Halt;
use super::ladder::Ladder;
use super::local::Core;

const STUB_RETRY: Duration = Duration::from_secs(60);
const BUSY_MIN: Duration = Duration::from_secs(1);
const CLOCK_RETRY: Duration = Duration::from_secs(1);
const STEP_GAP: Duration = Duration::from_millis(250);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Next {
    Go,
    Fallback,
    Wait(Duration),
    Stop(ChatPhase),
}

enum Stepped {
    Done(ChatResult<SyncOutcome>),
    Interrupted,
    Restarted,
}

impl Runtime {
    pub(super) async fn drive(self: &Arc<Self>, core: Arc<Core>, generation: u64) -> Halt {
        let sender = self
            .rt
            .spawn(super::sender::run(self.clone(), core.clone(), generation));
        let halt = self.sync_until_halt(&core, generation).await;
        sender.abort();
        halt
    }

    async fn sync_until_halt(&self, core: &Core, generation: u64) -> Halt {
        let mut ladder = Ladder::new(self.api.transport().tiers());
        let mut last_step: Option<Instant> = None;
        loop {
            if self.take_wake() {
                ladder.restart();
                last_step = None;
            }
            core.sync.set_visible(self.signals.visible());
            ladder.follow(self.api.transport().tiers());
            let plan = capped(core.sync.plan(ladder.tier()), self.hold_max_ms());
            let waited = last_step.map_or(Duration::MAX, |at| at.elapsed());
            let delay = Duration::from_millis(plan.delay_ms)
                .max(STEP_GAP)
                .saturating_sub(waited);
            if !delay.is_zero() {
                if self.pause(delay, generation).await {
                    return Halt::Restart;
                }
                continue;
            }
            last_step = Some(Instant::now());
            let result = match self.step(core, plan, generation).await {
                Stepped::Restarted => return Halt::Restart,
                Stepped::Interrupted => continue,
                Stepped::Done(result) => result,
            };
            self.note_step(core, &result, plan.tier);
            let pause = match next_of(&result) {
                Next::Go => {
                    ladder.answered(Instant::now());
                    Duration::ZERO
                }
                Next::Fallback => ladder.missed(Instant::now()),
                Next::Wait(pause) => pause,
                Next::Stop(phase) => return Halt::Phase(phase),
            };
            if !pause.is_zero() && self.pause(pause, generation).await {
                return Halt::Restart;
            }
        }
    }

    async fn step(&self, core: &Core, plan: PollPlan, generation: u64) -> Stepped {
        tokio::select! {
            result = core.sync.step(plan) => Stepped::Done(result),
            _ = self.signals.kicked() => Stepped::Interrupted,
            _ = self.signals.woke() => Stepped::Interrupted,
            _ = self.signals.restarted(generation) => Stepped::Restarted,
        }
    }

    pub(super) async fn pause(&self, pause: Duration, generation: u64) -> bool {
        tokio::select! {
            _ = tokio::time::sleep(pause) => false,
            _ = self.signals.kicked() => false,
            _ = self.signals.woke() => false,
            _ = self.signals.restarted(generation) => true,
        }
    }

    fn hold_max_ms(&self) -> Option<u64> {
        self.config().map(|config| config.hold_max_ms)
    }

    fn note_step(&self, core: &Core, result: &ChatResult<SyncOutcome>, tier: Tier) {
        if let Err(ChatError::Clock { server_now }) = result {
            self.api.channel().note_server_now(*server_now);
        }
        let online = result.is_ok();
        self.status.update(|status| {
            status.online = online;
            if online {
                status.tier = Some(tier);
            }
        });
        if matches!(result, Ok(SyncOutcome::Changed | SyncOutcome::Reset)) {
            self.recount(core);
        }
    }
}

pub fn capped(plan: PollPlan, hold_max_ms: Option<u64>) -> PollPlan {
    PollPlan {
        hold_ms: hold_max_ms
            .filter(|max| *max > 0)
            .map_or(plan.hold_ms, |max| plan.hold_ms.min(max)),
        ..plan
    }
}

pub fn next_of(result: &ChatResult<SyncOutcome>) -> Next {
    let error = match result {
        Ok(SyncOutcome::Retry { after_ms }) => return Next::Wait(Duration::from_millis(*after_ms)),
        Ok(_) => return Next::Go,
        Err(error) => error,
    };
    match error {
        ChatError::Busy { retry_ms } => Next::Wait(Duration::from_millis(*retry_ms).max(BUSY_MIN)),
        ChatError::Clock { .. } => Next::Wait(CLOCK_RETRY),
        ChatError::DeviceRevoked => Next::Stop(ChatPhase::Revoked),
        ChatError::ReloginRequired => Next::Stop(ChatPhase::Unbound),
        ChatError::SessionBoundOther => Next::Stop(ChatPhase::BoundOther),
        ChatError::Disabled => Next::Stop(ChatPhase::Off),
        ChatError::Unimplemented => Next::Wait(STUB_RETRY),
        _ => Next::Fallback,
    }
}
