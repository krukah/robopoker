use mccfr::*;
use pokerkit::*;
use std::collections::HashMap;

/// Local accumulators over a frozen view: a miss reads the view, and a first write warmstarts
/// from it. Every subgame profile is one of these, over the view that lifts its blueprint.
pub struct Overlay<V>
where
    V: Profile + Copy,
{
    view: V,
    local: HashMap<InfoOf<V>, HashMap<EdgeOf<V>, Encounter>>,
    t: usize,
}

impl<V> From<V> for Overlay<V>
where
    V: Profile + Copy,
{
    fn from(view: V) -> Self {
        Self {
            view,
            local: HashMap::new(),
            t: 0,
        }
    }
}

impl<V> CfrSpec for Overlay<V>
where
    V: Profile + Copy,
{
    type F = V::F;
}

impl<V> Storage for Overlay<V>
where
    V: Profile + Copy,
{
    fn mut_weight(&mut self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> &mut Probability {
        self.encounter(info, edge).weight_mut()
    }

    fn mut_regret(&mut self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> &mut Utility {
        self.encounter(info, edge).regret_mut()
    }

    fn mut_payoff(&mut self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> &mut Utility {
        self.encounter(info, edge).payoff_mut()
    }

    fn mut_visits(&mut self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> &mut u32 {
        self.encounter(info, edge).visits_mut()
    }
}

impl<V> Profile for Overlay<V>
where
    V: Profile + Copy,
{
    fn t(&self) -> usize {
        self.t
    }

    fn cum_weight(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Probability {
        self.cell(info, edge)
            .map_or_else(|| self.view().cum_weight(info, edge).max(EPSILON), Encounter::weight)
    }

    fn cum_regret(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Utility {
        self.cell(info, edge)
            .map_or_else(|| self.view().cum_regret(info, edge).max(EPSILON), Encounter::regret)
    }

    fn cum_payoff(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Utility {
        self.cell(info, edge)
            .map_or_else(|| self.view().cum_payoff(info, edge), Encounter::payoff)
    }

    fn cum_visits(&self, info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> u32 {
        self.cell(info, edge)
            .map_or_else(|| self.view().cum_visits(info, edge), Encounter::visits)
    }

    fn sum_regret(&self) -> Utility {
        self.local()
            .values()
            .flat_map(HashMap::values)
            .map(|e| e.regret().max(0.))
            .sum::<Utility>()
            / self.t().max(1) as Utility
    }
}

impl<V> Clock for Overlay<V>
where
    V: Profile + Clock + Copy,
{
    fn increment(&mut self) {
        *self.t_mut() += 1;
    }

    fn temperature(&self) -> Entropy {
        self.view().temperature()
    }

    fn smoothing(&self) -> Energy {
        self.view().smoothing()
    }

    fn curiosity(&self) -> Probability {
        self.view().curiosity()
    }
}

impl<V> Overlay<V>
where
    V: Profile + Copy,
{
    pub fn view(&self) -> V {
        self.view
    }

    fn local(&self) -> &HashMap<InfoOf<V>, HashMap<EdgeOf<V>, Encounter>> {
        &self.local
    }

    fn local_mut(&mut self) -> &mut HashMap<InfoOf<V>, HashMap<EdgeOf<V>, Encounter>> {
        &mut self.local
    }

    fn t_mut(&mut self) -> &mut usize {
        &mut self.t
    }

    fn cell(&self, info: &InfoOf<V>, edge: &EdgeOf<V>) -> Option<&Encounter> {
        self.local().get(info).and_then(|edges| edges.get(edge))
    }

    /// The local encounter, warmstarted from the view on first touch.
    fn encounter(&mut self, info: &InfoOf<V>, edge: &EdgeOf<V>) -> &mut Encounter {
        let view = self.view();
        self.local_mut()
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_insert_with(|| view.warmstart(info, edge))
    }
}
