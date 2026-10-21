use tokio::sync::{Notify, watch};

pub struct Signals {
    kick: Notify,
    outbox: Notify,
    wake: Notify,
    visible: watch::Sender<bool>,
    generation: watch::Sender<u64>,
}

impl Signals {
    pub fn new() -> Self {
        Self {
            kick: Notify::new(),
            outbox: Notify::new(),
            wake: Notify::new(),
            visible: watch::Sender::new(false),
            generation: watch::Sender::new(0),
        }
    }

    pub fn kick(&self) {
        self.kick.notify_one();
    }

    pub async fn kicked(&self) {
        self.kick.notified().await;
    }

    pub fn outbox(&self) {
        self.outbox.notify_one();
    }

    pub async fn outbox_filled(&self) {
        self.outbox.notified().await;
    }

    pub fn wake(&self) {
        self.wake.notify_waiters();
    }

    pub async fn woke(&self) {
        self.wake.notified().await;
    }

    pub fn visible(&self) -> bool {
        *self.visible.borrow()
    }

    pub fn set_visible(&self, visible: bool) -> bool {
        let changed = self.visible.send_if_modified(|current| {
            let changed = *current != visible;
            *current = visible;
            changed
        });
        if changed && visible {
            self.kick();
        }
        changed
    }

    pub fn generation(&self) -> u64 {
        *self.generation.borrow()
    }

    pub fn restart(&self) {
        self.generation.send_modify(|generation| *generation += 1);
    }

    pub async fn restarted(&self, since: u64) {
        let mut generation = self.generation.subscribe();
        let _ = generation.wait_for(|current| *current != since).await;
    }
}
