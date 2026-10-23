use super::*;
use kicker::Action;
use pokerkit::*;

/// Namespace for the metrics routines.
pub struct Metrics;

impl Metrics {
    /// Build a summary from a sequence of hand recaps.
    pub fn summarize(recaps: &[Recap]) -> Summary {
        Summary {
            hands: recaps.len(),
            won: recaps.iter().map(Recap::won).sum(),
            total_pot: recaps.iter().map(|r| r.pot() as i64).sum(),
            series: recaps.iter().map(Recap::won).collect(),
            vpip: recaps.iter().filter(|r| r.vpip()).count(),
            pfr: recaps.iter().filter(|r| r.pfr()).count(),
            showdowns: recaps.iter().filter(|r| r.showdown()).count(),
            showdown_wins: recaps.iter().filter(|r| r.showdown() && r.won() > 0).count(),
            folds: Self::count(recaps, |a| matches!(a, Action::Fold)),
            checks: Self::count(recaps, |a| matches!(a, Action::Check)),
            calls: Self::count(recaps, |a| matches!(a, Action::Call(_))),
            raises: Self::count(recaps, |a| matches!(a, Action::Raise(_) | Action::Shove(_))),
            ..Summary::default()
        }
    }

    /// Build a summary from (pnl, pot) pairs without replay.
    pub fn summarize_pnl(rows: &[(Chips, Chips)]) -> Summary {
        Summary {
            hands: rows.len(),
            won: rows.iter().map(|&(pnl, _)| pnl).sum(),
            total_pot: rows.iter().map(|&(_, pot)| pot as i64).sum(),
            series: rows.iter().map(|&(pnl, _)| pnl).collect(),
            ..Summary::default()
        }
    }

    fn count<F>(recaps: &[Recap], kind: F) -> usize
    where
        F: Fn(&Action) -> bool,
    {
        recaps
            .iter()
            .flat_map(Recap::actions)
            .filter(|(_, action)| kind(action))
            .count()
    }
}
