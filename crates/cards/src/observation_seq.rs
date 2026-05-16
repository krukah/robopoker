use super::*;

/// An [`Observation`] enriched with deal-time board ordering.
///
/// Wraps an `Observation` (unordered card sets) together with a [`HandSeq`]
/// for the board's deal order. Derefs to `Observation`, so all existing
/// methods (`pocket`, `public`, `street`, `equity`, `opponents`, `children`, …)
/// work transparently.
///
/// The ordering applies only to the public/board cards. Hole card ordering
/// is not tracked.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct ObservationSeq {
    observation: Observation,
    perm: Perm,
}

impl ObservationSeq {
    /// The board as an ordered [`HandSeq`].
    pub fn board(&self) -> HandSeq {
        HandSeq::from((*self.observation.public(), self.perm))
    }
    /// The board permutation (deal order of community cards).
    pub fn perm(&self) -> Perm {
        self.perm
    }
}

impl std::ops::Deref for ObservationSeq {
    type Target = Observation;

    fn deref(&self) -> &Observation {
        &self.observation
    }
}

/// Upgrade an `Observation` with identity (canonical) board ordering.
impl From<Observation> for ObservationSeq {
    fn from(observation: Observation) -> Self {
        Self {
            observation,
            perm: Perm::identity(),
        }
    }
}

/// Downgrade to `Observation`, discarding deal order.
impl From<ObservationSeq> for Observation {
    fn from(seq: ObservationSeq) -> Self {
        seq.observation
    }
}

/// Construct from an explicit `(Observation, Perm)` pair.
impl From<(Observation, Perm)> for ObservationSeq {
    fn from((observation, perm): (Observation, Perm)) -> Self {
        Self { observation, perm }
    }
}

/// Construct from hole cards and an ordered board slice.
/// The `Perm` is inferred from the slice ordering; the
/// `Observation` is assembled from the underlying card sets.
impl From<(Hole, &[Card])> for ObservationSeq {
    fn from((hole, board): (Hole, &[Card])) -> Self {
        Self {
            observation: Observation::from((
                Hand::from(hole),
                board.iter().copied().collect::<Hand>(),
            )),
            perm: Perm::of(board),
        }
    }
}

impl std::fmt::Display for ObservationSeq {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.observation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deref_delegates() {
        let obs = Observation::try_from("AcKs~QhJhTd").unwrap();
        let seq = ObservationSeq::from(obs);
        assert_eq!(seq.pocket(), obs.pocket());
        assert_eq!(seq.public(), obs.public());
        assert_eq!(seq.street(), obs.street());
    }
    #[test]
    fn identity_board_is_canonical() {
        let obs = Observation::try_from("AcKs~QhJhTd").unwrap();
        let seq = ObservationSeq::from(obs);
        assert_eq!(
            seq.board().cards().collect::<Vec<_>>(),
            Vec::<Card>::from(*obs.public())
        );
    }
    #[test]
    fn from_ordered_board() {
        let hole = Hole::try_from("AcKs").unwrap();
        let board = Card::parse("Td Jh Qh").unwrap();
        let seq = ObservationSeq::from((hole, board.as_slice()));
        assert_eq!(seq.board().cards().collect::<Vec<_>>(), board);
        assert_eq!(seq.street(), Street::Flop);
    }
    #[test]
    fn round_trip_observation() {
        let obs = Observation::try_from("AcKs~QhJhTd").unwrap();
        let perm = Perm::from(3u8);
        let seq = ObservationSeq::from((obs, perm));
        assert_eq!(Observation::from(seq), obs);
        assert_eq!(seq.perm(), perm);
    }
    #[test]
    fn board_preserves_hand() {
        let hole = Hole::try_from("7h2c").unwrap();
        let board = Card::parse("Qs As Td Jh 5d").unwrap();
        let seq = ObservationSeq::from((hole, board.as_slice()));
        assert_eq!(Hand::from(seq.board()), *seq.public());
    }
    #[test]
    fn preflop_empty_board() {
        let obs = Observation::try_from("AcKs").unwrap();
        let seq = ObservationSeq::from(obs);
        assert_eq!(seq.board().size(), 0);
        assert_eq!(seq.perm(), Perm::identity());
    }
}
