//! Chip-exact replay of a [`Record`] at the file's native chip scale.
//!
//! This is the conformance oracle. It is deliberately *independent* of
//! `kicker`: `pokerkit::Chips = i16` cannot hold a six-way all-in pot, and the
//! engine's blinds are compile-time constants at 1/50 of Pluribus's scale. So
//! the arithmetic here is its own — which is what makes it useful as a
//! reference implementation to diff `kicker::FunTable` against when multiway
//! parity lands.
//!
//! Settlement runs in **half-chips** ([`Halves`]) so that a two-way split of an
//! odd pot is exact rather than rounded; the Pluribus corpus contains eight
//! such hands. A split that is still inexact in half-chips sets
//! [`Outcome::odd_chips`] instead of silently rounding.

use crate::*;
use anyhow::Context;
use anyhow::bail;
use deuce::*;

/// A decision node, capturing the state *before* the logged action.
///
/// Everything Tier 1 (abstraction calibration) and Tier 2 (policy agreement)
/// need about a betting decision, in the engine's own accounting terms.
#[derive(Debug, Clone)]
pub struct Node {
    seat: usize,
    street: Street,
    depth: usize,
    pot: Amount,
    stake: Amount,
    top: Amount,
    stack: Amount,
    alive: usize,
    step: Step,
}

impl Node {
    pub fn seat(&self) -> usize {
        self.seat
    }

    pub fn street(&self) -> Street {
        self.street
    }
    /// Raises already made on this street — the `depth` axis of the bet grid.
    pub fn depth(&self) -> usize {
        self.depth
    }
    /// Total chips in the middle before this action, current street included.
    /// Matches `kicker::GameN::pot`.
    pub fn pot(&self) -> Amount {
        self.pot
    }
    /// The actor's wager already on this street.
    pub fn stake(&self) -> Amount {
        self.stake
    }
    /// The largest wager on this street.
    pub fn top(&self) -> Amount {
        self.top
    }
    /// The actor's chips behind, before acting.
    pub fn stack(&self) -> Amount {
        self.stack
    }
    /// Players not yet folded, this actor included.
    pub fn alive(&self) -> usize {
        self.alive
    }
    /// What the player actually did.
    pub fn step(&self) -> &Step {
        &self.step
    }

    pub fn to_call(&self) -> Amount {
        self.top - self.stake
    }
    /// Chips the actor puts in now. This is the quantity `kicker` denominates
    /// bet sizes against: `Action::Raise(chips)` is an increment from the
    /// actor's current stake, not a to-amount.
    pub fn put(&self) -> Amount {
        match self.step {
            Step::BetTo(_, to) => to - self.stake,
            Step::CheckCall(_) => self.to_call(),
            _ => 0,
        }
    }
    /// True when the action commits the actor's last chip.
    pub fn is_shove(&self) -> bool {
        self.put() >= self.stack
    }
    /// Raise size on the engine's pot-fraction axis: chips in now over the pot
    /// before the action. Mirrors `Size::translate`'s `Grid::Postflop` branch.
    pub fn pot_fraction(&self) -> f64 {
        f64::from(self.put()) / f64::from(self.pot.max(1))
    }
    /// Raise size on the engine's BB axis, used for preflop opens. Mirrors
    /// `Size::translate`'s `Grid::Opening` branch, where `Edge::Open(n)` puts
    /// in `n` big blinds.
    pub fn bb_units(&self, bblind: Amount) -> f64 {
        f64::from(self.put()) / f64::from(bblind.max(1))
    }
}

/// Result of replaying one hand.
#[derive(Debug, Clone)]
pub struct Outcome {
    finishing: Vec<Halves>,
    nodes: Vec<Node>,
    board: Vec<Card>,
    flopped: Vec<usize>,
    showdown: bool,
    odd_chips: bool,
}

impl Outcome {
    /// Per-seat stacks after settlement, in half-chips.
    pub fn finishing(&self) -> &[Halves] {
        &self.finishing
    }
    /// Decision nodes in the order they occurred.
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn board(&self) -> &[Card] {
        &self.board
    }
    /// Seats still in the hand when the flop was dealt. Empty if the hand
    /// ended preflop.
    pub fn flopped(&self) -> &[usize] {
        &self.flopped
    }
    /// True when two or more players were live at the end of the river.
    pub fn showdown(&self) -> bool {
        self.showdown
    }
    /// True when a pot could not be split evenly even in half-chips, so the
    /// remainder was awarded to the earliest seat.
    pub fn odd_chips(&self) -> bool {
        self.odd_chips
    }
    /// Whether the replay reproduces the logged `finishing_stacks` exactly.
    pub fn agrees_with(&self, record: &Record) -> bool {
        let logged = record.finishing_stacks();
        logged.len() == self.finishing.len() && logged.iter().zip(self.finishing.iter()).all(|(l, f)| l == f)
    }
}

/// Mutable view of the chip accounting, so the wager rule lives in one place.
struct Table<'a> {
    stacks: &'a mut Vec<Amount>,
    committed: &'a mut Vec<Amount>,
    stakes: &'a mut Vec<Amount>,
    pot: &'a mut Amount,
}

impl Table<'_> {
    /// Moves `amount` (capped at the seat's stack) from stack to pot.
    fn wager(&mut self, seat: usize, amount: Amount) {
        let amount = amount.clamp(0, self.stacks[seat]);
        self.stacks[seat] -= amount;
        self.committed[seat] += amount;
        self.stakes[seat] += amount;
        *self.pot += amount;
    }
}

/// Replays a hand and settles the pot.
///
/// Applies the logged actions rather than deriving turn order — a mismatch in
/// the result is then unambiguously an arithmetic or settlement disagreement.
/// Turn order is validated separately by [`turn_order`].
pub fn replay(record: &Record) -> anyhow::Result<Outcome> {
    if record.variant() != "NT" {
        bail!("variant {} is not no-limit hold'em", record.variant());
    }
    let n = record.n();
    let mut stacks = record.starting_stacks().to_vec();
    let mut committed = vec![0 as Amount; n];
    let mut stakes = vec![0 as Amount; n];
    let mut folded = vec![false; n];
    let mut mucked = vec![false; n];
    let mut holes = vec![None::<Hole>; n];
    let mut board = Vec::<Card>::new();
    let mut flopped = Vec::<usize>::new();
    let mut nodes = Vec::<Node>::new();
    let mut pot = 0 as Amount;
    let mut street = Street::Pref;
    let mut depth = 0usize;

    let mut table = Table {
        stacks: &mut stacks,
        committed: &mut committed,
        stakes: &mut stakes,
        pot: &mut pot,
    };
    for (seat, ante) in record.antes().iter().enumerate() {
        table.wager(seat, *ante);
        // Antes are dead money, not a street wager to be matched.
        table.stakes[seat] = 0;
    }
    for (seat, blind) in record.blinds().iter().enumerate() {
        table.wager(seat, *blind);
    }

    for step in record.actions() {
        match step {
            Step::Deal(seat, hole) | Step::Show(seat, hole) => holes[*seat] = Some(*hole),
            Step::Muck(seat) => mucked[*seat] = true,
            Step::Board(cards) => {
                if board.is_empty() {
                    flopped = (0..n).filter(|i| !folded[*i]).collect();
                }
                board.extend(cards.iter().copied());
                street = street_of(board.len())?;
                table.stakes.iter_mut().for_each(|s| *s = 0);
                depth = 0;
            }
            Step::Fold(seat) | Step::CheckCall(seat) | Step::BetTo(seat, _) => {
                let seat = *seat;
                let top = table.stakes.iter().copied().max().unwrap_or(0);
                nodes.push(Node {
                    seat,
                    street,
                    depth,
                    pot: *table.pot,
                    stake: table.stakes[seat],
                    top,
                    stack: table.stacks[seat],
                    alive: folded.iter().filter(|f| !**f).count(),
                    step: step.clone(),
                });
                match step {
                    Step::Fold(_) => folded[seat] = true,
                    Step::CheckCall(_) => table.wager(seat, top - table.stakes[seat]),
                    Step::BetTo(_, to) => {
                        if *to <= table.stakes[seat] {
                            bail!("`cbr {to}` does not exceed seat {seat}'s stake of {}", table.stakes[seat]);
                        }
                        table.wager(seat, to - table.stakes[seat]);
                        depth += 1;
                    }
                    _ => unreachable!("outer match restricts to decision steps"),
                }
            }
        }
    }

    let live = (0..n).filter(|i| !folded[*i]).count();
    let (payouts, odd_chips) = settle(&committed, &folded, &mucked, &holes, &board)?;
    let finishing = (0..n).map(|i| Halves::from(stacks[i]) * 2 + payouts[i]).collect();
    Ok(Outcome {
        finishing,
        nodes,
        board,
        flopped,
        showdown: live > 1,
        odd_chips,
    })
}

/// Awards the pot, layering side pots by commitment level.
///
/// Returns per-seat payouts in half-chips, and whether any layer failed to
/// divide evenly among its winners.
fn settle(
    committed: &[Amount],
    folded: &[bool],
    mucked: &[bool],
    holes: &[Option<Hole>],
    board: &[Card],
) -> anyhow::Result<(Vec<Halves>, bool)> {
    let n = committed.len();
    let mut payouts = vec![0 as Halves; n];
    let mut odd = false;
    // A muck forfeits, but never to the point of leaving the pot unclaimed.
    let forfeits = |i: usize| folded[i] || mucked[i];
    let contends: Vec<usize> = match (0..n).filter(|i| !forfeits(*i)).count() {
        0 => (0..n).filter(|i| !folded[*i]).collect(),
        _ => (0..n).filter(|i| !forfeits(*i)).collect(),
    };
    if contends.is_empty() {
        bail!("no contenders for the pot");
    }
    let strengths = contends
        .iter()
        .map(|&i| {
            let hole = holes[i].with_context(|| format!("seat {i} reached showdown without hole cards"))?;
            Ok((i, Strength::from(Hand::add(Hand::from(hole), Hand::from(board.to_vec())))))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let mut levels = committed.to_vec();
    levels.sort_unstable();
    levels.dedup();
    let mut floor = 0 as Amount;
    for level in levels {
        if level <= floor {
            continue;
        }
        let layer: Halves = committed
            .iter()
            .map(|c| Halves::from((*c).min(level) - floor.min(*c)).max(0) * 2)
            .sum();
        let eligible: Vec<(usize, Strength)> = strengths
            .iter()
            .copied()
            .filter(|(i, _)| committed[*i] >= level)
            .collect();
        floor = level;
        if layer == 0 {
            continue;
        }
        let Some(best) = eligible.iter().map(|(_, s)| *s).max() else {
            // Only forfeiting seats reached this level; their chips are dead.
            // Cannot happen in a well-formed log, but the pot must balance.
            bail!("side pot layer at {level} has no eligible claimant");
        };
        let winners: Vec<usize> = eligible.iter().filter(|(_, s)| *s == best).map(|(i, _)| *i).collect();
        let share = layer / winners.len() as Halves;
        let remainder = layer % winners.len() as Halves;
        odd |= remainder != 0;
        for (rank, seat) in winners.iter().enumerate() {
            payouts[*seat] += share + Halves::from((rank as Halves) < remainder);
        }
    }
    Ok((payouts, odd))
}

fn street_of(cards: usize) -> anyhow::Result<Street> {
    match cards {
        0 => Ok(Street::Pref),
        3 => Ok(Street::Flop),
        4 => Ok(Street::Turn),
        5 => Ok(Street::Rive),
        n => bail!("{n} board cards is not a hold'em street"),
    }
}

/// Checks the logged action order against hold'em turn order.
///
/// Returns the index of the first out-of-order decision, or `None` if the log
/// is consistent. Kept separate from [`replay`] so that a settlement
/// disagreement and a turn-order disagreement never masquerade as each other.
pub fn turn_order(record: &Record) -> Option<usize> {
    let n = record.n();
    let mut folded = vec![false; n];
    let mut allin = vec![false; n];
    let mut stakes = vec![0 as Amount; n];
    let mut stacks = record.starting_stacks().to_vec();
    for (seat, blind) in record.blinds().iter().enumerate() {
        let posted = (*blind).min(stacks[seat]);
        stakes[seat] = posted;
        stacks[seat] -= posted;
    }
    // Preflop opens after the last forced blind; postflop opens left of the
    // button, which is the last seat. Heads-up inverts postflop: the big blind
    // acts first once the flop is out.
    let big = record.blinds().iter().rposition(|b| *b > 0).unwrap_or(0);
    let mut expect = next_live(big + 1, &folded, &allin, n);
    let mut index = 0usize;
    for step in record.actions() {
        match step {
            Step::Board(_) => {
                stakes.iter_mut().for_each(|s| *s = 0);
                expect = next_live(usize::from(n == 2), &folded, &allin, n);
            }
            Step::Fold(seat) | Step::CheckCall(seat) | Step::BetTo(seat, _) => {
                if *seat != expect {
                    return Some(index);
                }
                let top = stakes.iter().copied().max().unwrap_or(0);
                match step {
                    Step::Fold(_) => folded[*seat] = true,
                    Step::CheckCall(_) => {
                        let put = (top - stakes[*seat]).min(stacks[*seat]);
                        stakes[*seat] += put;
                        stacks[*seat] -= put;
                    }
                    Step::BetTo(_, to) => {
                        let put = (*to - stakes[*seat]).min(stacks[*seat]);
                        stakes[*seat] += put;
                        stacks[*seat] -= put;
                    }
                    _ => {}
                }
                allin[*seat] = stacks[*seat] == 0;
                index += 1;
                expect = next_live(*seat + 1, &folded, &allin, n);
            }
            _ => {}
        }
    }
    None
}

fn next_live(from: usize, folded: &[bool], allin: &[bool], n: usize) -> usize {
    (from..from + n)
        .map(|i| i % n)
        .find(|i| !folded[*i] && !allin[*i])
        .unwrap_or(from % n)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(actions: &str, finishing: &str) -> Record {
        let text = format!(
            "variant = 'NT'\nantes = [0, 0, 0, 0, 0, 0]\n\
             blinds_or_straddles = [50, 100, 0, 0, 0, 0]\nmin_bet = 100\n\
             starting_stacks = [10000, 10000, 10000, 10000, 10000, 10000]\n\
             actions = [{actions}]\nfinishing_stacks = [{finishing}]\n"
        );
        crate::parse(&text, "test").unwrap()
    }

    const DEAL: &str = "'d dh p1 TcQc', 'd dh p2 8s4c', 'd dh p3 9c3d', \
                        'd dh p4 Ah4h', 'd dh p5 Th5s', 'd dh p6 6c7s'";

    #[test]
    fn folds_around_award_the_blinds() {
        let r = record(
            &format!("{DEAL}, 'p3 f', 'p4 f', 'p5 f', 'p6 f', 'p1 f'"),
            "9950, 10050, 10000, 10000, 10000, 10000",
        );
        let outcome = replay(&r).unwrap();
        assert!(outcome.agrees_with(&r));
        assert!(!outcome.showdown());
        assert!(outcome.flopped().is_empty());
    }

    #[test]
    fn river_bet_takes_it_down() {
        let r = record(
            &format!(
                "{DEAL}, 'p3 f', 'p4 cbr 210', 'p5 f', 'p6 f', 'p1 cc', 'p2 f', \
                 'd db 7d5h9d', 'p1 cc', 'p4 cc', 'd db 7c', 'p1 cc', 'p4 cc', \
                 'd db Qh', 'p1 cbr 230', 'p4 f'"
            ),
            "10310, 9900, 10000, 9790, 10000, 10000",
        );
        let outcome = replay(&r).unwrap();
        assert!(outcome.agrees_with(&r));
        assert_eq!(outcome.flopped(), [0, 3]);
        assert_eq!(outcome.board().len(), 5);
    }

    #[test]
    fn nodes_use_the_engines_sizing_convention() {
        let r = record(
            &format!("{DEAL}, 'p3 f', 'p4 cbr 210', 'p5 f', 'p6 f', 'p1 f', 'p2 f'"),
            "9950, 9900, 10000, 10150, 10000, 10000",
        );
        let outcome = replay(&r).unwrap();
        let open = outcome.nodes().iter().find(|n| n.seat() == 3).unwrap();
        assert_eq!(open.street(), Street::Pref);
        assert_eq!(open.depth(), 0);
        assert_eq!(open.pot(), 150);
        assert_eq!(open.put(), 210);
        assert_eq!(open.bb_units(100), 2.1);
        assert!(!open.is_shove());
        assert!(outcome.agrees_with(&r));
    }

    #[test]
    fn turn_order_accepts_a_well_formed_log() {
        let r = record(
            &format!("{DEAL}, 'p3 f', 'p4 cbr 210', 'p5 f', 'p6 f', 'p1 cc', 'p2 f', 'd db 7d5h9d', 'p1 cc', 'p4 cc'"),
            "9790, 9900, 10000, 9790, 10000, 10000",
        );
        assert_eq!(turn_order(&r), None);
    }

    #[test]
    fn turn_order_catches_a_skipped_seat() {
        let r = record(&format!("{DEAL}, 'p4 f'"), "9950, 9900, 10000, 10000, 10000, 10000");
        assert_eq!(turn_order(&r), Some(0));
    }
}
