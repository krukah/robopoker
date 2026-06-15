//! Reference resolution: convert string refs in scenarios.json into
//! the typed inputs `Witness::try_build` requires.
//!
//! This is where the type-safety win lives — every parser used here
//! is the canonical one from the gameplay/cards crates, so a typo or
//! invalid action sequence fails at this layer with a real error,
//! not silently as a SKIP later.

use crate::schema::*;
use deuce::*;
use kicker::*;

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

    pub fn category_default(&self, name: &str) -> Option<&'a Expect> {
        self.scenarios
            .categories
            .get(name)
            .and_then(|c| c.default_expect.as_ref())
    }
}

/// Build a typed `Witness` from a hand-ref + history-ref pair.
///
/// Fails on bad encoding, unknown ref, board/hole collision, illegal action
/// sequence, or `expected_spr` mismatch against the SPR bucket the witness
/// actually lands in.
pub fn build_witness(catalog: &Catalog, hand_ref: &str, history_ref: &str) -> anyhow::Result<Witness> {
    let hand = catalog.hand(hand_ref)?;
    let history = catalog.history(history_ref)?;
    let seen = match &history.seen_template {
        None => hand.cards.clone(),
        Some(tmpl) => tmpl.replacen('*', &hand.cards, 1),
    };
    let turn = Turn::try_from(history.turn.as_str()).map_err(|e| anyhow::anyhow!("turn `{}`: {e}", history.turn))?;
    let obs = Observation::try_from(seen.as_str()).map_err(|e| anyhow::anyhow!("observation `{seen}`: {e:?}"))?;
    let past = history
        .past
        .iter()
        .map(|s| Action::try_from(s.as_str()).map_err(|e| anyhow::anyhow!("action `{s}`: {e}")))
        .collect::<anyhow::Result<Vec<_>>>()?;
    let witness = match history.stacks {
        None => Witness::try_build(turn, obs, past),
        Some(stacks) => past
            .into_iter()
            .try_fold(Witness::initial_with(turn, Arrangement::from(obs), stacks, 0), |w, a| w.try_push(a)),
    }
    .map_err(|e| anyhow::anyhow!("witness build (hand={hand_ref}, history={history_ref}): {e}"))?;
    if let Some(claim) = &history.expected_spr {
        let want = match claim.to_ascii_lowercase().as_str() {
            "committed" => SPR::Committed,
            "low" => SPR::Low,
            "mid" => SPR::Mid,
            "deep" => SPR::Deep,
            x => anyhow::bail!("expected_spr `{x}` must be committed|low|mid|deep"),
        };
        let actual = witness.head().geometry();
        if actual != want {
            anyhow::bail!(
                "history `{history_ref}` declares expected_spr=`{claim}` but witness lands at SPR=`{actual}`. \
                 Fix `stacks`/`past` or correct `expected_spr`."
            );
        }
    }
    Ok(witness)
}

pub fn parse_edge(s: &str) -> anyhow::Result<Edge> {
    Edge::try_from(s).map_err(|e| anyhow::anyhow!("edge `{s}`: {e}"))
}
