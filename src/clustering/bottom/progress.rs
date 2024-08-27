use tokio::time::Instant;

/// A struct to track and display progress of a long-running operation.
pub struct Progress {
    begin: Instant,
    delta: Instant,
    complete: u32,
}
impl Progress {
    const CHECKPOINT: usize = 10_000;
    const TOTAL: usize = 2_809_475_760;

    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            begin: now,
            delta: now,
            complete: 0,
        }
    }
    pub fn tick(&mut self) {
        self.complete += 1;
        if self.complete % Self::CHECKPOINT as u32 == 0 {
            use std::io::Write;
            let now = Instant::now();
            let total_t = now.duration_since(self.begin);
            let delta_t = now.duration_since(self.delta);
            self.delta = now;
            print!("\x1B[4A"); // Move cursor up 4 lines (for 4 lines of output)
            print!("\x1B[0J"); // Clear from cursor to end of screen
            println!("Elapsed: {:.0?}", total_t);
            #[rustfmt::skip]
            println!("Mean Freq:{:>10.0}", self.complete as f32 / total_t.as_secs_f32());
            #[rustfmt::skip]
            println!("Last Freq:{:>10.0}", Self::CHECKPOINT as f32 / delta_t.as_secs_f32());
            #[rustfmt::skip]
            println!("{:10}{:>10.1}%", self.complete, (self.complete as f32 / Self::TOTAL as f32) * 100.0);
            std::io::stdout().flush().unwrap();
        }
    }
}
