use deuce::Observation;
use pokerkit::*;

/// Helpers shared by the solve paths.
pub(crate) struct Solving;

impl Solving {
    /// Normalize a `(observation, reach)` stream to sum to 1, leaving an
    /// all-zero stream untouched.
    pub(crate) fn normalize(raws: Vec<(Observation, Probability)>) -> Vec<(Observation, Probability)> {
        let total = raws.iter().map(|(_, r)| *r).sum::<Probability>();
        match total {
            0.0 => raws,
            mass => raws.into_iter().map(|(obs, r)| (obs, r / mass)).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use deuce::Street;
    use kicker::*;

    impl Solving {
        /// Heads-up preflop recall from `pov`, having played `actions`.
        fn recall(pov: usize, actions: Vec<Action>) -> Witness {
            Witness::from((Turn::Choice(pov), Observation::from(Street::Pref), actions))
        }
    }

    /// The whole reason one range panel can look smooth while the other looks spiky:
    /// [`Flagship::reach`] multiplies over the *subject's* own decisions, and an empty product
    #[test]
    fn only_the_seat_that_acted_has_a_posterior() {
        let opened = Solving::recall(0, vec![Action::Call(1)]);
        let solver = Flagship::default();
        assert_eq!(solver.decisions(&opened, NlheTurn::narrator(&opened)), 1);
        assert_eq!(solver.decisions(&opened, NlheTurn::opponent(&opened)), 0);
        let ref facing = opened.with_pov(Turn::Choice(1));
        assert_eq!(solver.decisions(facing, NlheTurn::narrator(facing)), 0);
        assert_eq!(solver.decisions(facing, NlheTurn::opponent(facing)), 1);
    }

    /// Decision counts are a property of the public action history, not of the POV the client
    /// happens to be sitting in — so the two endpoints always report the same pair of numbers
    #[test]
    fn decision_counts_are_pov_symmetric() {
        let ref acted = Solving::recall(0, vec![Action::Call(1), Action::Check]);
        let ref flipped = acted.with_pov(Turn::Choice(1));
        let solver = Flagship::default();
        assert_eq!(
            solver.decisions(acted, NlheTurn::narrator(acted)),
            solver.decisions(flipped, NlheTurn::opponent(flipped))
        );
        assert_eq!(
            solver.decisions(acted, NlheTurn::opponent(acted)),
            solver.decisions(flipped, NlheTurn::narrator(flipped))
        );
    }

    /// Nobody has acted, so neither panel is showing anything the blueprint
    /// learned — both are the uniform prior over `deck − board`.
    #[test]
    fn root_node_conditions_on_nothing() {
        let ref root = Solving::recall(0, vec![]);
        let solver = Flagship::default();
        assert_eq!(solver.decisions(root, NlheTurn::narrator(root)), 0);
        assert_eq!(solver.decisions(root, NlheTurn::opponent(root)), 0);
    }
}
