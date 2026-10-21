use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use chat_core::channel::{Channel, InnerAnswer};
use chat_core::{ChatResult, Clock, Millis, Tier, Transport, async_trait};

use super::exchange::{Call, Signer, attempt, over_tiers};

const HOLD_FOR: Duration = Duration::from_secs(20 * 60);

pub struct Lane {
    channel: Channel,
    last: Mutex<Option<Tier>>,
    held: Mutex<HashMap<String, Held>>,
}

struct Held {
    signer: Arc<Signer>,
    since: Instant,
}

impl Lane {
    pub fn new() -> Self {
        Self {
            channel: Channel::new(Arc::new(WallClock)),
            last: Mutex::new(None),
            held: Mutex::new(HashMap::new()),
        }
    }

    pub async fn send(
        &self,
        transport: &dyn Transport,
        signer: &Signer,
        call: &Call,
    ) -> ChatResult<InnerAnswer> {
        let order = tier_order(transport.tiers(), *lock(&self.last));
        let (tier, answer) = over_tiers(order, |tier| {
            attempt(&self.channel, transport, signer, call, tier)
        })
        .await?;
        *lock(&self.last) = Some(tier);
        Ok(answer)
    }

    pub fn hold(&self, request_id: &str, signer: Arc<Signer>) {
        let mut held = lock(&self.held);
        held.retain(|_, entry| entry.since.elapsed() < HOLD_FOR);
        held.insert(
            request_id.to_owned(),
            Held {
                signer,
                since: Instant::now(),
            },
        );
    }

    pub fn held(&self, request_id: &str) -> Option<Arc<Signer>> {
        lock(&self.held)
            .get(request_id)
            .map(|entry| entry.signer.clone())
    }

    pub fn release(&self, request_id: &str) {
        lock(&self.held).remove(request_id);
    }
}

pub fn tier_order(tiers: Vec<Tier>, last: Option<Tier>) -> Vec<Tier> {
    let mut order = tiers;
    if let Some(at) = last.and_then(|last| order.iter().position(|tier| *tier == last)) {
        order[..=at].rotate_right(1);
    }
    order
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

struct WallClock;

#[async_trait]
impl Clock for WallClock {
    fn now_ms(&self) -> Millis {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|since| Millis::try_from(since.as_millis()).unwrap_or(Millis::MAX))
            .unwrap_or_default()
    }

    async fn sleep(&self, ms: u64) {
        tokio::time::sleep(Duration::from_millis(ms)).await;
    }
}
