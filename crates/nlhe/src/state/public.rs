//! NLHE public state: current-street history + available choices.
use super::*;
use kicker::*;
use mccfr::*;

/// NLHE public state: current-street action history (reset on every `Draw`) and the available
/// choices, both packed as [`Path`] into 64 bits, plus `field` — the button-anchored live set
/// and acting position, which is the cross-street state `subgame` drops at each `Draw`. `field`
/// carries no information at N=2 but distinguishes multiway spots at N>2.
///
/// Street is not stored here; it is embedded in [`NlheSecret`]'s encoding.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct NlhePublic {
    subgame: Subgame,
    choices: Path,
    field: Field,
}

impl NlhePublic {
    pub fn new(subgame: Subgame, choices: Path, field: Field) -> Self {
        Self {
            subgame,
            choices,
            field,
        }
    }
    /// The public half at `game`, given the path's edges nearest first: the current street's
    /// choice edges and the menu they leave.
    pub fn at<P>(game: &NlheGame, past: P) -> Self
    where
        P: IntoIterator<Item = NlheEdge>,
    {
        let subgame = past
            .into_iter()
            .map(Edge::from)
            .take_while(Edge::is_choice)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Subgame>();
        Self::new(subgame, game.as_ref().choices(subgame.aggression()), game.field())
    }
    /// Current-street historical edges as a Path.
    pub fn subgame(&self) -> Subgame {
        self.subgame
    }
    /// Aggression (trailing aggressive actions) for bet sizing grid selection.
    pub fn aggression(&self) -> usize {
        self.subgame.aggression()
    }
    /// Button-anchored live set + acting position.
    pub fn field(&self) -> Field {
        self.field
    }
    /// The available choices, packed.
    fn options(&self) -> Path {
        self.choices
    }
}

impl CfrPublic for NlhePublic {
    type G = NlheGame;

    fn menu(&self) -> impl Iterator<Item = NlheEdge> + use<> {
        self.options().into_iter().map(NlheEdge::from)
    }

    fn count(&self) -> usize {
        self.options().length()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aggression_counts_from_path() {
        let subgame = [Edge::Check, Edge::Raise(Odds::new(1, 2)), Edge::Raise(Odds::new(1, 1))]
            .into_iter()
            .collect::<Subgame>();
        let choices = Path::default();
        let public = NlhePublic::new(subgame, choices, Field::default());
        assert_eq!(public.aggression(), 2);
    }
    #[test]
    fn history_returns_subgame_edges() {
        let subgame = [Edge::Check, Edge::Raise(Odds::new(1, 2))]
            .into_iter()
            .collect::<Subgame>();
        let choices = Path::default();
        let public = NlhePublic::new(subgame, choices, Field::default());
        let history = public.subgame().into_iter().collect::<Vec<_>>();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0], Edge::Check);
        assert_eq!(history[1], Edge::Raise(Odds::new(1, 2)));
    }
    #[test]
    fn choices_returns_stored_choices() {
        let subgame = Subgame::default();
        let choices = [Edge::Fold, Edge::Call, Edge::Shove].into_iter().collect::<Path>();
        let public = NlhePublic::new(subgame, choices, Field::default());
        assert_eq!(public.menu().count(), 3);
    }
    #[test]
    fn path_returns_subgame() {
        let subgame = [Edge::Check, Edge::Check].into_iter().collect::<Subgame>();
        let choices = Path::default();
        let public = NlhePublic::new(subgame, choices, Field::default());
        assert_eq!(public.subgame(), subgame);
    }
}
