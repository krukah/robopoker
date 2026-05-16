//! NLHE public state: current-street history + available choices + pot geometry.
use super::*;
use rbp_gameplay::*;
use rbp_mccfr::*;

/// NLHE public state: subgame history, available actions, and pot geometry.
///
/// Stores the current-street action sequence, the available choices at this
/// decision point, and a discrete SPR bucket. Subgame and choices are encoded
/// as [`Path`] for compact 64-bit representation.
///
/// # Design
///
/// What's needed for info set indexing:
/// - `subgame`: Current-street action history (resets on each Draw)
/// - `choices`: Available actions at this decision point
/// - `geometry`: SPR bucket — distinguishes "300% pot on 6bb pot" from
///   "300% pot on 60bb pot" which the abstraction would otherwise collapse.
///
/// Street information comes from [`NlheSecret`] which embeds street in its encoding.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct NlhePublic {
    subgame: Path,
    choices: Path,
    geometry: Geometry,
}

impl NlhePublic {
    /// Creates a new public state from subgame history, available choices, and SPR bucket.
    pub fn new(subgame: Path, choices: Path, geometry: Geometry) -> Self {
        Self {
            subgame,
            choices,
            geometry,
        }
    }
    /// Current-street historical edges as a Path.
    pub fn subgame(&self) -> Path {
        self.subgame
    }
    /// Aggression (trailing aggressive actions) for bet sizing grid selection.
    pub fn aggression(&self) -> usize {
        self.subgame.aggression()
    }
    /// Pot-geometry bucket at this decision point.
    pub fn geometry(&self) -> Geometry {
        self.geometry
    }
}

impl CfrPublic for NlhePublic {
    type E = NlheEdge;
    type T = NlheTurn;

    fn choices(&self) -> impl Iterator<Item = Self::E> + use<> {
        self.choices.into_iter().map(NlheEdge::from)
    }

    fn subgame(&self) -> Vec<Self::E> {
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
        let public = NlhePublic::new(subgame, choices, super::Geometry::default());
        assert_eq!(public.aggression(), 2);
    }
    #[test]
    fn history_returns_subgame_edges() {
        let subgame = [Edge::Check, Edge::Raise(Odds::new(1, 2))]
            .into_iter()
            .collect::<Path>();
        let choices = Path::default();
        let public = NlhePublic::new(subgame, choices, super::Geometry::default());
        let history = public.subgame().into_iter().collect::<Vec<_>>();
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
        let public = NlhePublic::new(subgame, choices, super::Geometry::default());
        assert_eq!(public.choices().count(), 3);
    }
    #[test]
    fn path_returns_subgame() {
        let subgame = [Edge::Check, Edge::Check].into_iter().collect::<Path>();
        let choices = Path::default();
        let public = NlhePublic::new(subgame, choices, super::Geometry::default());
        assert_eq!(public.subgame(), subgame);
    }
}
