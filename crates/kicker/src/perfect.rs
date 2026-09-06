//! Complete-information game history for **training time**, the god's-view
//! counterpart to [`Witness`].
//!
//! CFR traversal knows both players' cards, but strategies are indexed only by
//! `NlheInfo` (public edges + private bucket). `Perfect` stores the complete
//! root state needed for reach probabilities and counterfactual values. As with
//! `Witness`, blinds are not in `actions` — `root` is already POST-blind.
use super::*;
use deuce::*;

/// Complete game history with both players' cards known.
#[derive(Debug, Clone)]
pub struct Perfect {
    root: Game,
    actions: Vec<Action>,
}

impl From<(&Witness, Hole)> for Perfect {
    /// Creates history from witness with assumed opponent hole; hero is
    /// derived from `witness.turn()`.
    fn from((witness, hole): (&Witness, Hole)) -> Self {
        debug_assert_eq!(witness.base().n(), 2);
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
    /// Rebuilds the [`Arrangement`] from hero's hole cards and the history's
    /// [`Draw`](Action::Draw)s, preserving per-street card assignment.
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
