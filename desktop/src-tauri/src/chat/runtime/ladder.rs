use std::time::{Duration, Instant};

use chat_core::Tier;

const BACKOFF_MS: [u64; 5] = [1_000, 2_000, 4_000, 8_000, 15_000];
const REVISIT: Duration = Duration::from_secs(600);

pub struct Ladder {
    tiers: Vec<Tier>,
    at: usize,
    misses: usize,
    rounds: usize,
    left_first_at: Option<Instant>,
}

impl Ladder {
    pub fn new(tiers: Vec<Tier>) -> Self {
        Self {
            tiers,
            at: 0,
            misses: 0,
            rounds: 0,
            left_first_at: None,
        }
    }

    pub fn tier(&self) -> Tier {
        self.tiers.get(self.at).copied().unwrap_or(Tier::Direct)
    }

    pub fn follow(&mut self, tiers: Vec<Tier>) {
        if tiers != self.tiers {
            self.tiers = tiers;
            self.restart();
        }
    }

    pub fn answered(&mut self, now: Instant) {
        self.misses = 0;
        self.rounds = 0;
        let stayed_long = self
            .left_first_at
            .is_some_and(|since| now.saturating_duration_since(since) >= REVISIT);
        if stayed_long {
            self.at = 0;
            self.left_first_at = None;
        }
    }

    pub fn missed(&mut self, now: Instant) -> Duration {
        let len = self.tiers.len().max(1);
        self.misses += 1;
        if self.misses < len {
            self.at = (self.at + 1) % len;
            self.left_first_at.get_or_insert(now);
            return Duration::ZERO;
        }
        let pause = BACKOFF_MS[self.rounds.min(BACKOFF_MS.len() - 1)];
        self.rounds += 1;
        self.misses = 0;
        self.at = 0;
        self.left_first_at = None;
        Duration::from_millis(spread(pause, super::clock::jitter(pause * 2 / 5)))
    }

    pub fn restart(&mut self) {
        self.at = 0;
        self.misses = 0;
        self.rounds = 0;
        self.left_first_at = None;
    }
}

pub fn spread(pause_ms: u64, roll_ms: u64) -> u64 {
    pause_ms - pause_ms / 5 + roll_ms.min(pause_ms * 2 / 5)
}
