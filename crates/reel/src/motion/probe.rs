use crate::*;
use pokerkit::Utility;

/// Knuth's multiplicative-hash constant — spreads phase seeds apart.
pub const KNUTH: u64 = 2_654_435_761;

/// Gain when the hero folds — a booked loss.
pub const HERO_FOLD: Utility = -0.5;

/// Gain when the villain folds — the pot is taken down.
pub const VILLAIN_FOLD: Utility = 0.55;

/// Gain when the hero's terminal action is a shove.
pub const HERO_SHOVE: Utility = 0.3;

/// Gain when the villain's terminal action is a shove.
pub const VILLAIN_SHOVE: Utility = -0.35;

/// Gain when a call closes the line.
pub const CALLED: Utility = 0.2;

/// Gain of any other terminal action.
pub const RAISED: Utility = 0.15;

/// One exploration trajectory through a phase's subgame: the slot taken at
/// each decision layer, and the counterfactual gain its terminal booked.
#[derive(Debug, Clone)]
pub struct Probe {
    line: Vec<Slot>,
    gain: Utility,
}

impl Probe {
    /// Every distinct exploration line through phase `s` of the scene,
    /// ordered so consecutive probes open with different first actions:
    /// bucketed by opening slot, shuffled within each bucket by a generator
    /// seeded from the phase, round-robined across buckets. Deterministic —
    /// chosen, not random.
    pub fn ordered<G>(scene: &Scene<G>, s: usize) -> Vec<Self>
    where
        G: Scenic,
    {
        let stint = &scene.stints()[s];
        let mut buckets = std::collections::BTreeMap::<Slot, Vec<Vec<Slot>>>::new();
        Probe::lines(scene, s)
            .into_iter()
            .for_each(|line| buckets.entry(line[0]).or_default().push(line));
        let mut rnd = Lcg::new((s as u64).wrapping_mul(KNUTH).wrapping_add(INC));
        buckets.values_mut().for_each(|bucket| {
            (1..bucket.len())
                .rev()
                .for_each(|i| bucket.swap(i, (rnd.roll() * (i + 1) as f64) as usize));
        });
        let mut order = Vec::new();
        while buckets.values().any(|bucket| !bucket.is_empty()) {
            buckets
                .values_mut()
                .filter(|bucket| !bucket.is_empty())
                .for_each(|bucket| order.push(bucket.remove(0)));
        }
        order
            .into_iter()
            .map(|line| Self {
                gain: Probe::line_gain(scene, stint.moves(), &line),
                line,
            })
            .collect()
    }

    pub fn line(&self) -> &[Slot] {
        &self.line
    }

    pub fn gain(&self) -> Utility {
        self.gain
    }
}

impl Probe {
    /// Every distinct trajectory of depth 1..=N with no two consecutive
    /// identical edges, where a fold or call closes the line; the chosen line
    /// is excluded. Deterministic DFS order.
    fn lines<G>(scene: &Scene<G>, s: usize) -> Vec<Vec<Slot>>
    where
        G: Scenic,
    {
        let stint = &scene.stints()[s];
        let mut out = Vec::new();
        let mut pfx = Vec::new();
        Probe::walk(scene, stint.moves(), &mut pfx, &mut out);
        out.into_iter().filter(|line| *line != stint.picks()).collect()
    }

    fn walk<G>(scene: &Scene<G>, moves: &[usize], pfx: &mut Vec<Slot>, out: &mut Vec<Vec<Slot>>)
    where
        G: Scenic,
    {
        if let Some(&last) = pfx.last() {
            out.push(pfx.clone());
            if G::deed(&scene.menu(moves[pfx.len() - 1])[last.raw()]).ends() {
                return;
            }
        }
        if pfx.len() == moves.len() {
            return;
        }
        for (slot, edge) in scene.menu(moves[pfx.len()]).iter().enumerate() {
            if pfx
                .last()
                .is_some_and(|&p| scene.menu(moves[pfx.len() - 1])[p.raw()] == *edge)
            {
                continue;
            }
            pfx.push(Slot::new(slot));
            Probe::walk(scene, moves, pfx, out);
            pfx.pop();
        }
    }

    /// Deterministic counterfactual gain for a probe: the hero folding books a
    /// loss, the villain folding books a gain, terminal aggression in between.
    fn line_gain<G>(scene: &Scene<G>, moves: &[usize], line: &[Slot]) -> Utility
    where
        G: Scenic,
    {
        let layer = moves[line.len() - 1];
        let hero = scene.heroic(layer);
        match (G::deed(&scene.menu(layer)[line.last().expect("nonempty line").raw()]), hero) {
            (Deed::Fold, true) => HERO_FOLD,
            (Deed::Fold, false) => VILLAIN_FOLD,
            (Deed::Shove, true) => HERO_SHOVE,
            (Deed::Shove, false) => VILLAIN_SHOVE,
            (Deed::Call, _) => CALLED,
            _ => RAISED,
        }
    }
}
