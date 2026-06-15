//! [`Brain`] — the trait every bot composition implements.
//!
//! Each cell of the bot-config hypercube is a kind of brain:
//! [`Blueprint`](super::Blueprint) is the base (in-memory blueprint
//! lookup); [`Depth`](super::Depth), [`World`](super::World), and
//! `World<Depth<…>>` wrap an inner brain with a subgame solver.
//!
//! Two trivial accessors (`tag`, `model`), one overrideable hook
//! ([`solve`](Brain::solve)), one default body ([`distrib`](Brain::distrib)).
//! Subgame layers override only `solve`; preflop blueprint, postflop
//! solve+blend, and the no-solve fallback all live in the default.
use std::collections::BTreeMap;
use std::time::Duration;

use cowboys::*;
use endgame::SubgameHyperParams;
use holdem::*;
use kicker::Street;
use mccfr::RefProf;
use mccfr::Solver;
use pokerkit::Probability;
use pokerkit::Utility;

use super::Solved;
use super::Tag;

pub trait Brain: Send {
    /// The bot's identity + cube coordinate. Used for telemetry labels
    /// and trace fields at every emission site downstream.
    fn tag(&self) -> Tag;
    fn model(&self) -> &'static Flagship;

    /// Override to run a subgame solver. Default = blueprint base (no
    /// solve). Subgame layers ([`Depth`](super::Depth),
    /// [`World`](super::World), `World<Depth<…>>`) override to run their
    /// `flagship.adapt_*` and harvest refined+visits.
    fn solve(&self, _recall: &Witness, _info: NlheInfo, _deadline: Duration) -> Option<Solved> {
        None
    }

    /// In-memory blueprint policy at the current decision. Both branches
    /// of the postflop pipeline (preflop pass-through and "no-solve"
    /// fallback) read this; subgame impls also use it as the prior for
    /// the visits blend.
    fn policy(&self, recall: &Witness) -> BTreeMap<Edge, Probability> {
        let model = self.model();
        let info = NlheInfo::from((recall, model.encoder().abstraction(&recall.seen())));
        model
            .profile()
            .averaged_distribution(&info)
            .into_iter()
            .filter(|(e, _)| e.is_choice())
            .map(|(e, p)| (Edge::from(e), p))
            .collect()
    }

    /// Postflop pipeline shared by every brain.
    ///
    /// - Preflop → blueprint lookup, regardless of subgame layers.
    /// - Postflop with no solver (blueprint base) → blueprint lookup.
    /// - Postflop with a solver → run `solve`, blend refined with
    ///   blueprint by visit counts via [`Solved::extract`] (the only
    ///   extraction strategy — pure-blueprint = use
    ///   [`Blueprint`](super::Blueprint) directly).
    fn distrib(&self, recall: &Witness) -> BTreeMap<Edge, Probability> {
        let model = self.model();
        let game = recall.head();
        let info = NlheInfo::from((recall, model.encoder().abstraction(&recall.seen())));
        if game.street() == Street::Pref {
            return self.policy(recall);
        }
        let tag = self.tag();
        let timeout = Duration::from_millis(SubgameHyperParams::get().timeout_ms());
        let span = tracing::info_span!(
            "subgame.solve",
            variant = tag.label,
            depth = if tag.config.depth { "on" } else { "off" },
            world = if tag.config.world { "on" } else { "off" },
            dirac = if tag.config.dirac { "on" } else { "off" },
            iterations = tracing::field::Empty,
            regret_norm = tracing::field::Empty,
        );
        let Some(solved) = span.in_scope(|| self.solve(recall, info, timeout)) else {
            return self.policy(recall);
        };
        let visits = solved
            .visits()
            .values()
            .map(|&v| v as Utility)
            .sum::<Utility>()
            .max(1.0);
        let relative = solved.regret() / visits / game.pot().max(1) as Utility;
        span.record("iterations", solved.iterations());
        span.record("regret_norm", relative);
        solved.emit_postflop(tag, game.street(), game.pot(), relative);
        let ref policy = self.policy(recall);
        solved.extract(policy, tag, game.street())
    }
}
