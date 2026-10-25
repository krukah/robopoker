use super::*;
use deuce::*;
use kicker::*;
use parlor::records::{Hand as HandRecord, Participant, Play};
use pokerkit::*;

/// Namespace for the replay routines.
pub struct Replay;

impl Replay {
    /// Replay a stored hand to extract per-player results.
    pub fn replay(
        hand: &HandRecord,
        participants: &[Participant],
        plays: &[Play],
        seat: Position,
    ) -> anyhow::Result<Recap> {
        let participant = participants
            .iter()
            .find(|p| p.seat() == seat)
            .ok_or_else(|| anyhow::anyhow!("seat {seat} not found"))?;
        let reveals = Replay::plays_arrangement(participant.hole(), hand.board(), plays);
        let witness = plays
            .iter()
            .filter(|p| !p.action().is_blind())
            .filter(|p| !p.action().is_chance())
            .try_fold(
                Witness::initial_with(Turn::Choice(seat), reveals, Replay::stacks(participants)?, hand.dealer()),
                |r, p| r.try_push(p.action()),
            )?;
        let actions = witness
            .plays()
            .into_iter()
            .filter(|&(pos, _, _)| pos == seat)
            .map(|(_, a, s)| (s, a))
            .collect::<Vec<(Street, Action)>>();
        let folded = actions.iter().any(|(_, a)| matches!(a, Action::Fold));
        Ok(Recap::new(
            hand.id(),
            witness.clone(),
            seat,
            participant.hole(),
            participant.stack(),
            participant.pnl(),
            hand.pot(),
            witness.head().street(),
            folded,
            witness.head().is_showdown(),
            actions
                .iter()
                .any(|(s, a)| *s == Street::Pref && matches!(a, Action::Call(_) | Action::Raise(_) | Action::Shove(_))),
            actions
                .iter()
                .any(|(s, a)| *s == Street::Pref && matches!(a, Action::Raise(_) | Action::Shove(_))),
            actions,
        ))
    }

    /// Build the stacks array from participants.
    pub fn stacks(participants: &[Participant]) -> anyhow::Result<[Chips; N]> {
        participants.iter().try_fold([0i16; N], |mut acc, p| {
            anyhow::ensure!(p.seat() < N, "seat {} out of bounds for N={}", p.seat(), N);
            acc[p.seat()] = p.stack();
            Ok(acc)
        })
    }

    /// Build an [`Arrangement`] from a hole and board stored as unordered sets.
    ///
    /// Deterministic per-street card assignment via [`Hand`] iteration order
    /// (low-to-high). Matches [`Replayer`]'s board ordering so [`Witness`]
    /// Draw actions stay consistent with the Replayer's deals.
    pub fn board_arrangement(hole: Hole, board: Board) -> Arrangement {
        Arrangement::from(Hand::from(hole).chain(Hand::from(board)).collect::<Vec<Card>>())
    }

    /// Build an [`Arrangement`] from a hole and the [`Draw`](Action::Draw)
    /// actions in a play sequence, preserving the actual deal order.
    ///
    /// Falls back to [`Replay::board_arrangement`] when no Draw actions are present
    /// (backward compatibility with hands recorded before draws were persisted).
    pub fn plays_arrangement(hole: Hole, board: Board, plays: &[Play]) -> Arrangement {
        let draws = plays
            .iter()
            .map(parlor::Play::action)
            .filter_map(Action::hand)
            .flatten()
            .collect::<Vec<Card>>();
        if draws.is_empty() {
            Replay::board_arrangement(hole, board)
        } else {
            Arrangement::from(Hand::from(hole).chain(draws).collect::<Vec<Card>>())
        }
    }
}
