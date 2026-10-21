use std::time::{Duration, Instant};

use chat_core::api::wire::{ConfigView, DeviceView, LimitsView};
use chat_core::sync::{PollPlan, SyncOutcome};
use chat_core::{
    Access, ChatError, ChatPhase, ConversationKind, ConversationSummary, DeviceStatus, MemberRole,
    MemberState, NotifyLevel, Tier,
};
use uuid::Uuid;

use super::bringup::{backoff, config_of, history_gate};
use super::counts::quiet;
use super::drive::{Next, capped, next_of};
use super::expiry::until;
use super::ladder::{Ladder, spread};
use super::sender::retry_in;
use super::signals::Signals;
use super::wake::WakeWatch;

const TIERS: [Tier; 3] = [Tier::Direct, Tier::Relay, Tier::Pro];

#[test]
fn a_lost_tier_moves_on_at_once_and_a_lost_round_backs_off() {
    let now = Instant::now();
    let mut ladder = Ladder::new(TIERS.to_vec());
    assert_eq!(ladder.tier(), Tier::Direct);
    assert_eq!(ladder.missed(now), Duration::ZERO);
    assert_eq!(ladder.tier(), Tier::Relay);
    assert_eq!(ladder.missed(now), Duration::ZERO);
    assert_eq!(ladder.tier(), Tier::Pro);
    let first = ladder.missed(now);
    assert!((800..=1_200).contains(&first.as_millis()), "{first:?}");
    assert_eq!(ladder.tier(), Tier::Direct);
    for _ in 0..12 {
        ladder.missed(now);
    }
    let round = ladder.missed(now) + ladder.missed(now) + ladder.missed(now);
    assert!(round.as_millis() <= 18_000, "{round:?}");
}

#[test]
fn a_working_fallback_stays_until_the_revisit() {
    let now = Instant::now();
    let mut ladder = Ladder::new(TIERS.to_vec());
    ladder.missed(now);
    ladder.answered(now + Duration::from_secs(60));
    assert_eq!(ladder.tier(), Tier::Relay);
    ladder.answered(now + Duration::from_secs(601));
    assert_eq!(ladder.tier(), Tier::Direct);
}

#[test]
fn an_answer_resets_the_backoff() {
    let now = Instant::now();
    let mut ladder = Ladder::new(vec![Tier::Direct]);
    for _ in 0..6 {
        ladder.missed(now);
    }
    ladder.answered(now);
    assert!(ladder.missed(now).as_millis() <= 1_200);
}

#[test]
fn a_new_edge_order_starts_from_its_first_tier() {
    let now = Instant::now();
    let mut ladder = Ladder::new(TIERS.to_vec());
    ladder.missed(now);
    ladder.follow(TIERS.to_vec());
    assert_eq!(ladder.tier(), Tier::Relay);
    ladder.follow(vec![Tier::Pro, Tier::Direct, Tier::Relay]);
    assert_eq!(ladder.tier(), Tier::Pro);
}

#[test]
fn spread_stays_within_a_fifth_of_the_pause() {
    assert_eq!(spread(1_000, 0), 800);
    assert_eq!(spread(1_000, 400), 1_200);
    assert_eq!(spread(1_000, 9_999), 1_200);
}

#[test]
fn backoff_grows_and_caps_at_a_minute() {
    assert!(backoff(1) <= Duration::from_millis(2_400));
    assert!(backoff(3) >= Duration::from_millis(6_400));
    assert!(backoff(40) <= Duration::from_secs(72));
    assert!(backoff(40) >= Duration::from_secs(48));
}

#[test]
fn the_expiry_timer_wakes_for_the_next_message_but_looks_again_soon() {
    assert_eq!(until(None, 0), Duration::from_secs(15));
    assert_eq!(until(Some(4_000), 1_000), Duration::from_secs(3));
    assert_eq!(until(Some(900_000), 0), Duration::from_secs(15));
    assert_eq!(until(Some(10), 1_000), Duration::from_secs(1));
}

#[test]
fn a_late_tick_means_the_machine_slept() {
    let mut watch = WakeWatch::new(0);
    assert!(!watch.woke(5_000));
    assert!(!watch.woke(16_000));
    assert!(watch.woke(16_000 + 90_000));
    assert!(!watch.woke(16_000 + 95_000));
    assert!(watch.woke(0));
}

#[test]
fn sync_results_map_to_the_next_move() {
    assert_eq!(next_of(&Ok(SyncOutcome::Changed)), Next::Go);
    assert_eq!(next_of(&Ok(SyncOutcome::Idle)), Next::Go);
    assert_eq!(next_of(&Ok(SyncOutcome::Reset)), Next::Go);
    assert_eq!(
        next_of(&Ok(SyncOutcome::Retry { after_ms: 4_000 })),
        Next::Wait(Duration::from_secs(4))
    );
    assert_eq!(next_of(&Err(ChatError::Offline)), Next::Fallback);
    assert_eq!(next_of(&Err(ChatError::Protocol)), Next::Fallback);
    assert_eq!(
        next_of(&Err(ChatError::Busy { retry_ms: 10 })),
        Next::Wait(Duration::from_secs(1))
    );
    assert_eq!(
        next_of(&Err(ChatError::ReloginRequired)),
        Next::Stop(ChatPhase::Unbound)
    );
    assert_eq!(
        next_of(&Err(ChatError::SessionBoundOther)),
        Next::Stop(ChatPhase::BoundOther)
    );
    assert_eq!(
        next_of(&Err(ChatError::DeviceRevoked)),
        Next::Stop(ChatPhase::Revoked)
    );
    assert_eq!(
        next_of(&Err(ChatError::Disabled)),
        Next::Stop(ChatPhase::Off)
    );
    assert_eq!(
        next_of(&Err(ChatError::Unimplemented)),
        Next::Wait(Duration::from_secs(60))
    );
}

#[test]
fn the_server_caps_how_long_a_poll_may_park() {
    let plan = PollPlan {
        tier: Tier::Direct,
        hold_ms: 20_000,
        delay_ms: 0,
    };
    assert_eq!(capped(plan, Some(12_000)).hold_ms, 12_000);
    assert_eq!(capped(plan, Some(30_000)).hold_ms, 20_000);
    assert_eq!(capped(plan, None).hold_ms, 20_000);
    assert_eq!(capped(plan, Some(0)).hold_ms, 20_000);
    let hidden = PollPlan {
        hold_ms: 0,
        delay_ms: 60_000,
        ..plan
    };
    assert_eq!(capped(hidden, Some(12_000)), hidden);
}

#[test]
fn sends_retry_unless_the_server_refused_for_good() {
    assert_eq!(retry_in(&ChatError::NotMember, 0), None);
    assert_eq!(retry_in(&ChatError::Muted { until: 1 }, 0), None);
    assert_eq!(retry_in(&ChatError::Invalid, 3), None);
    assert_eq!(
        retry_in(&ChatError::SlowMode { retry_ms: 5_000 }, 0),
        Some(Duration::from_secs(5))
    );
    assert_eq!(
        retry_in(&ChatError::EpochStale, 0),
        Some(Duration::from_secs(1))
    );
    assert_eq!(
        retry_in(&ChatError::Offline, 0),
        Some(Duration::from_secs(2))
    );
    assert_eq!(
        retry_in(&ChatError::Offline, 3),
        Some(Duration::from_secs(16))
    );
    assert_eq!(
        retry_in(&ChatError::Offline, 30),
        Some(Duration::from_secs(300))
    );
    assert_eq!(
        retry_in(&ChatError::SendPaused, 0),
        Some(Duration::from_secs(60))
    );
}

#[tokio::test]
async fn showing_the_window_kicks_the_loop_and_hiding_does_not() {
    let signals = Signals::new();
    assert!(!signals.visible());
    assert!(signals.set_visible(true));
    tokio::time::timeout(Duration::from_millis(100), signals.kicked())
        .await
        .expect("a show must kick");
    assert!(!signals.set_visible(true));
    assert!(signals.set_visible(false));
    assert!(
        tokio::time::timeout(Duration::from_millis(50), signals.kicked())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn a_restart_reaches_every_waiter() {
    let signals = std::sync::Arc::new(Signals::new());
    let generation = signals.generation();
    let waiter = {
        let signals = signals.clone();
        tokio::spawn(async move { signals.restarted(generation).await })
    };
    signals.restart();
    tokio::time::timeout(Duration::from_millis(200), waiter)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(signals.generation(), generation + 1);
}

#[test]
fn a_new_device_waits_three_days_for_old_history() {
    let device = DeviceView {
        id: Uuid::new_v4(),
        platform: "linux".into(),
        name: "Linux".into(),
        app_version: "8.6.0".into(),
        tier: "direct".into(),
        status: DeviceStatus::Active,
        created_at: 1_000,
        last_seen_at: 1_000,
        history_unlocked: false,
        current: true,
    };
    let gate = history_gate(&device);
    assert!(gate.locked);
    assert_eq!(gate.unlocks_at, Some(1_000 + 72 * 3_600_000));
    let unlocked = history_gate(&DeviceView {
        history_unlocked: true,
        ..device
    });
    assert!(!unlocked.locked);
    assert_eq!(unlocked.unlocks_at, None);
}

#[test]
fn server_config_maps_to_the_client_config() {
    let config = config_of(ConfigView {
        enabled: true,
        access: "all".into(),
        send_paused: false,
        hold_max_ms: 20_000,
        server_now: 42,
        incarnation: Uuid::nil(),
        ock_current: 1,
        limits: LimitsView {
            text_max: 4096,
            embeds_max: 3,
            group_members_max: 200,
            devices_max: 10,
            envelope_max: 65_536,
            disappearing_ttls: vec![3_600, 86_400, 604_800],
        },
    });
    assert_eq!(config.access, Access::All);
    assert_eq!(config.limits.group_members_max, 200);
    let beta = config_of(sample_config());
    assert_eq!(beta.access, Access::Beta);
    assert!(beta.send_paused);
    assert_eq!(beta.ock_current, 2);
}

#[test]
fn requests_rooms_and_muted_chats_stay_out_of_the_badge() {
    let open = summary();
    assert!(!quiet(&open, 1_000));
    assert!(quiet(
        &ConversationSummary {
            accepted: false,
            ..summary()
        },
        1_000
    ));
    assert!(quiet(
        &ConversationSummary {
            kind: ConversationKind::Room,
            ..summary()
        },
        1_000
    ));
    assert!(quiet(
        &ConversationSummary {
            muted_until: Some(2_000),
            ..summary()
        },
        1_000
    ));
    assert!(!quiet(
        &ConversationSummary {
            muted_until: Some(500),
            ..summary()
        },
        1_000
    ));
    assert!(quiet(
        &ConversationSummary {
            notify: NotifyLevel::None,
            ..summary()
        },
        1_000
    ));
}

fn sample_config() -> ConfigView {
    ConfigView {
        enabled: false,
        access: "beta".into(),
        send_paused: true,
        hold_max_ms: 15_000,
        server_now: 7,
        incarnation: Uuid::nil(),
        ock_current: 2,
        limits: LimitsView {
            text_max: 1,
            embeds_max: 1,
            group_members_max: 1,
            devices_max: 1,
            envelope_max: 1,
            disappearing_ttls: Vec::new(),
        },
    }
}

fn summary() -> ConversationSummary {
    ConversationSummary {
        id: Uuid::nil(),
        kind: ConversationKind::Direct,
        title: None,
        avatar_url: None,
        peer: None,
        role: MemberRole::Member,
        state: MemberState::Active,
        accepted: true,
        member_count: 2,
        last_seq: 5,
        last_at: 900,
        last_read_seq: 3,
        unread: 2,
        mentions: 0,
        preview: None,
        muted_until: None,
        notify: NotifyLevel::All,
        pinned_at: None,
        archived: false,
        ttl_s: None,
        room_code: None,
        history_locked: false,
        has_draft: false,
    }
}
