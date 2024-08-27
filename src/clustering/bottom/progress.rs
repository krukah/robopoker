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
            begin: now,
            delta: now,
            ticks: 0,
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
            print!("\r"); // Move cursor to the beginning of the line
            print!("\x1B[K"); // Clear the line
            print!(
                "Elapsed: {:8.0?} | Mean Freq: {:10.0} | Last Freq: {:10.0} | Progress: {:6.2}% {:>10}",
                total_t,
                self.ticks as f32 / total_t.as_secs_f32(),
                Self::CHECKPOINT as f32 / delta_t.as_secs_f32(),
                (self.ticks as f32 / self.total as f32) * 100.0,
                self.ticks
            );
            std::io::stdout().flush().unwrap();
        }
    }
}
