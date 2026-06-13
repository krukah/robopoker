use super::*;
use rbp_cards::*;
use rbp_core::*;
use rbp_gameplay::*;
use rbp_gameroom::records::{Hand as HandRecord, Participant, Play};
use rbp_nlhe::*;
use std::sync::Arc;
use tokio_postgres::Client;

/// AIVAT variance reduction estimator.
///
/// Uses blueprint strategy probabilities and expected values as control
/// variates to reduce the variance of player performance estimates.
/// Applies corrections at three node types:
///   1. Hero action nodes:    correction = Σ_a [σ(a) * v(a)] - v(observed_a)
///   2. Villain action nodes: negated hero-perspective correction (zero-sum)
///   3. Chance nodes:         E[baseline] - observed baseline (turn + river only)
pub struct Aivat(Arc<Client>);

impl Aivat {
    pub fn new(client: Arc<Client>) -> Self {
        Self(client)
    }
    /// Compute AIVAT-adjusted value for a single hand from one player's perspective.
    pub async fn evaluate(
        &self,
        hand: &HandRecord,
        parts: &[Participant],
        plays: &[Play],
        seat: Position,
        recap: &Recap,
    ) -> anyhow::Result<AivatResult> {
        let (hero, villain, chance) = self.corrections(hand, parts, plays, seat).await?;
        let total = hero + villain + chance;
        Ok(AivatResult {
            raw: recap.won(),
            adjusted: recap.won() as Utility + total,
            corrections: total,
            hero_corrections: hero,
            villain_corrections: villain,
            chance_corrections: chance,
        })
    }
    /// Compute AIVAT delta from a batch of results and a precomputed summary.
    pub fn summarize(summary: &Summary, results: &[AivatResult]) -> AivatDelta {
        let series: Vec<Utility> = results.iter().map(|r| r.adjusted).collect();
        let n = series.len() as f32;
        let won: Utility = series.iter().sum();
        let mean = ratio(won, n);
        let variance = ratio(series.iter().map(|&x| (x - mean) * (x - mean)).sum(), n);
        let stderr = ratio(variance.sqrt(), n.sqrt());
        let raw = summary.stddev() * summary.stddev();
        let adj = stderr * stderr * n;
        AivatDelta {
            series,
            won,
            stderr,
            reduction: if adj > 0.0 { raw / adj } else { 1.0 },
            pvalue: if stderr > 0.0 { 2.0 * erf(-mean.abs() / stderr) } else { 1.0 },
        }
    }
    /// Compute corrections at hero actions, villain actions, and chance nodes.
    async fn corrections(
        &self,
        hand: &HandRecord,
        parts: &[Participant],
        plays: &[Play],
        seat: Position,
    ) -> anyhow::Result<(Utility, Utility, Utility)> {
        let mut walker = Replayer::new(hand, parts, plays)?;
        let villain_seat = 1 - seat;
        let hero_hole = parts
            .iter()
            .find(|p| p.seat() == seat)
            .map(rbp_gameroom::Participant::hole)
            .ok_or_else(|| anyhow::anyhow!("seat {seat} not found"))?;
        let villain_hole = parts
            .iter()
            .find(|p| p.seat() == villain_seat)
            .map(rbp_gameroom::Participant::hole)
            .ok_or_else(|| anyhow::anyhow!("seat {villain_seat} not found"))?;
        let hero = Turn::Choice(seat);
        let villain = Turn::Choice(villain_seat);
        let hero_arr = plays_arrangement(hero_hole, hand.board(), plays);
        let villain_arr = plays_arrangement(villain_hole, hand.board(), plays);
        let mut hero_recall = Witness::try_arrange(hero, hero_arr, Vec::new()).map_err(|e| anyhow::anyhow!("{e}"))?;
        let mut villain_recall =
            Witness::try_arrange(villain, villain_arr, Vec::new()).map_err(|e| anyhow::anyhow!("{e}"))?;
        let hero_hand = rbp_cards::Hand::from(hero_hole);
        let villain_hand = rbp_cards::Hand::from(villain_hole);
        let mut hero_total = 0.0f32;
        let mut villain_total = 0.0f32;
        let mut chance_total = 0.0f32;
        for play in plays.iter().filter(|p| p.action().is_choice()) {
            if walker.terminal() {
                break;
            }
            // Intercept chance nodes for chance corrections before advancing
            while walker.game().turn() == Turn::Chance {
                if walker.game().street() != Street::Pref
                    && let Some(observed) = walker.peek_deal()
                    && let Some(delta) = self
                        .chance_node_correction(
                            &hero_recall,
                            &villain_recall,
                            walker.game(),
                            hero_hand,
                            villain_hand,
                            observed,
                            seat,
                        )
                        .await?
                {
                    chance_total += delta;
                }
                walker.deal_one()?;
            }
            // Hero action-node correction
            if walker.game().turn() == hero
                && play.action().is_choice()
                && let Some(delta) = self
                    .action_node_correction(&hero_recall, walker.game(), play.action())
                    .await?
            {
                hero_total += delta;
            }
            // Villain action-node correction (negated for hero's perspective)
            if walker.game().turn() == villain
                && play.action().is_choice()
                && let Some(delta) = self
                    .action_node_correction(&villain_recall, walker.game(), play.action())
                    .await?
            {
                villain_total -= delta;
            }
            hero_recall = hero_recall
                .try_push(play.action())
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            villain_recall = villain_recall
                .try_push(play.action())
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            walker.apply(play.action())?;
        }
        Ok((hero_total, villain_total, chance_total))
    }
    /// Chance-node correction using SQL CTE for batch computation.
    ///
    /// Enumerates all possible deals, maps each to an abstraction bucket via
    /// isomorphism, then computes E[baseline] - observed_baseline in one query.
    /// Skipped for preflop→flop (C(48,3) = 17,296 deals is too large).
    async fn chance_node_correction(
        &self,
        hero_recall: &Witness,
        villain_recall: &Witness,
        game: &Game,
        hero_hand: rbp_cards::Hand,
        villain_hand: rbp_cards::Hand,
        observed: rbp_cards::Hand,
        seat: Position,
    ) -> anyhow::Result<Option<Utility>> {
        let board = rbp_cards::Hand::from(game.board());
        let mask = rbp_cards::Hand::add(rbp_cards::Hand::add(hero_hand, villain_hand), board);
        let n = game.street().next().n_revealed();
        let deals: Vec<rbp_cards::Hand> = HandIterator::from((n, mask)).collect();
        if deals.is_empty() {
            return Ok(None);
        }
        // Determine who acts first after the deal (same for all deals)
        let sample = game.try_apply(Action::Draw(deals[0]))?;
        let (recall, hero_acts_next) = match sample.turn() {
            Turn::Choice(s) if s == seat => (hero_recall, true),
            Turn::Choice(_) => (villain_recall, false),
            _ => return Ok(None),
        };
        let pocket = *recall.seen().pocket();
        let isos: Vec<i64> = deals
            .iter()
            .map(|d| Observation::from((pocket, rbp_cards::Hand::add(board, *d))))
            .map(|o| i64::from(Isomorphism::from(o)))
            .collect();
        let observed_iso =
            i64::from(Isomorphism::from(Observation::from((pocket, rbp_cards::Hand::add(board, observed)))));
        let info = NlheInfo::from((recall, Abstraction::default()));
        let past = i64::from(info.subgame());
        let choices = i64::from(info.choices());
        let Some((avg, obs)) = self
            .0
            .eval_chance_correction(&isos, past, choices, observed_iso)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?
        else {
            return Ok(None);
        };
        let delta = chance_correction(avg, obs);
        Ok(Some(if hero_acts_next { delta } else { -delta }))
    }
    /// Look up blueprint strategy at a decision point and compute the action correction.
    async fn action_node_correction(
        &self,
        recall: &Witness,
        game: &Game,
        observed: Action,
    ) -> anyhow::Result<Option<Utility>> {
        let iso = Isomorphism::from(recall.seen());
        let abs = match self
            .0
            .eval_abstraction(i64::from(iso))
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?
        {
            Some(a) => Abstraction::from(a),
            None => return Ok(None),
        };
        let info = NlheInfo::from((recall, abs));
        let rows = self
            .0
            .eval_policy(i64::from(info.subgame()), i16::from(info.bucket()), i64::from(info.choices()))
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        if rows.is_empty() {
            return Ok(None);
        }
        let policy: Vec<(Probability, Utility)> = rows.iter().map(|(_, w, v)| (*w, *v)).collect();
        if policy.iter().map(|(w, _)| w).sum::<Probability>() <= 0.0 {
            return Ok(None);
        }
        let edge = NlheEdge::from(game.edgify(observed, recall.aggression()));
        let idx = rows.iter().position(|(e, _, _)| NlheEdge::from(*e as u64) == edge);
        Ok(Some(action_correction(&policy, idx)))
    }
}
