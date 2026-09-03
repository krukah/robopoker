//! [`Nest<B>`] — Modicum-style off-tree action nesting at play time.
//!
//! Wraps an inner [`Brain`] (canonically `World<Depth<Blueprint>>`). When the
//! opponent has made an off-tree raise this street, `Nest` re-solves the
//! augmented subgame (`flagship.adapt_nested`) instead of snapping; otherwise
//! it delegates to the inner brain.
//!
//! # Why `distrib` is overridden (not `solve`)
//!
//! Off-tree raises only exist under [`pokerkit::Translation::Exact`], and the
//! default [`Brain::distrib`] pipeline routes through `NlheInfo::from` →
//! `Recall::history()`, which **panics** on the `Translated::Free` arm. So the
//! whole postflop path here must be off-tree-safe: everything is derived from
//! [`Recall::typed_history`] and [`Recall::head`] (which replay actions
//! directly, never through translation), never from `history()`.
//!
//! # Scope
//!
//! Handles a single off-tree entry per current-street subgame (the Modicum
//! case). Multi-off-tree lines (a nested re-solve within a nested re-solve)
//! fall back to an off-tree-safe blueprint lookup rather than panicking.
//! Runtime-validated at N4 on the retrained V1 blueprint (live-solve smoke +
//! Slumbot off-tree baseline, 0 panics).
use std::collections::BTreeMap;
use std::time::Duration;

use deuce::Street;
use kicker::*;
use mccfr::RefProf;
use mccfr::Solver;
use nlhe::*;
use pokerkit::Probability;
use pokerkit::Translated;
use subgame::SubgameHyperParams;

use super::Brain;
use super::Mount;
use super::Solved;
use super::Tag;

pub struct Nest<B>
where
    B: Brain,
{
    inner: B,
}

impl<B> Mount for Nest<B>
where
    B: Brain + Mount,
{
    fn mount(tag: Tag, model: &'static Flagship) -> Self {
        Self {
            inner: B::mount(tag, model),
        }
    }
}

impl<B> Nest<B>
where
    B: Brain,
{
    /// Whether a translated step stayed on the abstraction grid. A line is
    /// on-tree iff every step is `Snap`; any `Free` is an off-tree action.
    fn is_snap(step: &Translated<Edge, Action>) -> bool {
        matches!(step, Translated::Snap(_))
    }

    /// Index one-past the last `Draw` — the start of the current street in a
    /// `typed_history` slice.
    fn street_start(typed: &[Translated<Edge, Action>]) -> usize {
        typed
            .iter()
            .rposition(|s| matches!(s, Translated::Snap(Edge::Draw)))
            .map_or(0, |i| i + 1)
    }

    /// The latest off-tree action in the current street, as `(action_index,
    /// action)`. Off-tree actions are exactly the `Translated::Free` arm — the
    /// whole `Action` flows straight through to `adapt_nested`/`augment`, with
    /// no assumption here about *what* the action is.
    fn current_offtree(typed: &[Translated<Edge, Action>]) -> Option<(usize, Action)> {
        let start = Self::street_start(typed);
        typed[start..].iter().enumerate().rev().find_map(|(k, s)| match s {
            Translated::Free(action) => Some((start + k, *action)),
            _ => None,
        })
    }

    /// Hero's current info-set, built off-tree-safely: current-street canonical
    /// choice edges only (off-tree `Free` edges dropped, matching how
    /// `NestEncoder` keys hero's augmented-branch node), plus the live hand
    /// bucket and legal actions from `recall.head()`.
    fn safe_info(model: &Flagship, recall: &Witness, typed: &[Translated<Edge, Action>]) -> NlheInfo {
        let subgame = typed[Self::street_start(typed)..]
            .iter()
            .filter_map(|s| match s {
                Translated::Snap(e) if e.is_choice() => Some(*e),
                _ => None,
            })
            .collect::<Path>();
        let choices = recall.head().choices(subgame.aggression());
        NlheInfo::from((subgame, model.encoder().abstraction(&recall.seen()), choices))
    }

    /// Roll the recall back by `count` actions (removing the off-tree raise and
    /// everything after it) to reach the nesting entry — the node where the
    /// opponent is about to make the off-tree raise.
    fn rollback(recall: &Witness, count: usize) -> Witness {
        (0..count).fold(recall.clone(), |r, _| r.undo())
    }

    /// Off-tree-safe blueprint policy at `info` (never touches `history()`).
    fn blueprint(model: &Flagship, info: &NlheInfo) -> BTreeMap<Edge, Probability> {
        model
            .profile()
            .averaged_distribution(info)
            .into_iter()
            .filter(|(e, _)| e.is_choice())
            .map(|(e, p)| (Edge::from(e), p))
            .collect()
    }
}

impl<B> Brain for Nest<B>
where
    B: Brain,
{
    fn tag(&self) -> Tag {
        self.inner.tag()
    }

    fn model(&self) -> &'static Flagship {
        self.inner.model()
    }

    fn distrib(&self, recall: &Witness) -> BTreeMap<Edge, Probability> {
        let typed = recall.typed_history();
        if typed.iter().all(Self::is_snap) {
            return self.inner.distrib(recall);
        }
        let model = self.model();
        let ref info = Self::safe_info(model, recall, &typed);
        let game = recall.head();
        // Nest only the hand's first off-tree action, at a postflop hero node.
        // A preceding `Free` (multi-off-tree) would survive the rollback and
        // panic `adapt_nested`'s history walk — so that shape, along with every
        // preflop/non-choice node, falls through to the off-tree-safe blueprint.
        match Self::current_offtree(&typed) {
            Some((idx, action))
                if game.street() != Street::Pref
                    && matches!(game.turn(), Turn::Choice(_))
                    && typed[..idx].iter().all(Self::is_snap) =>
            {
                let entry = Self::rollback(recall, typed.len() - idx);
                let deadline = Duration::from_millis(SubgameHyperParams::get().timeout_ms());
                let solved = Solved::run_nested(model.adapt_nested(&entry, action), *info, deadline);
                solved.adopt_refined(&Self::blueprint(model, info), self.tag(), game.street())
            }
            _ => Self::blueprint(model, info),
        }
    }
}
