use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chat_core::{Clock, Millis, async_trait};

pub struct SystemClock;

#[async_trait]
impl Clock for SystemClock {
    fn now_ms(&self) -> Millis {
        wall_ms()
    }

    async fn sleep(&self, ms: u64) {
        tokio::time::sleep(Duration::from_millis(ms)).await;
    }
}

pub fn wall_ms() -> Millis {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| i64::try_from(since.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

pub fn jitter(max_ms: u64) -> u64 {
    if max_ms == 0 {
        return 0;
    }
    let roll = chat_core::chat_wire::random_bytes::<8>()
        .map(u64::from_le_bytes)
        .unwrap_or(0);
    roll % (max_ms + 1)
}
