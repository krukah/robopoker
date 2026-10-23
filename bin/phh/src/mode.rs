use super::*;

#[derive(Subcommand, Debug)]
pub(crate) enum Mode {
    /// Replay every hand: does settlement reproduce the log, and does the
    /// engine agree about turn order and street boundaries?
    Conform,
    /// Measure observed bet sizes against the live `Size` grid.
    Calibrate {
        /// Restrict to one player's raises, e.g. `Pluribus`.
        #[arg(long)]
        player: Option<String>,
    },
    /// Score our blueprint's postflop choices against what was actually played,
    /// on the hands that reduce to our heads-up tree.
    Agree {
        /// Restrict to one player's decisions, e.g. `Pluribus`.
        #[arg(long)]
        player: Option<String>,
        /// Invert `--player`: score everyone else instead.
        #[arg(long)]
        others: bool,
        /// Backend holding the blueprint to score.
        #[arg(long, default_value = "https://robopoker.io", env = "PHH_API")]
        api: String,
    },
    /// Score what the logged decisions were *worth* against our blueprint, by
    /// playing each hand out behind both their action and ours.
    Duel(duel::Args),
}
