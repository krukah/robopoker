//! Reference resolution: convert string refs in scenarios.json into
//! the typed inputs `Witness::try_build` requires.
//!
//! This is where the type-safety win lives — every parser used here
//! is the canonical one from the gameplay/cards crates, so a typo or
//! invalid action sequence fails at this layer with a real error,
//! not silently as a SKIP later.

use crate::schema::{HandDef, HistoryDef, Scenarios};
use rbp_cards::Observation;
use rbp_gameplay::{Action, Edge, Turn, Witness};

/// Wraps a Scenarios document with typed lookup helpers.
pub struct Catalog<'a> {
    scenarios: &'a Scenarios,
}

impl<'a> Catalog<'a> {
    pub fn new(scenarios: &'a Scenarios) -> Self {
        Self { scenarios }
    }

    pub fn hand(&self, name: &str) -> anyhow::Result<&'a HandDef> {
        self.scenarios
            .hands
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("unknown hand `{name}`"))
    }

    pub fn history(&self, dotted: &str) -> anyhow::Result<&'a HistoryDef> {
        let (street, name) = dotted
            .split_once('.')
            .ok_or_else(|| anyhow::anyhow!("history ref `{dotted}` must be `street.name`"))?;
        self.scenarios
            .histories
            .get(street)
            .ok_or_else(|| anyhow::anyhow!("unknown street `{street}` in history ref `{dotted}`"))?
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("unknown history `{name}` in street `{street}`"))
    }

    pub fn category_default(&self, name: &str) -> Option<&'a crate::schema::Expect> {
        self.scenarios
            .categories
            .get(name)
            .and_then(|c| c.default_expect.as_ref())
    }
}

/// Build a typed `Witness` from a hand-ref + history-ref pair.
///
/// Failures at this step are typically (a) bad hand encoding, (b) unknown
/// reference, (c) board/hole conflict (same card in both), or (d) action
/// sequence the rules engine considers illegal.
pub fn build_witness(catalog: &Catalog, hand_ref: &str, history_ref: &str) -> anyhow::Result<Witness> {
    let hand = catalog.hand(hand_ref)?;
    let history = catalog.history(history_ref)?;

    // Construct the seen string. For preflop, just the hole. For postflop,
    // substitute the hole into the `_seen` template (`* ~ Kh 7d 2c`).
    let seen_str = match &history.seen_template {
        None => hand.cards.clone(),
        Some(tmpl) => tmpl.replacen('*', &hand.cards, 1),
    };

    let turn = Turn::try_from(history.turn.as_str()).map_err(|e| anyhow::anyhow!("turn `{}`: {e}", history.turn))?;
    let observation =
        Observation::try_from(seen_str.as_str()).map_err(|e| anyhow::anyhow!("observation `{seen_str}`: {e:?}"))?;
    let past = history
        .past
        .iter()
        .map(|s| Action::try_from(s.as_str()).map_err(|e| anyhow::anyhow!("action `{s}`: {e}")))
        .collect::<anyhow::Result<Vec<_>>>()?;

    Witness::try_build(turn, observation, past)
        .map_err(|e| anyhow::anyhow!("witness build (hand={hand_ref}, history={history_ref}): {e}"))
}

/// Parse a scenarios `edge` string ("F", "*", "!", "1:2", "2bb") into the
/// typed `Edge` enum.
pub fn parse_edge(s: &str) -> anyhow::Result<Edge> {
    Edge::try_from(s).map_err(|e| anyhow::anyhow!("edge `{s}`: {e}"))
}
