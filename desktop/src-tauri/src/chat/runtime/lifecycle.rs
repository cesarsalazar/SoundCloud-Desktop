use std::sync::Arc;
use std::time::Duration;

use chat_core::{ChatError, ChatPhase, ChatResult, HistoryGate};

use super::bringup::Halt;
use super::local::Core;
use super::{Runtime, clock};

const OFF_RECHECK: Duration = Duration::from_secs(3600);
const OFF_SPREAD_MS: u64 = 600_000;

impl Runtime {
    pub(super) fn reroute(&self) {
        let preferred = self.api.transport().tiers().first().copied();
        if preferred.is_some() && preferred != self.status.get().tier {
            self.kick();
        }
    }

    pub fn relogged(&self) {
        self.session.log_in();
        self.restart();
    }

    pub async fn log_out(&self) -> ChatResult<()> {
        self.session.log_out();
        self.wipe(true).await
    }

    pub(super) async fn auth_changed(&self) {
        let token = crate::auth::session_token(Some(&self.app)).await;
        let rebound = token.is_some()
            && matches!(
                self.status().phase,
                ChatPhase::Unbound | ChatPhase::BoundOther
            );
        if self.session.changed(token.as_deref()) || rebound {
            self.session.log_in();
            self.restart();
        }
    }

    pub async fn wipe(&self, forget_device: bool) -> ChatResult<()> {
        let _lifecycle = self.lifecycle.lock().await;
        let core = self.core.write().ok().and_then(|mut core| core.take());
        self.api.set_identity(None);
        self.epochs.clear();
        self.restart();
        let keychain = self.keychain.clone();
        let wiped = self
            .rt
            .spawn_blocking(move || {
                if let Some(core) = core {
                    core.store.wipe()?;
                }
                if forget_device {
                    keychain.wipe()?;
                }
                Ok(())
            })
            .await
            .unwrap_or(Err(ChatError::Internal));
        self.status.update(|status| {
            status.device_id = None;
            status.history = HistoryGate::default();
            status.unread = 0;
            status.mentions = 0;
            status.requests = 0;
        });
        wiped
    }

    pub(super) fn set_core(&self, core: Option<Arc<Core>>) {
        if let Ok(mut current) = self.core.write() {
            *current = core;
        }
    }

    pub(super) async fn run(self: Arc<Self>) {
        let mut failures = 0;
        loop {
            let generation = self.signals.generation();
            let halt = match self.bring_up(&mut failures).await {
                Ok(core) => {
                    failures = 0;
                    self.drive(core, generation).await
                }
                Err(halt) => halt,
            };
            self.settle(halt, generation).await;
        }
    }

    pub(super) async fn settle(&self, halt: Halt, generation: u64) {
        match halt {
            Halt::Restart => {}
            Halt::Retry(pause) => {
                self.status.update(|status| status.online = false);
                self.pause(pause, generation).await;
            }
            Halt::Phase(phase) => {
                self.status.update(|status| {
                    status.phase = phase;
                    status.online = false;
                });
                if phase == ChatPhase::Off {
                    let recheck = OFF_RECHECK + Duration::from_millis(clock::jitter(OFF_SPREAD_MS));
                    tokio::select! {
                        _ = tokio::time::sleep(recheck) => {}
                        _ = self.signals.restarted(generation) => {}
                    }
                } else {
                    self.signals.restarted(generation).await;
                }
            }
        }
    }
}
