//! Standing a heads-up preflop in for a multiway one.
//!
//! [`crate::reduce`] establishes that a hand's postflop *shape* is ours. Its
//! preflop is not: a 6-max betting round has folds, callers and dead blinds
//! that have no heads-up analogue, and our blueprint's information sets are
//! keyed on the edge path that led to the node. So the preflop has to be
//! replaced, not translated.
//!
//! What the replacement has to preserve is what the postflop decision actually
//! depends on: **who holds the betting lead**, then **how big the pot is**,
//! and only then how contested it got. A projection that matches the pot but
//! hands the lead to the wrong player has changed the spot outright. Pot size
//! outranks raise count because it is what postflop play actually keys on —
//! through the stack-to-pot ratio — and because raise count is only ever a
//! proxy for it. A 40bb single-raised pot is a better fit for a heads-up 3-bet
//! path than for the biggest single-raise path our grid can build.
//!
//! Candidates are enumerated from the live tree rather than written down, so
//! the projection tracks `OPENS` / `PLURIBUS_INDICES` as they change.

use crate::*;
use deuce::*;
use kicker::*;
use pokerkit::*;

/// A heads-up preflop path standing in for a multiway one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Projection {
    actions: Vec<Action>,
    pot: Chips,
    target: Chips,
    raises: usize,
    aggressor: Option<Turn>,
}

impl Projection {
    /// The substituted preflop, ready to prefix a [`Witness`]'s action list.
    pub fn actions(&self) -> &[Action] {
        &self.actions
    }

    /// Pot at the flop, in engine chips.
    pub fn pot(&self) -> Chips {
        self.pot
    }

    /// Pot we were aiming for, in engine chips.
    pub fn target(&self) -> Chips {
        self.target
    }

    /// Raises along this path. Stands in for how contested the pot got.
    pub fn raises(&self) -> usize {
        self.raises
    }

    /// Who took the last preflop aggression, if anyone.
    pub fn aggressor(&self) -> Option<Turn> {
        self.aggressor
    }

    /// Pot error as a fraction of the pot being stood in for.
    ///
    /// The honest measure of the substitution: postflop play keys on the
    /// stack-to-pot ratio, so a 2bb miss on a 40bb pot is a far smaller lie
    /// than a 2bb miss on a 5bb one.
    ///
    /// Part of the residual is dead money, which our pot structurally cannot
    /// hold: a folded blind sits in the real pot, and both our players are
    /// live. The rest is the grid's own granularity.
    ///
    /// This is a diagnostic on the *projection*, not a filter on spots. Our
    /// information sets are keyed on the current street's edges only, so a
    /// substituted preflop cannot reach a postflop lookup at all — see
    /// [`crate::Spot`]. Where the pot does matter is [`crate::Ledger`], which
    /// plays chips out from it.
    pub fn skew(&self) -> f64 {
        f64::from(self.pot - self.target) / f64::from(self.target.max(1))
    }
}

/// Picks the heads-up preflop that best stands in for `reduction`.
///
/// The in-position seat plays the button (`Choice(0)`, which posts the small
/// blind and acts last postflop) and the out-of-position seat plays the big
/// blind, matching the ordering [`crate::shape`] established.
pub fn project(reduction: &Reduction, bblind: Amount) -> Option<Projection> {
    let target = Chips::try_from(i64::from(reduction.pot()) * i64::from(B_BLIND) / i64::from(bblind.max(1))).ok()?;
    let aggressor = reduction.aggressor().map(|seat| match seat {
        seat if seat == reduction.seats().ip() => Turn::Choice(0),
        _ => Turn::Choice(1),
    });
    // Lead first, size second, contest last — see the module docs.
    candidates()
        .iter()
        .min_by_key(|c| {
            (
                usize::from(c.aggressor != aggressor),
                (c.pot - target).unsigned_abs(),
                c.raises.abs_diff(reduction.depth()),
            )
        })
        .map(|c| Projection {
            actions: c.actions.clone(),
            pot: c.pot,
            target,
            raises: c.raises,
            aggressor: c.aggressor,
        })
}

/// Every heads-up preflop path that reaches a flop, walked off the live grid.
fn candidates() -> &'static [Projection] {
    static PATHS: std::sync::OnceLock<Vec<Projection>> = std::sync::OnceLock::new();
    PATHS.get_or_init(|| {
        let mut found = Vec::new();
        walk(Game::root(), Vec::new(), 0, None, &mut found);
        found
    })
}

/// Depth-first over preflop decisions, collecting the paths that see a flop.
///
/// Folds are skipped — a folded preflop never reaches the postflop decision we
/// are trying to reconstruct.
fn walk(game: Game, actions: Vec<Action>, raises: usize, aggressor: Option<Turn>, found: &mut Vec<Projection>) {
    match game.turn() {
        Turn::Chance => found.push(Projection {
            pot: game.pot(),
            target: 0,
            actions,
            raises,
            aggressor,
        }),
        Turn::Terminal => {}
        Turn::Choice(seat) => {
            let passive = if game.may_check() { Action::Check } else { Action::Call(game.to_call()) };
            if let Ok(next) = game.try_apply(passive) {
                let mut path = actions.clone();
                path.push(passive);
                walk(next, path, raises, aggressor, found);
            }
            for edge in Edge::raises(Street::Pref, raises) {
                let action = game.snap(game.actionize(edge));
                let Ok(next) = game.try_apply(action) else { continue };
                let mut path = actions.clone();
                path.push(action);
                walk(next, path, raises + 1, Some(Turn::Choice(seat)), found);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reduction(pot: Amount, depth: usize, ip_leads: bool) -> Reduction {
        // Seats 1 (oop) and 5 (ip) — a big blind versus a button, the corpus's
        // most common reducible shape.
        Reduction::sample(1, 5, pot, depth, ip_leads.then_some(5).or(Some(1)).filter(|_| depth > 0))
    }

    #[test]
    fn every_candidate_reaches_a_flop() {
        assert!(!candidates().is_empty());
        for candidate in candidates() {
            let game = candidate.actions.iter().fold(Game::root(), |g, a| g.apply(*a));
            assert_eq!(game.turn(), Turn::Chance, "path {:?} does not reach a flop", candidate.actions);
            assert_eq!(game.pot(), candidate.pot);
        }
    }

    #[test]
    fn a_limped_pot_projects_to_a_limp() {
        let p = project(&reduction(200, 0, false), 100).unwrap();
        assert_eq!(p.raises(), 0);
        assert_eq!(p.aggressor(), None);
        assert_eq!(p.actions(), [Action::Call(1), Action::Check]);
        assert_eq!(p.pot(), 4);
    }

    /// Button opens, big blind calls, 0.5bb of dead small blind left behind.
    /// Ours cannot hold that half blind, so the pot lands one chip light.
    #[test]
    fn a_button_open_projects_to_a_button_open() {
        let p = project(&reduction(500, 1, true), 100).unwrap();
        assert_eq!(p.raises(), 1);
        assert_eq!(p.aggressor(), Some(Turn::Choice(0)));
        assert_eq!(p.pot(), 10);
        assert_eq!(p.target(), 10);
        assert_eq!(p.skew(), 0.0);
    }

    /// The lead outranks the pot: a projection is allowed to miss the size, but
    /// not to hand the betting lead to the wrong player.
    #[test]
    fn the_lead_is_never_traded_for_a_closer_pot() {
        for pot in [300, 500, 900, 1600, 2500] {
            for lead in [true, false] {
                let p = project(&reduction(pot, 1, lead), 100).unwrap();
                let expect = if lead { Turn::Choice(0) } else { Turn::Choice(1) };
                assert_eq!(p.aggressor(), Some(expect), "pot {pot} lead {lead}");
            }
        }
    }

    /// Raise count is honoured when the pot does not argue otherwise: an 18bb
    /// pot led by the out-of-position player is exactly a heads-up 3-bet pot.
    #[test]
    fn a_three_bet_pot_keeps_its_raise_count() {
        let p = project(&reduction(1800, 2, false), 100).unwrap();
        assert_eq!(p.raises(), 2);
        assert_eq!(p.aggressor(), Some(Turn::Choice(1)));
        assert!(p.skew().abs() < 0.15, "skewed {:.1}%", 100.0 * p.skew());
    }

    /// A raised pot cannot be smaller than an open plus a call, so the sweep
    /// starts at 4.5bb — the button min-raising to 2bb, the big blind calling,
    /// and the small blind's half blind left dead.
    #[test]
    fn drift_stays_small_across_the_realistic_range() {
        let worst = (450..=4000)
            .step_by(50)
            .flat_map(|pot| [true, false].map(|lead| project(&reduction(pot, 1, lead), 100).unwrap()))
            .map(|p| (p.skew().abs() * 1000.0) as i64)
            .max()
            .unwrap();
        assert!(worst <= 350, "worst pot skew is {:.1}%", worst as f64 / 10.0);
    }

    /// The one shape heads-up genuinely cannot hold: a *small* pot led by the
    /// out-of-position player.
    ///
    /// In 6-max an early seat opens and a later one calls all the time, leaving
    /// a 5bb pot whose aggressor is out of position. Heads-up, the big blind's
    /// only route to the lead is a limp-raise, and that pot cannot be smaller
    /// than 6bb. Nothing to fix here — it is a property of the two games, so
    /// [`Projection::skew`] exists to let callers see it and weigh it.
    #[test]
    fn a_small_out_of_position_lead_has_no_heads_up_analogue() {
        let tight = project(&reduction(450, 1, true), 100).unwrap();
        let loose = project(&reduction(450, 1, false), 100).unwrap();
        assert!(tight.skew().abs() < 0.15, "in position fits: {:.1}%", 100.0 * tight.skew());
        assert!(loose.skew() > 0.25, "out of position cannot: {:.1}%", 100.0 * loose.skew());
        // The bigger the pot, the less the floor bites.
        let big = project(&reduction(2000, 1, false), 100).unwrap();
        assert!(big.skew().abs() < 0.10, "large pots fit either way: {:.1}%", 100.0 * big.skew());
    }
}
