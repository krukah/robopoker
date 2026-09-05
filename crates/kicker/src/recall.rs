//! Trait for game history types that can replay actions.
//!
//! Both [`Perfect`] and [`Witness`] represent game histories from different
//! perspectives (complete vs hero-only information). This trait captures
//! their shared interface for action replay, state reconstruction, and
//! edge conversion.
//!
//! # Blind Handling
//!
//! Blinds are constant and deterministic, so they are NOT stored in `actions()`.
//! The `root()` method returns a POST-blind game state. Use `all_actions()`
//! when you need the complete action sequence including blinds (e.g., for display).
use super::*;
use deuce::Card;
use pokerkit::Translated;

/// A game history that can be replayed from a root state.
///
/// Provides default implementations for derived computations:
/// - `head()` — Current game state
/// - `states()` — Full sequence of game states
/// - `history()` — Full edge history (all streets)
/// - `subgame()` — Current street edges only
/// - `choices()` — Available actions at current state
/// - `aggression()` — Trailing aggressive action count
/// - `complete()` — Complete action sequence including blinds (for display)
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
            .rev()
            .take_while(|a| a.is_choice())
            .filter(|a| a.is_aggro())
            .count()
    }

    /// Full edge history (all streets).
    ///
    /// Maps each `Action` onto an `Edge` via the global [`pokerkit::translation`].
    /// All current [`pokerkit::Translation`] variants (`Snap`, `Harmonic`,
    /// `Phargmax`) always resolve on-tree; the `Translated::Free` arm
    /// is unreachable under the live enum and triggers `unreachable!()`
    /// if hit. If a future Brown-style variant is added, this arm needs
    /// to be revisited (likely via a custom history walker on the
    /// player that consumes off-tree actions).
    fn history(&self) -> Vec<Edge> {
        let translation = pokerkit::translation();
        let ref mut rng = rand::rng();
        self.states()
            .into_iter()
            .zip(self.actions().iter())
            .scan(Subgame::default(), |past, (game, action)| {
                let edge = game.snapped(past.aggression(), *action, &translation, rng);
                *past = past.flow(edge);
                Some(edge)
            })
            .collect()
    }

    /// Full typed history, preserving off-tree actions.
    ///
    /// Like [`Self::history`] but does not panic on the [`Translated::Free`]
    /// arm: off-grid raises (emitted only under [`pokerkit::Translation::Exact`])
    /// are handed back verbatim as `Translated::Free(Action::Raise(_))`. A
    /// nesting player walks this to locate the off-tree entry and re-solve an
    /// augmented subgame — see `docs/active/off-tree-nesting.md`.
    ///
    /// For depth (aggression) bookkeeping the walk still advances its edge
    /// path by the *canonical* snap of each action ([`Game::edgify`]), so a
    /// trailing on-tree action after an off-tree raise translates against the
    /// correct grid cell. The output element stays `Free` regardless.
    fn typed_history(&self) -> Vec<Translated<Edge, Action>> {
        let translation = pokerkit::translation();
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

    /// Current street edges only (trailing choice edges before any Draw).
    /// Width-pinned by player count: heads-up `Subgame = Path<1>`.
    fn subgame(&self) -> Subgame {
        self.history()
            .into_iter()
            .rev()
            .take_while(Edge::is_choice)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
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

    /// Regression: under the default `Translation::Snap` (which is what
    /// `translation()` returns when `init_translation` was never called),
    /// `Recall::history()` produces the same edge sequence as a fresh
    /// manual walk that calls `Game::edgify` directly. This proves the
    /// new wiring is behavior-preserving for every existing caller.
    ///
    /// This test does NOT call `init_translation(...)` — doing so would
    /// corrupt other tests in the same binary because `TRANSLATION` is a
    /// process-global `OnceLock`. The default-Snap path is tested here;
    /// non-Snap behavior is exhaustively covered via direct
    /// `Game::translate` tests in `game.rs`.
    #[test]
    fn history_under_default_snap_matches_edgify_walk() {
        let recall = Witness::initial(Game::root().dealer())
            .push(Action::Call(1))
            .push(Action::Check);
        let flop = recall.head().deck().deal(Street::Pref);
        let recall = recall
            .push(Action::Draw(flop))
            .push(Action::Raise(6))
            .push(Action::Call(6));
        let turn = recall.head().deck().deal(Street::Flop);
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

    /// Under the default `Snap`, `typed_history` emits every step as
    /// `Translated::Snap`, and unwrapping those snaps reproduces `history`
    /// exactly. (The `Free` arm is only reachable under `Translation::Exact`,
    /// which cannot be set here without corrupting the process-global
    /// `OnceLock`; off-tree preservation is covered by the direct
    /// `Game::translate` Exact tests in `game.rs`.)
    #[test]
    fn typed_history_under_default_snap_matches_history() {
        let recall = Witness::initial(Game::root().dealer())
            .push(Action::Call(1))
            .push(Action::Check);
        let flop = recall.head().deck().deal(Street::Pref);
        let recall = recall
            .push(Action::Draw(flop))
            .push(Action::Raise(6))
            .push(Action::Call(6));
        let unwound = recall
            .typed_history()
            .into_iter()
            .map(|step| match step {
                pokerkit::Translated::Snap(edge) => edge,
                pokerkit::Translated::Free(_) => unreachable!("default Snap never emits Free"),
            })
            .collect::<Vec<_>>();
        assert_eq!(unwound, recall.history(), "typed_history snaps must match history under default Snap");
    }
}
