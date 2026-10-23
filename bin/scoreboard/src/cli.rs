use super::*;

#[derive(Parser, Debug)]
#[command(name = "scoreboard", about = "Rank trained fingerprints by benchmark bb/100 (litmus alongside).")]
pub(crate) struct Cli {
    /// Emit JSON instead of a formatted table.
    #[arg(long)]
    pub(crate) json: bool,
}

impl Cli {
    pub(crate) fn json(&self) -> bool {
        self.json
    }
}
