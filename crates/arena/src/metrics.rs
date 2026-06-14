use super::*;
use croupier::Action;
use fulcrum::*;

/// Build a summary from a sequence of hand recaps.
pub fn summarize(recaps: &[Recap]) -> Summary {
    recaps.iter().fold(Summary::default(), |mut s, r| {
        s.hands += 1;
        s.won += r.won();
        s.total_pot += r.pot() as i64;
        s.series.push(r.won());
        s.vpip += r.vpip() as usize;
        s.pfr += r.pfr() as usize;
        s.showdowns += r.showdown() as usize;
        s.showdown_wins += (r.showdown() && r.won() > 0) as usize;
        for (_, action) in r.actions() {
            match action {
                Action::Fold => s.folds += 1,
                Action::Check => s.checks += 1,
                Action::Call(_) => s.calls += 1,
                Action::Raise(_) | Action::Shove(_) => s.raises += 1,
                _ => {}
            }
        }
        s
    })
}
/// Build a summary from (pnl, pot) pairs without replay.
pub fn summarize_pnl(rows: &[(Chips, Chips)]) -> Summary {
    rows.iter().fold(Summary::default(), |mut s, &(pnl, pot)| {
        s.hands += 1;
        s.won += pnl;
        s.total_pot += pot as i64;
        s.series.push(pnl);
        s
    })
}
