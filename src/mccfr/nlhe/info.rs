use super::*;
use crate::cards::*;
use crate::gameplay::*;
use crate::mccfr::*;
use crate::*;
use std::hash::Hash;

/// can't tell whether default bucket makes sense.
/// it requires default absttraction, for one, which
/// i guess would just be P::00, but it can't be derived in [Abstraction]
/// because of the Middle bit hashing we do in [Abstraction]
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Info {
    history: Path,
    present: Abstraction,
    choices: Path,
}

impl Info {
    pub fn history(&self) -> &Path {
        &self.history
    }
    pub fn present(&self) -> &Abstraction {
        &self.present
    }
    pub fn choices(&self) -> &Path {
        &self.choices
    }
    pub fn edges(&self) -> Vec<Edge> {
        self.choices.into_iter().collect()
    }
}

/// Infoset construction - centralized logic for creating Info from various contexts
impl Info {
    /// Create Info from current game state and encoder lookup
    /// Replaces the seed logic in Encoder
    pub fn from_game(game: &Game, encoder: &NlheEncoder) -> Self {
        let iso = Isomorphism::from(game.sweat());
        let present = encoder.abstraction(&iso);
        let depth = 0;
        let history = Path::default();
        let choices = Self::futures(game, depth);
        Self::from((history, present, choices))
    }
    /// Create Info from Recall history with abstraction
    /// Used during inference when only abstraction is available
    pub fn from_path(recall: &Recall, present: Abstraction) -> Self {
        // @reversed-history
        let history = recall.path().rev().collect::<Path>();
        let depth = Self::depth(&history);
        let choices = Self::futures(&recall.head(), depth);
        Self::from((history, present, choices))
    }
    /// Create Info from tree traversal
    /// Replaces Encoder::info
    pub fn from_tree(
        tree: &Tree<Turn, Edge, Game, Info>,
        leaf: Branch<Edge, Game>,
        encoder: &NlheEncoder,
    ) -> Self {
        let (edge, ref game, head) = leaf;
        let iso = Isomorphism::from(game.sweat());
        let present = encoder.abstraction(&iso);
        let history = std::iter::once(edge)
            .chain(tree.at(head).map(|(_, e)| e))
            .take(crate::MAX_DEPTH_SUBGAME)
            .collect::<Path>()
            .rev()
            .collect::<Path>();
        let depth = Self::depth(&history);
        let choices = Self::futures(game, depth);
        Self::from((history, present, choices))
    }
}

/// Edge selection - street and depth aware logic for determining available actions
impl Info {
    /// Get available raise odds for given street and raise depth
    /// This is the SINGLE SOURCE OF TRUTH for which edges get trained
    /// It kinda, sorta, resembles geometric growth in min bet size
    pub fn raises(street: Street, depth: usize) -> &'static [Odds] {
        if depth > crate::MAX_RAISE_REPEATS {
            &[]
        } else {
            match street {
                Street::Pref => &Odds::PREF_RAISES,
                Street::Flop => &Odds::FLOP_RAISES,
                _ => match depth {
                    0 => &Odds::LATE_RAISES,
                    _ => &Odds::LAST_RAISES,
                },
            }
        }
    }
    /// Get all available edges for a game state at given depth
    /// Replaces Encoder::choices
    pub fn futures(game: &Game, depth: usize) -> Path {
        game.legal()
            .into_iter()
            .flat_map(|action| Self::unfold(game, depth, action))
            .collect()
    }
    /// Expand Action into Edge(s), using street-specific grids
    /// Replaces Encoder::unfold
    pub fn unfold(game: &Game, depth: usize, action: Action) -> Vec<Edge> {
        match action {
            Action::Raise(_) => Self::raises(game.street(), depth)
                .iter()
                .map(|&odds| Edge::from(odds))
                .filter(|edge| game.is_allowed(&Self::actionize(game, *edge)))
                .collect(),
            _ => vec![Edge::from(action)],
        }
    }
}

/// Conversions - street-aware edge/action mapping
impl Info {
    /// Convert Action to Edge using street-specific grid
    /// Guarantees result is in training set
    /// Replaces Game::edgify (with added depth parameter)
    pub fn edgify(game: &Game, action: Action, depth: usize) -> Edge {
        match action {
            Action::Fold => Edge::Fold,
            Action::Check => Edge::Check,
            Action::Draw(_) => Edge::Draw,
            Action::Call(_) => Edge::Call,
            Action::Blind(_) => Edge::Call,
            Action::Shove(_) => Edge::Shove,
            Action::Raise(x) => Edge::Raise(Odds::nearest((x, game.pot()), game.street(), depth)),
        }
    }
    /// Convert Edge to Action using game state
    /// Replaces Game::actionize
    pub fn actionize(game: &Game, edge: Edge) -> Action {
        match edge {
            Edge::Fold => game.folds(),
            Edge::Draw => game.reveal(),
            Edge::Call => game.calls(),
            Edge::Check => game.check(),
            Edge::Shove => game.shove(),
            Edge::Raise(odds) => {
                let min = game.to_raise();
                let max = game.to_shove();
                let pot = game.pot() as crate::Utility;
                let odd = crate::Utility::from(odds);
                match (pot * odd) as Chips {
                    bet if bet >= max => game.shove(),
                    bet if bet <= min => game.raise(),
                    bet => Action::Raise(bet),
                }
            }
        }
    }
}

/// Depth calculation - unifies Path::raises and Recall::depth
impl Info {
    /// Count aggressive actions in history for raise depth tracking
    /// Replaces both Path::raises() and Recall::depth()
    pub fn depth(history: &Path) -> usize {
        history
            .into_iter()
            .rev()
            .take_while(|e| e.is_choice())
            .filter(|e| e.is_aggro())
            .count()
    }
}

impl TreeInfo for Info {
    type E = Edge;
    type T = Turn;
    fn choices(&self) -> Vec<Self::E> {
        self.edges()
    }
}

impl From<(Path, Abstraction, Path)> for Info {
    fn from((history, present, futures): (Path, Abstraction, Path)) -> Self {
        Self {
            history,
            present,
            choices: futures,
        }
    }
}
impl From<Info> for (Path, Abstraction, Path) {
    fn from(info: Info) -> Self {
        (info.history, info.present, info.choices)
    }
}

impl std::fmt::Display for Info {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}>>{}<<{}", self.history, self.present, self.choices)
    }
}

impl Arbitrary for Info {
    fn random() -> Self {
        Self::from((Path::random(), Abstraction::random(), Path::random()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consistent_edge_collection_flop() {
        // Verify that training and inference use same grids on flop
        let game = Game::root();
        let game = game.apply(Action::Call(1));
        let game = game.apply(Action::Check);
        let flop = game.deck().deal(Street::Pref);
        let game = game.apply(Action::Draw(flop));
        assert_eq!(game.street(), Street::Flop);
        let depth = 0;
        let learned = Info::futures(&game, depth)
            .into_iter()
            .collect::<Vec<Edge>>();
        // Verify that all legal raises snap to trained edges
        for amount in game.to_raise()..game.to_shove() {
            let action = Action::Raise(amount);
            if game.is_allowed(&action) {
                let edge = Info::edgify(&game, action, depth);
                assert!(
                    learned.contains(&edge),
                    "Raise({}) -> {:?} not in trained edges for flop depth={}",
                    amount,
                    edge,
                    depth
                );
            }
        }
    }

    #[test]
    fn consistent_edge_collection_turn() {
        // Verify that training and inference use same grids on turn
        let game = Game::root();
        let game = game.apply(Action::Call(1));
        let game = game.apply(Action::Check);
        let flop = game.deck().deal(Street::Pref);
        let game = game.apply(Action::Draw(flop));
        let game = game.apply(Action::Check);
        let game = game.apply(Action::Check);
        let turn = game.deck().deal(Street::Flop);
        let game = game.apply(Action::Draw(turn));
        assert_eq!(game.street(), Street::Turn);
        let depth = 0;
        let trained = Info::futures(&game, depth)
            .into_iter()
            .collect::<Vec<Edge>>();
        // Verify that all legal raises snap to trained edges
        for amount in game.to_raise()..game.to_shove() {
            let action = Action::Raise(amount);
            let edge = Info::edgify(&game, action, depth);
            assert!(
                trained.contains(&edge),
                "Raise({}) -> {:?} not in trained edges for turn depth={}",
                amount,
                edge,
                depth
            );
        }
    }
    #[test]
    fn consistent_depth_calculation() {
        // Verify that depth calculation is consistent
        // Iterating backwards (reversed):
        // Shove (aggro, count=1), Raise(1,2) (aggro, count=2), Check (not aggro),
        // Draw (not choice, STOP)
        // So depth = 2 (only counts current street aggressive actions)
        assert_eq!(
            2,
            Info::depth(
                &[
                    Edge::Draw,
                    Edge::Raise(Odds(1, 1)),
                    Edge::Call,
                    Edge::Draw,
                    Edge::Check,
                    Edge::Raise(Odds(1, 2)),
                    Edge::Shove,
                ]
                .into_iter()
                .collect::<Path>(),
            )
        );
        // Test with no draw - should count all
        assert_eq!(
            &(Info::depth(
                &[
                    Edge::Raise(Odds(1, 1)),
                    Edge::Raise(Odds(1, 2)),
                    Edge::Shove,
                ]
                .into_iter()
                .collect::<Path>(),
            )),
            &3,
        );
        // Test with only passive actions
        assert_eq!(
            0,
            Info::depth(
                &[
                    Edge::Check, //
                    Edge::Call,
                    Edge::Check
                ]
                .into_iter()
                .collect::<Path>()
            ),
        );
    }

    #[test]
    fn roundtrip_string_serialization() {
        let info = Info::random();
        let (history, present, choices) = info.into();
        let history_i64: i64 = history.into();
        let present_i64: i64 = present.into();
        let choices_i64: i64 = choices.into();
        let deserialized = Info::from((history_i64.into(), present_i64.into(), choices_i64.into()));
        assert_eq!(info, deserialized);
    }

    #[test]
    fn roundtrip_edgify_actionize() {
        // Verify edgify → actionize → edgify is stable
        let depth = 0;
        let game = Game::root();
        for a1 in game.legal() {
            let e1 = Info::edgify(&game, a1, depth);
            let a2 = Info::actionize(&game, e1);
            let e2 = Info::edgify(&game, a2, depth);
            assert_eq!(
                e1, e2,
                "Roundtrip failed: {:?} -> {:?} -> {:?} -> {:?}",
                a1, e1, a2, e2
            );
        }
    }

    #[test]
    fn raises_grid_selection() {
        // Verify correct grid selected for each street/depth combination
        assert_eq!(Info::raises(Street::Pref, 0).len(), 10);
        assert_eq!(Info::raises(Street::Flop, 0).len(), 5);
        assert_eq!(Info::raises(Street::Turn, 0).len(), 2);
        assert_eq!(Info::raises(Street::Turn, 1).len(), 1);
        assert_eq!(Info::raises(Street::Rive, 0).len(), 2);
        assert_eq!(Info::raises(Street::Rive, 1).len(), 1);
        // Verify MAX_RAISE_REPEATS cutoff
        let deep = crate::MAX_RAISE_REPEATS + 1;
        assert_eq!(Info::raises(Street::Pref, deep).len(), 0);
    }

    #[test]
    fn unfold_respects_street() {
        let game = Game::root();
        // Preflop should expand to 10 raises
        let pref_raises = Info::unfold(&game, 0, Action::Raise(game.to_raise()));
        assert_eq!(pref_raises.len(), 10);
        // Move to flop
        let game = game.apply(Action::Call(1));
        let game = game.apply(Action::Check);
        let flop = game.deck().deal(Street::Pref);
        let game = game.apply(Action::Draw(flop));
        // Flop should expand to 5 raises
        let flop_raises = Info::unfold(&game, 0, Action::Raise(game.to_raise()));
        assert_eq!(flop_raises.len(), 5);
    }

    // All legal raise amounts should map to trained edges
    #[test]
    fn raises_into_edges() {
        let game = Game::root();
        let depth = 0;
        let trained = Info::futures(&game, depth)
            .into_iter()
            .collect::<std::collections::HashSet<Edge>>();
        for amount in game.to_raise()..game.to_shove() {
            let raise = Action::Raise(amount);
            let edge = Info::edgify(&game, raise, depth);
            assert!(trained.contains(&edge));
        }
    }
}
