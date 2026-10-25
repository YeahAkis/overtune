pub const PLAY_THRESHOLD_MS: u64 = 30_000;

pub fn play_threshold_ms(duration_ms: Option<u64>) -> u64 {
    match duration_ms {
        Some(d) => PLAY_THRESHOLD_MS.min(d / 2),
        None => PLAY_THRESHOLD_MS,
    }
}
