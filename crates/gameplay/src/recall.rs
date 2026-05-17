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
use rbp_cards::Card;
use rbp_translate::Translated;

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
    /// Maps each `Action` onto an `Edge` via the global [`rbp_core::translation`].
    /// All current [`rbp_core::Translation`] variants (`Snap`, `Harmonic`,
    /// `Phargmax`) always resolve on-tree; the `Translated::Free` arm
    /// is unreachable under the live enum and triggers `unreachable!()`
    /// if hit. If a future Brown-style variant is added, this arm needs
    /// to be revisited (likely via a custom history walker on the
    /// player that consumes off-tree actions).
    fn history(&self) -> Vec<Edge> {
        let translation = rbp_core::translation();
        let rng = &mut rand::rng();
        self.states()
            .into_iter()
            .zip(self.actions().iter())
            .scan(Path::default(), |past, (game, action)| {
                let edge = match game.translate(*action, past.aggression(), &translation, rng) {
                    Translated::Snap(edge) => edge,
                    Translated::Free(_) => unreachable!(
                        "no current Translation variant emits Translated::Free; \
                         add a custom history walker for any future \
                         off-tree-emitting translation",
                    ),
                };
                *past = (*past).into_iter().chain(std::iter::once(edge)).collect();
                Some(edge)
            })
            .collect()
    }

    /// Current street edges only (trailing choice edges before any Draw).
    fn subgame(&self) -> Path {
        self.history()
            .into_iter()
            .rev()
            .take_while(|e| e.is_choice())
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
    use rbp_cards::Street;

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
            .scan(Path::default(), |past, (game, action)| {
                let edge = game.edgify(*action, past.aggression());
                *past = (*past).into_iter().chain(std::iter::once(edge)).collect();
                Some(edge)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            translated, manual_edgify_walk,
            "default Snap must reproduce the historical edgify path",
        );
    }
}
