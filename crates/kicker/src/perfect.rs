//! Complete-information game history for **training time**.
//!
//! # Information Boundary
//!
//! | Type | Perspective | Context |
//! |------|-------------|---------|
//! | `Witness` | Hero only | Inference (strategy lookup) |
//! | `Perfect` | Both hands | Training (CFR traversal) |
//!
//! During CFR training, we traverse the game tree knowing both players' cards
//! (god's view), but strategies are indexed only by `NlheInfo` (public edges +
//! private bucket). `Perfect` stores the complete root state needed for reach
//! probability computation and counterfactual value calculation.
//!
//! # Conversions
//!
//! ```text
//! Perfect::from((witness, hole))  ────►  Perfect     (add opponent info)
//!                                 ◄────
//! perfect.witness(hero)                              (erase opponent info)
//!
//! witness.histories() ─────►  Vec<(Obs, Perfect)>  (iterate all opponents)
//! ```
//!
//! # Blind Handling
//!
//! Like `Witness`, blinds are constant and NOT stored in `actions`.
//! The `root` field stores a POST-blind game state.
use super::*;
use deuce::*;

/// Complete game history with both players' cards known.
///
/// Stores root game state (POST-blind, with all cards set) and action sequence
/// (excluding blinds). Game states are derived by applying actions to root.
#[derive(Debug, Clone)]
pub struct Perfect {
    root: Game,
    actions: Vec<Action>,
}

impl From<(&Witness, Hole)> for Perfect {
    /// Creates history from witness with assumed opponent hole.
    ///
    /// Hero is derived from `witness.turn()`. The root game has:
    /// - Hero's cards from `witness.seen()`
    /// - Opponent's cards from `hole` parameter
    /// - Blinds already posted (POST-blind state)
    fn from((witness, hole): (&Witness, Hole)) -> Self {
        debug_assert!(witness.base().n() == 2);
        let preblind = witness.base().fix(witness.turn(), hole);
        let root = Game::blinds().into_iter().fold(preblind, |mut g, a| g.consume(a));
        Self {
            root,
            actions: witness.actions().to_vec(),
        }
    }
}

impl Recall for Perfect {
    fn root(&self) -> Game {
        self.root
    }

    fn actions(&self) -> &[Action] {
        &self.actions
    }
}

#[allow(dead_code)]
impl Perfect {
    /// Erases opponent information, returning hero's perspective.
    ///
    /// Reconstructs the [`Arrangement`] from hero's hole cards and the
    /// [`Draw`](Action::Draw) actions in the history, preserving the
    /// per-street card assignment from the original [`Witness`].
    fn erase(&self, hero: Turn) -> Witness {
        let hole = self.root.seats()[hero.position()].cards();
        let reveals = Arrangement::from(
            Hand::from(hole)
                .chain(self.actions.iter().filter_map(Action::hand).flatten())
                .collect::<Vec<Card>>(),
        );
        let actions = self.actions.iter().filter(|a| a.is_choice()).copied().collect();
        Witness::try_arrange(hero, reveals, actions).expect("valid erase")
    }
}
