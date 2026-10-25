use crate::*;
use std::time::Duration;

/// How long each paced beat of a hand lasts, in milliseconds.
///
/// One [`Tempo`] scales the set; `decision` is unscaled, being the
/// player's clock rather than the table's. It sits above both the engine
/// that paces by it and the client that draws it: a duration those two
/// disagree about is a ring counting down to a deadline already passed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Pacing {
    hole: u64,
    board: u64,
    reveal: u64,
    results: u64,
    decision: u64,
    think: (u64, u64),
}

impl From<Tempo> for Pacing {
    fn from(tempo: Tempo) -> Self {
        Self {
            hole: tempo.scale(PACE_DEAL_HOLE),
            board: tempo.scale(PACE_DEAL_BOARD),
            reveal: tempo.scale(PACE_SHOWDOWN),
            results: tempo.scale(PACE_RESULTS),
            decision: PACE_DECISION,
            think: (tempo.scale(PACE_BOT_THINK_MIN), tempo.scale(PACE_BOT_THINK_MAX)),
        }
    }
}

impl Default for Pacing {
    fn default() -> Self {
        Self::from(Tempo::default())
    }
}

impl Pacing {
    fn hole_ms(self) -> u64 {
        self.hole
    }

    fn board_ms(self) -> u64 {
        self.board
    }

    fn reveal_ms(self) -> u64 {
        self.reveal
    }

    fn results_ms(self) -> u64 {
        self.results
    }

    fn decision_ms(self) -> u64 {
        self.decision
    }

    fn span(self) -> (u64, u64) {
        self.think
    }

    pub fn hole(self) -> Duration {
        Duration::from_millis(self.hole_ms())
    }

    pub fn board(self) -> Duration {
        Duration::from_millis(self.board_ms())
    }

    pub fn reveal(self) -> Duration {
        Duration::from_millis(self.reveal_ms())
    }

    pub fn results(self) -> Duration {
        Duration::from_millis(self.results_ms())
    }

    pub fn decision(self) -> Duration {
        Duration::from_millis(self.decision_ms())
    }

    /// How long a bot appears to think, drawn fresh for each decision.
    ///
    /// Uniform over the range and independent of the decision: a constant
    /// reads as a metronome, and anything correlated with the hand is a
    /// tell.
    pub fn think(self) -> Duration {
        Duration::from_millis(rand::random_range(self.span().0..=self.span().1))
    }

    /// A beat as whole seconds, rounded up, for a countdown ring. Reading
    /// zero while the table is still waiting is worse than reading one.
    pub fn seconds(beat: Duration) -> u8 {
        beat.as_millis().div_ceil(1000) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_pacing_is_the_table_tempo() {
        assert_eq!(Pacing::default(), Pacing::from(Tempo::Table));
        assert_eq!(Pacing::default().results(), Duration::from_millis(PACE_RESULTS));
    }

    #[test]
    fn tempo_scales_the_beats_but_not_the_players_clock() {
        let brisk = Pacing::from(Tempo::Brisk);
        let slow = Pacing::from(Tempo::Slow);
        assert!(brisk.results() < slow.results());
        assert!(brisk.board() < slow.board());
        assert_eq!(brisk.decision(), slow.decision());
    }

    #[test]
    fn think_time_stays_inside_its_range_and_varies() {
        let pacing = Pacing::default();
        let draws = (0..64).map(|_| pacing.think()).collect::<Vec<_>>();
        let lo = Duration::from_millis(PACE_BOT_THINK_MIN);
        let hi = Duration::from_millis(PACE_BOT_THINK_MAX);
        assert!(draws.iter().all(|d| *d >= lo && *d <= hi));
        assert!(draws.iter().collect::<std::collections::HashSet<_>>().len() > 1);
    }

    /// The whole reason this type is on the wire: a client that guessed the
    /// tempo would read the default's ring against another tempo's clock.
    #[test]
    fn a_ring_reads_the_pacing_it_was_given() {
        assert_eq!(Pacing::seconds(Pacing::default().decision()), 10);
        assert_eq!(Pacing::seconds(Pacing::default().results()), 3);
        assert_eq!(Pacing::seconds(Pacing::from(Tempo::Slow).results()), 5);
        assert_eq!(Pacing::seconds(Pacing::from(Tempo::Brisk).results()), 2);
    }

    #[test]
    fn a_partial_second_still_shows_a_second() {
        assert_eq!(Pacing::seconds(Duration::from_millis(1)), 1);
        assert_eq!(Pacing::seconds(Duration::from_millis(1001)), 2);
        assert_eq!(Pacing::seconds(Duration::ZERO), 0);
    }

    #[test]
    fn pacing_survives_the_wire() {
        let sent = Pacing::from(Tempo::Slow);
        let json = serde_json::to_string(&sent).expect("serialise");
        assert_eq!(serde_json::from_str::<Pacing>(&json).expect("deserialise"), sent);
    }
}
