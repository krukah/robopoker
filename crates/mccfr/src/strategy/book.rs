use crate::*;
use rbp_core::*;
use std::collections::HashMap;

/// Accessor trait for HashMap-based CFR storage.
///
/// Games that store encounters as `HashMap<I, HashMap<E, Encounter>>` with a
/// `usize` epoch counter can implement this single trait to receive blanket
/// impls for [`CfrRule`], [`Storage`], [`Profile`], and [`CfrSampling`].
///
/// Edge-specific warmstart defaults are wired through [`CfrEdge::default_policy`]
/// and [`CfrEdge::default_regret`], allowing games like NLHE to bias initial
/// strategies without overriding Storage/Profile manually.
pub trait CfrData: Sized {
    type T: CfrTurn;
    type E: CfrEdge;
    type G: CfrGame<E = Self::E, T = Self::T>;
    type I: CfrInfo<E = Self::E, T = Self::T>;
    fn encounters_ref(&self) -> &HashMap<Self::I, HashMap<Self::E, Encounter>>;
    fn encounters_mut(&mut self) -> &mut HashMap<Self::I, HashMap<Self::E, Encounter>>;
    fn epochs_ref(&self) -> usize;
    fn epochs_mut(&mut self) -> &mut usize;
    fn store_metrics(&self) -> Option<&Metrics>;
}

impl<P> CfrRule for P
where
    P: CfrData,
{
    type T = <P as CfrData>::T;
    type E = <P as CfrData>::E;
    type G = <P as CfrData>::G;
    type I = <P as CfrData>::I;
}

impl<P> MutProf for P
where
    P: CfrData,
{
    fn mut_weight(&mut self, info: &Self::I, edge: &Self::E) -> &mut Probability {
        &mut self
            .encounters_mut()
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_insert_with(|| Encounter::from(edge))
            .weight
    }

    fn mut_regret(&mut self, info: &Self::I, edge: &Self::E) -> &mut Utility {
        &mut self
            .encounters_mut()
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_insert_with(|| Encounter::from(edge))
            .regret
    }

    fn mut_payoff(&mut self, info: &Self::I, edge: &Self::E) -> &mut Utility {
        &mut self
            .encounters_mut()
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_insert_with(|| Encounter::from(edge))
            .payoff
    }

    fn mut_visits(&mut self, info: &Self::I, edge: &Self::E) -> &mut u32 {
        &mut self
            .encounters_mut()
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_insert_with(|| Encounter::from(edge))
            .visits
    }
}

impl<P> RefProf for P
where
    P: CfrData,
{
    fn t(&self) -> usize {
        self.epochs_ref()
    }

    fn metrics(&self) -> Option<&Metrics> {
        self.store_metrics()
    }

    fn cum_weight(&self, info: &Self::I, edge: &Self::E) -> Probability {
        self.encounters_ref()
            .get(info)
            .and_then(|memory| memory.get(edge))
            .map(|e| e.weight)
            .unwrap_or_default()
    }

    fn cum_regret(&self, info: &Self::I, edge: &Self::E) -> Utility {
        self.encounters_ref()
            .get(info)
            .and_then(|memory| memory.get(edge))
            .map(|e| e.regret)
            .unwrap_or_else(|| edge.default_regret())
    }

    fn cum_payoff(&self, info: &Self::I, edge: &Self::E) -> Utility {
        self.encounters_ref()
            .get(info)
            .and_then(|memory| memory.get(edge))
            .map(|e| e.payoff)
            .unwrap_or_default()
    }

    fn cum_visits(&self, info: &Self::I, edge: &Self::E) -> u32 {
        self.encounters_ref()
            .get(info)
            .and_then(|memory| memory.get(edge))
            .map(|e| e.visits)
            .unwrap_or_default()
    }

    fn sum_regret(&self) -> Utility {
        self.encounters_ref()
            .values()
            .flat_map(|edges| edges.values())
            .map(|e| e.regret.max(0.))
            .sum::<Utility>()
            / self.epochs_ref().max(1) as Utility
    }
}

impl<P> CfrSampling for P
where
    P: CfrData,
{
    fn increment(&mut self) {
        *self.epochs_mut() += 1;
    }

    fn walker(&self) -> Self::T {
        Self::T::from(self.epochs_ref() % Self::T::players())
    }
}
