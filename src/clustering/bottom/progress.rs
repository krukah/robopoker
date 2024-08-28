use tokio::time::Instant;

/// A struct to track and display progress of a long-running operation.
pub struct Progress {
    total: usize,
    ticks: usize,
    begin: Instant,
    delta: Instant,
}
impl Progress {
    const CHECKPOINT: usize = 1_000_000;
    pub fn new(total: usize) -> Self {
        let now = Instant::now();
        Self {
            total,
            ticks: 0,
            begin: now,
            delta: now,
        }
    }
    pub fn tick(&mut self) {
        self.ticks += 1;
        if self.ticks % Self::CHECKPOINT == 0 {
            use std::io::Write;
            let now = Instant::now();
            let total_t = now.duration_since(self.begin);
            let delta_t = now.duration_since(self.delta);
            self.delta = now;
            print!("\r");
            print!("\x1B[K");
            print!(
                "{:8.0?} {:>10} {:6.2}%   mean {:6.0}   last {:6.0}",
                total_t,
                self.ticks,
                self.ticks as f32 / self.total as f32 * 100f32,
                self.ticks as f32 / total_t.as_secs_f32(),
                Self::CHECKPOINT as f32 / delta_t.as_secs_f32(),
            );
            std::io::stdout().flush().unwrap();
        }
    }
}
