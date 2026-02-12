//! Trait for game history types that can replay actions.
//!
//! Both [`Perfect`] and [`Partial`] represent game histories from different
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
    fn history(&self) -> Vec<Edge> {
        self.states()
            .into_iter()
            .zip(self.actions().iter())
            .scan(Path::default(), |past, (game, action)| {
                let edge = game.edgify(*action, past.aggression());
                *past = past
                    .clone()
                    .into_iter()
                    .chain(std::iter::once(edge))
                    .collect();
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
}
