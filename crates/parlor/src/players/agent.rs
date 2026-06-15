//! [`Agent<B>`] — wraps a [`Brain`] and implements [`Player`].
//!
//! The single [`Player`] impl in this module covers every cell of the
//! bot-config hypercube. Agent's job each turn is to read the brain's
//! distribution and sample from it; the variety in bot behavior comes
//! entirely from the brain's structure (`Blueprint`, `Depth<B>`,
//! `World<B>`, `Dirac<B>` — and any future wrapper that adds an axis).
//!
//! Dirac-as-Brain means sampling from a Dirac delta = deterministic
//! argmax — no separate "argmax player" code path, the distribution's
//! shape carries the semantics.
use std::collections::BTreeMap;

use holdem::Flagship;
use kicker::Action;
use kicker::Edge;
use kicker::Game;
use kicker::Recall;
use kicker::Witness;
use pokerkit::Probability;
use rand::distr::weighted::WeightedIndex;
use rand::prelude::*;

use super::Brain;
use super::Mount;
use super::Tag;
use crate::Player;

pub struct Agent<B>
where
    B: Brain,
{
    brain: B,
}

impl<B> Agent<B>
where
    B: Brain + Mount + 'static,
{
    /// Mount and box in one shot. Used by [`zoo`](super::zoo) so each
    /// match arm fits on a single line.
    pub fn boxed(tag: Tag, model: &'static Flagship) -> Box<dyn Player> {
        Box::new(Self::mount(tag, model))
    }

    /// Sample an action from a distribution, with debug-level telemetry.
    /// Bound to `Agent` because both inputs (`brain` for the label, the
    /// distribution itself) come from this struct's state and decision-
    /// time game ref.
    fn sample(&self, game: &Game, dist: &BTreeMap<Edge, Probability>) -> Action {
        let label = self.brain.tag().label;
        let edges = dist.keys().copied().collect::<Vec<_>>();
        let weights = dist.values().copied().collect::<Vec<_>>();
        let action = WeightedIndex::new(&weights)
            .ok()
            .map(|d| edges[d.sample(&mut rand::rng())])
            .map_or_else(|| game.legal().choose(&mut rand::rng()).copied().unwrap(), |edge| game.actionize(edge));
        let policy = edges
            .iter()
            .zip(&weights)
            .map(|(e, p)| format!("{e}={p:.3}"))
            .collect::<Vec<_>>()
            .join(" ");
        let entropy = weights.iter().filter(|p| **p > 0.0).map(|p| -p * p.ln()).sum::<f32>();
        tracing::debug!(
            player = label,
            policy = %policy,
            entropy,
            action = %action,
            "sampled action",
        );
        action
    }
}

impl<B> Mount for Agent<B>
where
    B: Brain + Mount,
{
    fn mount(tag: Tag, model: &'static Flagship) -> Self {
        Self {
            brain: B::mount(tag, model),
        }
    }
}

#[async_trait::async_trait]
impl<B> Player for Agent<B>
where
    B: Brain + Mount + 'static,
{
    fn shows(&self) -> bool {
        true
    }

    #[tracing::instrument(skip_all, name = "subgame.decide", fields(
        variant = tracing::field::Empty,
        depth   = tracing::field::Empty,
        world   = tracing::field::Empty,
        dirac   = tracing::field::Empty,
    ))]
    async fn decide(&mut self, recall: &Witness) -> Action {
        let tag = self.brain.tag();
        let span = tracing::Span::current();
        span.record("variant", tag.label);
        span.record("depth", if tag.config.depth { "on" } else { "off" });
        span.record("world", if tag.config.world { "on" } else { "off" });
        span.record("dirac", if tag.config.dirac { "on" } else { "off" });
        let dist = self.brain.distrib(recall);
        self.sample(&recall.head(), &dist)
    }
}
