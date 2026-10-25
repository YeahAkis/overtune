use std::time::{Duration, Instant};

#[derive(Debug, Clone, Default)]
pub struct ListenTracker {
    segment: Option<Instant>,
    total: Duration,
}

impl ListenTracker {
    pub fn start(&mut self, at: Instant) {
        self.total = Duration::ZERO;
        self.segment = Some(at);
    }

    pub fn pause(&mut self, at: Instant) {
        if let Some(start) = self.segment.take() {
            self.total += at.saturating_duration_since(start);
        }
    }

    pub fn resume(&mut self, at: Instant) {
        if self.segment.is_none() {
            self.segment = Some(at);
        }
    }

    pub fn seek(&mut self, at: Instant) {
        if self.segment.is_some() {
            self.pause(at);
            self.segment = Some(at);
        }
    }

    pub fn listened_ms(&self, at: Instant) -> u64 {
        let open = self
            .segment
            .map(|s| at.saturating_duration_since(s))
            .unwrap_or_default();
        (self.total + open).as_millis() as u64
    }

    pub fn finish(&mut self, at: Instant) -> u64 {
        let ms = self.listened_ms(at);
        self.segment = None;
        self.total = Duration::ZERO;
        ms
    }
}
