//! NLHE public state: current-street history + available choices.
use super::*;
use rbp_gameplay::*;
use rbp_mccfr::*;

/// NLHE public state: subgame history and available actions.
///
/// Stores the current-street action sequence and the available choices at this
/// decision point. Both are encoded as [`Path`] for compact 64-bit representation.
///
/// # Design
///
/// Only what's needed for info set indexing:
/// - `subgame`: Current-street action history (resets on each Draw)
/// - `choices`: Available actions at this decision point
///
/// Street information comes from [`NlheSecret`] which embeds street in its encoding.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct NlhePublic {
    subgame: Path,
    choices: Path,
}

impl NlhePublic {
    /// Creates a new public state from subgame history and available choices.
    pub fn new(subgame: Path, choices: Path) -> Self {
        Self { subgame, choices }
    }
    /// Current-street historical edges as a Path.
    pub fn subgame(&self) -> Path {
        self.subgame
    }
    /// Aggression (trailing aggressive actions) for bet sizing grid selection.
    pub fn aggression(&self) -> usize {
        self.subgame.aggression()
    }
}

impl CfrPublic for NlhePublic {
    type E = NlheEdge;
    type T = NlheTurn;
    fn choices(&self) -> Vec<Self::E> {
        self.choices.into_iter().map(NlheEdge::from).collect()
    }
    fn history(&self) -> Vec<Self::E> {
        self.subgame.into_iter().map(NlheEdge::from).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aggression_counts_from_path() {
        let subgame = [
            Edge::Check,
            Edge::Raise(Odds::new(1, 2)),
            Edge::Raise(Odds::new(1, 1)),
        ]
        .into_iter()
        .collect::<Path>();
        let choices = Path::default();
        let public = NlhePublic::new(subgame, choices);
        assert_eq!(public.aggression(), 2);
    }
    #[test]
    fn history_returns_subgame_edges() {
        let subgame = [Edge::Check, Edge::Raise(Odds::new(1, 2))]
            .into_iter()
            .collect::<Path>();
        let choices = Path::default();
        let public = NlhePublic::new(subgame, choices);
        let history = public.history();
        assert_eq!(history.len(), 2);
        assert_eq!(Edge::from(history[0]), Edge::Check);
        assert_eq!(Edge::from(history[1]), Edge::Raise(Odds::new(1, 2)));
    }
    #[test]
    fn choices_returns_stored_choices() {
        let subgame = Path::default();
        let choices = [Edge::Fold, Edge::Call, Edge::Shove]
            .into_iter()
            .collect::<Path>();
        let public = NlhePublic::new(subgame, choices);
        let available = public.choices();
        assert_eq!(available.len(), 3);
    }
    #[test]
    fn path_returns_subgame() {
        let subgame = [Edge::Check, Edge::Check].into_iter().collect::<Path>();
        let choices = Path::default();
        let public = NlhePublic::new(subgame, choices);
        assert_eq!(public.subgame(), subgame);
    }
}
