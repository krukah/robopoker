//! Walking a [`Record`] through `kicker`'s state machine.
//!
//! Chip-exact conformance is not on the table: `pokerkit::Chips = i16` and the
//! blinds are compile-time constants at 1/50 of Pluribus's scale, so a 6-way
//! all-in pot would not even fit. What *is* on the table — and is the part of
//! `GameN<6>` no test currently covers — is the **structure**: whose turn it is,
//! when a street ends, and whether the logged action is legal at all.
//!
//! So this walks the engine and the log side by side at engine scale, and
//! records every place they disagree. A clean [`Trace`] over 10,000 hands is
//! meaningful evidence for the multiway state machine; [`crate::replay`] covers
//! the arithmetic the engine cannot.
//!
//! Chip amounts divide by [`scale`] and are then coerced with `Game::snap`,
//! which is legitimate rounding rather than disagreement — [`Trace::coerced`]
//! counts it separately so it never hides a real fault.

use crate::*;
use anyhow::bail;
use deuce::*;
use kicker::*;
use pokerkit::*;

/// Chip divisor taking the log's scale to the engine's.
///
/// Ratios are what matter and they already agree — Pluribus's 50/100 blinds on
/// 10,000 stacks and our 1/2 on 200 are both 100bb with a half-blind small —
/// so this is a pure unit change, at a 50× loss of granularity.
pub fn scale(record: &Record) -> anyhow::Result<Amount> {
    let big = record.blinds().iter().copied().max().unwrap_or(0);
    match big.checked_div(Amount::from(pokerkit::B_BLIND)) {
        Some(scale) if scale > 0 && scale * Amount::from(pokerkit::B_BLIND) == big => Ok(scale),
        _ => bail!("blind {big} is not a whole multiple of the engine's {}", pokerkit::B_BLIND),
    }
}

/// Where the engine and the log disagreed, if anywhere.
#[derive(Debug, Clone, Default)]
pub struct Trace {
    decisions: usize,
    misordered: Vec<usize>,
    illegal: Vec<usize>,
    coerced: usize,
    stalled: Option<usize>,
    terminated: bool,
}

impl Trace {
    /// Decision nodes walked.
    pub fn decisions(&self) -> usize {
        self.decisions
    }

    /// Decision indices where the engine expected a different seat to act.
    pub fn misordered(&self) -> &[usize] {
        &self.misordered
    }

    /// Decision indices where the engine rejected the logged action outright.
    pub fn illegal(&self) -> &[usize] {
        &self.illegal
    }

    /// Raises whose amount `snap` had to move — chip-scale rounding, expected.
    pub fn coerced(&self) -> usize {
        self.coerced
    }

    /// Decision index at which the engine and the log lost step without either
    /// disagreeing outright — the engine was not at a chance node when the log
    /// dealt, or the deal was rejected.
    pub fn stalled(&self) -> Option<usize> {
        self.stalled
    }

    /// Whether the walk consumed the whole log and reached a terminal node.
    pub fn terminated(&self) -> bool {
        self.terminated
    }

    /// Nothing structural disagreed.
    pub fn clean(&self) -> bool {
        self.misordered.is_empty() && self.illegal.is_empty() && self.stalled.is_none() && self.terminated
    }
}

/// Walks `record` through a `P`-seat engine game.
///
/// Seat `i` in the log is position `i` in the engine, and the button is the
/// last seat — which puts the log's small blind at `dealer + 1`, matching the
/// engine's posting order.
///
/// Takes the [`Outcome`] as well because all-in is a fact about the log's chip
/// scale, not the engine's: 8,975 of 10,000 chips is a shove, but it divides to
/// 179.5 engine chips and truncating that leaves a phantom half-chip behind.
/// [`Node::is_shove`] settles the question before any rounding happens.
pub fn walk<const P: usize>(record: &Record, outcome: &Outcome) -> anyhow::Result<Trace> {
    if record.n() != P {
        bail!("record seats {} does not match engine seats {P}", record.n());
    }
    let scale = scale(record)?;
    let stacks = record
        .starting_stacks()
        .iter()
        .map(|s| Chips::try_from(s / scale))
        .collect::<Result<Vec<_>, _>>()?;
    let mut game = record
        .holes()
        .into_iter()
        .enumerate()
        .filter_map(|(seat, hole)| hole.map(|h| (seat, h)))
        .fold(GameN::<P>::from_start(P - 1, std::array::from_fn(|i| stacks[i])), |game, (seat, hole)| {
            game.deal(seat, hole)
        });
    let mut trace = Trace::default();
    let mut nodes = outcome.nodes().iter();
    let mut board = record.board().into_iter();
    for step in record.actions() {
        match step {
            Step::Board(cards) => {
                if !advance(&mut game, cards.len(), &mut board) {
                    trace.stalled = Some(trace.decisions);
                    return Ok(trace);
                }
            }
            Step::Fold(seat) | Step::CheckCall(seat) | Step::BetTo(seat, _) => {
                let index = trace.decisions;
                trace.decisions += 1;
                let Some(node) = nodes.next() else {
                    trace.stalled = Some(index);
                    return Ok(trace);
                };
                if game.turn() != Turn::Choice(*seat) {
                    trace.misordered.push(index);
                    return Ok(trace);
                }
                let wanted = intent(&game, node, scale);
                let action = game.snap(wanted);
                trace.coerced += usize::from(action != wanted);
                let Ok(next) = game.try_apply(action) else {
                    trace.illegal.push(index);
                    return Ok(trace);
                };
                game = next;
            }
            _ => {}
        }
    }
    trace.terminated = game.turn() == Turn::Terminal;
    Ok(trace)
}

/// Translates a logged decision into the action the engine should see.
///
/// All-in is read off the log rather than recomputed after the divide, so a
/// shove stays a shove no matter how the chips round.
pub(crate) fn intent<const P: usize>(game: &GameN<P>, node: &Node, scale: Amount) -> Action {
    match node.step() {
        Step::Fold(_) => Action::Fold,
        _ if node.is_shove() => Action::Shove(game.to_shove()),
        Step::CheckCall(_) if game.to_call() == 0 => Action::Check,
        Step::CheckCall(_) => Action::Call(game.to_call()),
        Step::BetTo(_, _) => match Chips::try_from(node.put() / scale) {
            Ok(chips) if chips < game.to_shove() => Action::Raise(chips),
            _ => Action::Shove(game.to_shove()),
        },
        _ => Action::Fold,
    }
}

/// Feeds the engine chance nodes until it has the same board the log does.
///
/// Returns false if the engine was not at a chance node when the log dealt, or
/// if the deal was rejected — both structural disagreements the caller reports.
fn advance<const P: usize>(game: &mut GameN<P>, cards: usize, board: &mut impl Iterator<Item = Card>) -> bool {
    let dealt = board.by_ref().take(cards).collect::<Vec<_>>();
    if dealt.len() != cards || game.turn() != Turn::Chance {
        return false;
    }
    match game.try_apply(Action::Draw(Hand::from(dealt))) {
        Ok(next) => {
            *game = next;
            true
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEAL: &str = "'d dh p1 TcQc', 'd dh p2 8s4c', 'd dh p3 9c3d', \
                        'd dh p4 Ah4h', 'd dh p5 Th5s', 'd dh p6 6c7s'";

    fn record(actions: &str) -> Record {
        let text = format!(
            "variant = 'NT'\nantes = [0, 0, 0, 0, 0, 0]\n\
             blinds_or_straddles = [50, 100, 0, 0, 0, 0]\nmin_bet = 100\n\
             starting_stacks = [10000, 10000, 10000, 10000, 10000, 10000]\n\
             actions = [{DEAL}, {actions}]\n"
        );
        crate::parse(&text, "test").unwrap()
    }

    #[test]
    fn scale_matches_the_engines_blind() {
        assert_eq!(scale(&record("'p3 f'")).unwrap(), Amount::from(50 / pokerkit::S_BLIND));
    }

    fn trace(actions: &str) -> Trace {
        let record = record(actions);
        let outcome = crate::replay(&record).unwrap();
        walk::<6>(&record, &outcome).unwrap()
    }

    #[test]
    fn a_fold_around_walks_clean() {
        let trace = trace("'p3 f', 'p4 f', 'p5 f', 'p6 f', 'p1 f'");
        assert_eq!(trace.decisions(), 5);
        assert!(trace.misordered().is_empty());
        assert!(trace.illegal().is_empty());
        assert!(trace.terminated());
    }

    #[test]
    fn a_skipped_seat_is_caught() {
        let trace = trace("'p4 f'");
        assert_eq!(trace.misordered(), [0]);
        assert!(!trace.clean());
    }

    #[test]
    fn an_all_in_survives_the_chip_divide() {
        // 8,975 chips is a shove but divides to 179.5 engine chips; truncating
        // would leave the shover with a phantom half chip still behind.
        let trace = trace(
            "'p3 f', 'p4 f', 'p5 f', 'p6 cbr 225', 'p1 cbr 1025', 'p2 f', \
             'p6 cbr 2675', 'p1 cbr 10000', 'p6 cc', \
             'd db Kd2h7d', 'd db Js', 'd db 3s'",
        );
        assert_eq!(trace.stalled(), None);
        assert!(trace.clean());
    }
}
