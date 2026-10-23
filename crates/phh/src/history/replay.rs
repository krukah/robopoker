//! Chip-exact replay of a [`Record`] at the file's native chip scale.
//!
//! This is the conformance oracle. It is deliberately *independent* of `kicker`:
//! `pokerkit::Chips = i16` cannot hold a six-way all-in pot, and the engine's blinds are
//! compile-time constants at 1/50 of Pluribus's scale.

use crate::*;
use anyhow::Context;
use anyhow::bail;
use deuce::*;

/// Namespace for the replay routines.
pub struct Replay;

impl Replay {
    /// Replays a hand and settles the pot.
    ///
    /// Applies the logged actions rather than deriving turn order — a mismatch in
    /// the result is then unambiguously an arithmetic or settlement disagreement.
    /// Turn order is validated separately by [`Replay::turn_order`].
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

        let mut table = Table::new(&mut stacks, &mut committed, &mut stakes, &mut pot);
        for (seat, ante) in record.antes().iter().enumerate() {
            table.wager(seat, *ante);
            // Antes are dead money, not a street wager to be matched.
            table.stakes_mut()[seat] = 0;
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
                    street = Replay::street_of(board.len())?;
                    table.stakes_mut().iter_mut().for_each(|s| *s = 0);
                    depth = 0;
                }
                Step::Fold(seat) | Step::CheckCall(seat) | Step::BetTo(seat, _) => {
                    let seat = *seat;
                    let top = table.stakes().iter().copied().max().unwrap_or(0);
                    nodes.push(Node::new(
                        seat,
                        street,
                        depth,
                        table.pot(),
                        table.stakes()[seat],
                        top,
                        table.stacks()[seat],
                        folded.iter().filter(|f| !**f).count(),
                        step.clone(),
                    ));
                    match step {
                        Step::Fold(_) => folded[seat] = true,
                        Step::CheckCall(_) => table.wager(seat, top - table.stakes()[seat]),
                        Step::BetTo(_, to) => {
                            if *to <= table.stakes()[seat] {
                                bail!("`cbr {to}` does not exceed seat {seat}'s stake of {}", table.stakes()[seat]);
                            }
                            table.wager(seat, to - table.stakes()[seat]);
                            depth += 1;
                        }
                        _ => unreachable!("outer match restricts to decision steps"),
                    }
                }
            }
        }

        let live = (0..n).filter(|i| !folded[*i]).count();
        let (payouts, odd_chips) = Replay::settle(&committed, &folded, &mucked, &holes, &board)?;
        let finishing = (0..n).map(|i| Halves::from(stacks[i]) * 2 + payouts[i]).collect();
        Ok(Outcome::new(finishing, nodes, board, flopped, live > 1, odd_chips))
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
            let layer = committed
                .iter()
                .map(|c| Halves::from((*c).min(level) - floor.min(*c)).max(0) * 2)
                .sum::<Halves>();
            let eligible = strengths
                .iter()
                .copied()
                .filter(|(i, _)| committed[*i] >= level)
                .collect::<Vec<(usize, Strength)>>();
            floor = level;
            if layer == 0 {
                continue;
            }
            let Some(best) = eligible.iter().map(|(_, s)| *s).max() else {
                // Only forfeiting seats reached this level; their chips are dead.
                // Cannot happen in a well-formed log, but the pot must balance.
                bail!("side pot layer at {level} has no eligible claimant");
            };
            let winners = eligible
                .iter()
                .filter(|(_, s)| *s == best)
                .map(|(i, _)| *i)
                .collect::<Vec<usize>>();
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
    /// is consistent. Kept separate from [`Replay::replay`] so that a settlement
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
        // Preflop opens after the last forced blind; postflop opens left of the button, which is
        // the last seat. Heads-up inverts postflop: the big blind acts first once the flop is out.
        let big = record.blinds().iter().rposition(|b| *b > 0).unwrap_or(0);
        let mut expect = Replay::next_live(big + 1, &folded, &allin, n);
        let mut index = 0usize;
        for step in record.actions() {
            match step {
                Step::Board(_) => {
                    stakes.iter_mut().for_each(|s| *s = 0);
                    expect = Replay::next_live(usize::from(n == 2), &folded, &allin, n);
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
                    expect = Replay::next_live(*seat + 1, &folded, &allin, n);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Replay {
        fn record(actions: &str, finishing: &str) -> Record {
            let text = format!(
                "variant = 'NT'\nantes = [0, 0, 0, 0, 0, 0]\n\
                 blinds_or_straddles = [50, 100, 0, 0, 0, 0]\nmin_bet = 100\n\
                 starting_stacks = [10000, 10000, 10000, 10000, 10000, 10000]\n\
                 actions = [{actions}]\nfinishing_stacks = [{finishing}]\n"
            );
            crate::Parse::parse(&text, "test").unwrap()
        }
    }

    const DEAL: &str = "'d dh p1 TcQc', 'd dh p2 8s4c', 'd dh p3 9c3d', \
                        'd dh p4 Ah4h', 'd dh p5 Th5s', 'd dh p6 6c7s'";

    #[test]
    fn folds_around_award_the_blinds() {
        let r = Replay::record(
            &format!("{DEAL}, 'p3 f', 'p4 f', 'p5 f', 'p6 f', 'p1 f'"),
            "9950, 10050, 10000, 10000, 10000, 10000",
        );
        let outcome = Replay::replay(&r).unwrap();
        assert!(outcome.agrees_with(&r));
        assert!(!outcome.showdown());
        assert!(outcome.flopped().is_empty());
    }

    #[test]
    fn river_bet_takes_it_down() {
        let r = Replay::record(
            &format!(
                "{DEAL}, 'p3 f', 'p4 cbr 210', 'p5 f', 'p6 f', 'p1 cc', 'p2 f', \
                 'd db 7d5h9d', 'p1 cc', 'p4 cc', 'd db 7c', 'p1 cc', 'p4 cc', \
                 'd db Qh', 'p1 cbr 230', 'p4 f'"
            ),
            "10310, 9900, 10000, 9790, 10000, 10000",
        );
        let outcome = Replay::replay(&r).unwrap();
        assert!(outcome.agrees_with(&r));
        assert_eq!(outcome.flopped(), [0, 3]);
        assert_eq!(outcome.board().len(), 5);
    }

    #[test]
    fn nodes_use_the_engines_sizing_convention() {
        let r = Replay::record(
            &format!("{DEAL}, 'p3 f', 'p4 cbr 210', 'p5 f', 'p6 f', 'p1 f', 'p2 f'"),
            "9950, 9900, 10000, 10150, 10000, 10000",
        );
        let outcome = Replay::replay(&r).unwrap();
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
        let r = Replay::record(
            &format!("{DEAL}, 'p3 f', 'p4 cbr 210', 'p5 f', 'p6 f', 'p1 cc', 'p2 f', 'd db 7d5h9d', 'p1 cc', 'p4 cc'"),
            "9790, 9900, 10000, 9790, 10000, 10000",
        );
        assert_eq!(Replay::turn_order(&r), None);
    }

    #[test]
    fn turn_order_catches_a_skipped_seat() {
        let r = Replay::record(&format!("{DEAL}, 'p4 f'"), "9950, 9900, 10000, 10000, 10000, 10000");
        assert_eq!(Replay::turn_order(&r), Some(0));
    }
}
