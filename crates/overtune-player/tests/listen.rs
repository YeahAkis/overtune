use std::time::{Duration, Instant};

use overtune_player::listen::ListenTracker;

fn s(n: u64) -> Duration {
    Duration::from_secs(n)
}

#[test]
fn plain_listen() {
    let t0 = Instant::now();
    let mut t = ListenTracker::default();
    t.start(t0);
    assert_eq!(t.finish(t0 + s(42)), 42_000);
}

#[test]
fn seek_jump_is_not_counted() {
    let t0 = Instant::now();
    let mut t = ListenTracker::default();
    t.start(t0);
    t.seek(t0 + s(10));
    assert_eq!(t.finish(t0 + s(15)), 15_000);
}

#[test]
fn pause_gaps_do_not_count() {
    let t0 = Instant::now();
    let mut t = ListenTracker::default();
    t.start(t0);
    t.pause(t0 + s(5));
    t.resume(t0 + s(100));
    assert_eq!(t.finish(t0 + s(110)), 15_000);
}

#[test]
fn scrubbing_counts_only_wall_time() {
    let t0 = Instant::now();
    let mut t = ListenTracker::default();
    t.start(t0);
    for i in 1..=20 {
        t.seek(t0 + Duration::from_millis(100 * i));
    }
    assert_eq!(t.finish(t0 + s(2)), 2_000);
}

#[test]
fn seek_while_paused_stays_paused() {
    let t0 = Instant::now();
    let mut t = ListenTracker::default();
    t.start(t0);
    t.pause(t0 + s(3));
    t.seek(t0 + s(50));
    assert_eq!(t.finish(t0 + s(60)), 3_000);
}

#[test]
fn finish_resets() {
    let t0 = Instant::now();
    let mut t = ListenTracker::default();
    t.start(t0);
    t.finish(t0 + s(5));
    assert_eq!(t.finish(t0 + s(9)), 0);
    assert_eq!(t.listened_ms(t0 + s(9)), 0);
}

#[test]
fn listened_so_far_while_playing() {
    let t0 = Instant::now();
    let mut t = ListenTracker::default();
    t.start(t0);
    t.pause(t0 + s(4));
    t.resume(t0 + s(10));
    assert_eq!(t.listened_ms(t0 + s(12)), 6_000);
}
