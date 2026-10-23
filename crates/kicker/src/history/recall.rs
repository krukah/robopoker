//! Shared replay interface for [`Perfect`] and [`Witness`], the complete-info
//! and hero-only views of a game history.
//!
//! Blinds are constant and deterministic, so they are NOT stored in `actions()`:
//! `root()` returns a POST-blind state, and `complete()` re-attaches them for
//! display.
use super::*;
use deuce::Card;
use pokerkit::Translated;

/// A game history that can be replayed from a root state.
pub trait Recall {
    /// The starting game state for replaying actions (POST-blind).
    fn root(&self) -> Game;

    /// The action sequence from root to current state (excludes blinds).
    fn actions(&self) -> &[Action];

    /// Complete action sequence including blinds (for client display).
    fn complete(&self) -> Vec<Action> {
        Game::blinds()
            .into_iter()
            .chain(self.actions().iter().copied())
            .collect()
    }

    /// Current game state (replay actions from root).
    fn head(&self) -> Game {
        self.actions()
            .iter()
            .copied()
            .fold(self.root(), |mut g, a| g.consume(a))
    }

    /// Sequence of game states from root to head.
    fn states(&self) -> Vec<Game> {
        let root = self.root();
        let acts = self
            .actions()
            .iter()
            .copied()
            .scan(root, |g, a| Some(g.consume(a)))
            .collect::<Vec<Game>>();
        std::iter::once(root).chain(acts).collect()
    }

    /// Current aggression (trailing aggressive actions on current street).
    fn aggression(&self) -> usize {
        self.actions()
            .iter()
            .copied()
            .rev()
            .take_while(Action::is_choice)
            .filter(Action::is_aggro)
            .count()
    }

    /// Full edge history (all streets), via the global [`pokerkit::Translation::current`].
    ///
    /// Panics under [`pokerkit::Translation::Exact`], which declines to snap an off-grid raise;
    /// walk [`Self::typed_history`] there instead.
    fn history(&self) -> Vec<Edge> {
        let translation = pokerkit::Translation::current();
        let ref mut rng = rand::rng();
        self.states()
            .into_iter()
            .zip(self.actions().iter())
            .scan(Subgame::default(), |past, (game, action)| {
                let edge = game.translate(*action, past.aggression(), &translation, rng).snapped();
                *past = past.flow(edge);
                Some(edge)
            })
            .collect()
    }

    /// A [`Stride`] at every point of this recall's own walk. The canonical, deterministic
    /// counterpart to [`Self::history`]: edges come from [`Game::edgify`] rather than the
    /// process-global translation policy, so the walk neither consumes an RNG nor varies with
    /// `--translation`.
    fn walk(&self) -> Vec<Stride> {
        self.states()
            .into_iter()
            .zip(self.actions().iter())
            .scan(Subgame::default(), |past, (game, action)| {
                let edge = game.edgify(*action, past.aggression());
                *past = past.flow(edge);
                Some(Stride::from((game.turn(), game, edge)))
            })
            .collect()
    }

    /// Full typed history, preserving off-tree actions.
    ///
    /// Like [`Self::history`] but does not panic on the [`Translated::Free`] arm: off-grid
    /// raises (emitted only under [`pokerkit::Translation::Exact`]) are handed back verbatim as
    /// `Translated::Free(Action::Raise(_))`. A nesting player walks this to locate the off-tree
    /// entry and re-solve an augmented subgame — see `docs/active/off-tree-nesting.md`.
    fn typed_history(&self) -> Vec<Translated<Edge, Action>> {
        let translation = pokerkit::Translation::current();
        let ref mut rng = rand::rng();
        self.states()
            .into_iter()
            .zip(self.actions().iter())
            .scan(Subgame::default(), |past, (game, action)| {
                let step = game.translate(*action, past.aggression(), &translation, rng);
                let edge = match step {
                    Translated::Snap(edge) => edge,
                    Translated::Free(_) => game.edgify(*action, past.aggression()),
                };
                *past = past.flow(edge);
                Some(step)
            })
            .collect()
    }

    /// Current street edges only (trailing choice edges before any Draw). Width-pinned by
    /// player count: heads-up `Subgame = Path<1>`.
    ///
    /// Saturating: a live opponent's marathon min-raise war can exceed the packed capacity that
    /// training's raise grid never reaches. The key keeps its street prefix and the lookup
    /// degrades gracefully; see [`Path::saturated`].
    fn subgame(&self) -> Subgame {
        Subgame::saturated(
            self.history()
                .into_iter()
                .rev()
                .take_while(Edge::is_choice)
                .collect::<Vec<_>>()
                .into_iter()
                .rev(),
        )
    }

    /// Available actions at current state.
    fn choices(&self) -> Path {
        self.head().choices(self.aggression())
    }
    /// Community cards in deal order (flop first, then turn, then river).
    fn dealt(&self) -> Vec<Card> {
        self.actions()
            .iter()
            .filter_map(|a| match a {
                Action::Draw(h) => Some(Vec::<Card>::from(*h)),
                _ => None,
            })
            .flatten()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use deuce::Street;

    /// The board must survive the walk.
    #[test]
    fn walk_preserves_the_dealt_board() {
        let recall = Witness::initial(Game::root().dealer())
            .push(Action::Call(1))
            .push(Action::Check);
        let flop = recall.head().deck().deal(Street::Pref, &mut rand::rng());
        let recall = recall.push(Action::Draw(flop)).push(Action::Check);
        let after = recall.walk().last().expect("nonempty walk").game();
        assert_eq!(
            after.board_cards().into_iter().collect::<deuce::Hand>(),
            flop,
            "walk must carry the actually-dealt flop, not a re-deal"
        );
        assert_eq!(recall.walk(), recall.walk(), "walk consumes no RNG");
    }

    /// `walk` is the canonical edge mapping: under the default `Snap` it agrees with
    /// [`Recall::history`], so routing reach through it changes no edge semantics — it only
    #[test]
    fn walk_edges_match_history_under_default_snap() {
        let recall = Witness::initial(Game::root().dealer())
            .push(Action::Call(1))
            .push(Action::Check);
        let flop = recall.head().deck().deal(Street::Pref, &mut rand::rng());
        let recall = recall
            .push(Action::Draw(flop))
            .push(Action::Raise(6))
            .push(Action::Call(6));
        assert_eq!(
            recall.walk().into_iter().map(|s| s.edge()).collect::<Vec<_>>(),
            recall.history(),
            "canonical walk must reproduce the default-Snap edge path"
        );
    }

    /// Regression: under the default `Translation::Snap` (which is what `translation()` returns
    /// when `init_translation` was never called), `Recall::history()` produces the same edge
    #[test]
    fn history_under_default_snap_matches_edgify_walk() {
        let recall = Witness::initial(Game::root().dealer())
            .push(Action::Call(1))
            .push(Action::Check);
        let flop = recall.head().deck().deal(Street::Pref, &mut rand::rng());
        let recall = recall
            .push(Action::Draw(flop))
            .push(Action::Raise(6))
            .push(Action::Call(6));
        let turn = recall.head().deck().deal(Street::Flop, &mut rand::rng());
        let recall = recall
            .push(Action::Draw(turn))
            .push(Action::Check)
            .push(Action::Raise(8));
        let translated = recall.history();
        let manual_edgify_walk = recall
            .states()
            .into_iter()
            .zip(recall.actions().iter())
            .scan(Subgame::default(), |past, (game, action)| {
                let edge = game.edgify(*action, past.aggression());
                *past = (*past).into_iter().chain(std::iter::once(edge)).collect();
                Some(edge)
            })
            .collect::<Vec<_>>();
        assert_eq!(translated, manual_edgify_walk, "default Snap must reproduce the historical edgify path");
    }

    /// Under the default `Snap`, `typed_history` emits every step as `Translated::Snap`, and
    /// unwrapping those snaps reproduces `history` exactly.
    #[test]
    fn typed_history_under_default_snap_matches_history() {
        let recall = Witness::initial(Game::root().dealer())
            .push(Action::Call(1))
            .push(Action::Check);
        let flop = recall.head().deck().deal(Street::Pref, &mut rand::rng());
        let recall = recall
            .push(Action::Draw(flop))
            .push(Action::Raise(6))
            .push(Action::Call(6));
        let unwound = recall
            .typed_history()
            .into_iter()
            .map(pokerkit::Translated::snapped)
            .collect::<Vec<_>>();
        assert_eq!(unwound, recall.history(), "typed_history snaps must match history under default Snap");
    }
}
