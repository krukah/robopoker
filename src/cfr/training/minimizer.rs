use crate::cfr::profile::profile::P;
use crate::cfr::traits::action::E;
use crate::cfr::tree::info::I;
use crate::cfr::tree::tree::T;
use crate::Probability;
use crate::Utility;

/// optimizer'
pub(crate) struct M {
    time: usize,
    regrets: P,
    current: P,
    average: P,
}
impl M {
    pub fn average(&self) -> &P {
        &self.average
    }
    pub fn new(tree: &T) -> Self {
        let mut regrets = P::new();
        let mut average = P::new();
        let mut current = P::new();
        for info in tree.infosets() {
            let actions = info.sample().outgoing();
            let bucket = info.sample().bucket();
            let weight = 1.0 / actions.len() as Probability;
            let regret = 0.0;
            for action in actions {
                regrets.set(*bucket, *action, regret);
                average.set(*bucket, *action, weight);
                current.set(*bucket, *action, weight);
            }
        }
        Self {
            time: 0,
            average,
            current,
            regrets,
        }
    }
    // mutating update methods at each infoset
    pub fn update_regret(&mut self, info: &I) {
        for (ref action, regret) in self.regret_vector(info) {
            let bucket = info.sample().bucket();
            let running = self.regrets.get_mut(bucket, action);
            *running = regret;
        }
    }
    pub fn update_policy(&mut self, info: &I) {
        for (ref action, weight) in self.policy_vector(info) {
            let bucket = info.sample().bucket();
            let current = self.current.get_mut(bucket, action);
            let average = self.average.get_mut(bucket, action);
            *current = weight;
            *average *= self.time as Probability;
            *average += weight;
            *average /= self.time as Probability + 1.;
        }
        self.time += 1;
    }
    // policy calculation via cumulative regrets
    fn policy_vector(&self, info: &I) -> Vec<(E, Probability)> {
        let regrets = info
            .sample()
            .outgoing()
            .iter()
            .map(|action| (**action, self.running_regret(info, action)))
            .map(|(a, r)| (a, r.max(Utility::MIN_POSITIVE)))
            .collect::<Vec<(E, Probability)>>();
        let sum = regrets.iter().map(|(_, r)| r).sum::<Utility>();
        let policy = regrets.into_iter().map(|(a, r)| (a, r / sum)).collect();
        policy
    }
    // regret calculation via regret matching +
    fn regret_vector(&self, info: &I) -> Vec<(E, Utility)> {
        info.sample()
            .outgoing()
            .iter()
            .map(|action| (**action, self.matched_regret(info, action)))
            .collect()
    }
    // regret storge and calculation
    fn matched_regret(&self, info: &I, action: &E) -> Utility {
        let running = self.running_regret(info, action);
        let instant = self.instant_regret(info, action);
        (running + instant).max(Utility::MIN_POSITIVE)
    }
    fn running_regret(&self, info: &I, action: &E) -> Utility {
        let bucket = info.sample().bucket();
        *self.regrets.get_ref(bucket, action)
    }
    fn instant_regret(&self, info: &I, action: &E) -> Utility {
        info.roots()
            .iter()
            .map(|root| self.profile().gain(root, action))
            .sum::<Utility>()
    }
    fn profile(&self) -> &P {
        &self.current
    }
}
