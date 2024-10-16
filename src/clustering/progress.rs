#[deprecated]
use std::time::Instant;

/// A struct to track and display progress of a long-running operation.
pub struct Progress {
    total: usize,
    check: usize,
    ticks: usize,
    begin: Instant,
    delta: Instant,
}
impl Progress {
    pub fn new(total: usize, n: usize) -> Self {
        let check = (total / n).min(1);
        let now = Instant::now();
        Self {
            total,
            check,
            ticks: 0,
            begin: now,
            delta: now,
        }
    }
    pub fn tick(&mut self) {
        self.ticks += 1;
        if self.ticks % self.check == 0 {
            let now = Instant::now();
            let total_t = now.duration_since(self.begin);
            let delta_t = now.duration_since(self.delta);
            self.delta = now;
            log::info!(
                "progress: {:8.0?} {:>10} {:6.2}%   mean {:6.0}   last {:6.0}",
                total_t,
                self.ticks,
                self.ticks as f32 / self.total as f32 * 100f32,
                self.ticks as f32 / total_t.as_secs_f32(),
                self.check as f32 / delta_t.as_secs_f32(),
            );
        }
    }
}
