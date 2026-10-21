use std::sync::Arc;
use std::time::Duration;

use chat_core::api::{Api, CallOptions};
use chat_core::channel::InnerCall;
use chat_core::{ChatError, ChatPhase, ChatResult, HistoryGate};

use super::bringup::Halt;
use super::local::Core;
use super::{Runtime, clock};

const OFF_RECHECK: Duration = Duration::from_secs(3600);
const OFF_SPREAD_MS: u64 = 600_000;
const OWN_DEVICE: &str = "/chat/devices/me";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthMove {
    Stay,
    Restart,
    DropDevice,
}

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
        let was_signed_in = self.session.signed_in();
        let changed = self.session.changed(token.as_deref());
        match auth_move(was_signed_in, token.is_some(), changed, self.status().phase) {
            AuthMove::DropDevice => self.drop_device().await,
            AuthMove::Restart => {
                self.session.log_in();
                self.restart();
            }
            AuthMove::Stay => {}
        }
    }

    async fn drop_device(&self) {
        let identity = self.api.identity();
        let _ = self.wipe(true).await;
        let Some(identity) = identity else {
            return;
        };
        let api = Api::new(self.api.channel().clone(), self.api.transport().clone());
        api.set_identity(Some(identity));
        self.rt.spawn(async move {
            let _ = api.call(own_device_removal(), CallOptions::default()).await;
        });
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
                let generation = if phase == ChatPhase::Revoked {
                    let _ = self.wipe(true).await;
                    generation + 1
                } else {
                    generation
                };
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

pub fn auth_move(
    was_signed_in: bool,
    signed_in: bool,
    changed: bool,
    phase: ChatPhase,
) -> AuthMove {
    if was_signed_in && !signed_in {
        return AuthMove::DropDevice;
    }
    let stuck = matches!(
        phase,
        ChatPhase::Unbound | ChatPhase::BoundOther | ChatPhase::Revoked
    );
    if changed || (signed_in && stuck) {
        AuthMove::Restart
    } else {
        AuthMove::Stay
    }
}

fn own_device_removal() -> InnerCall {
    InnerCall {
        method: "DELETE".to_string(),
        path: OWN_DEVICE.to_string(),
        query: String::new(),
        body: Vec::new(),
    }
}
