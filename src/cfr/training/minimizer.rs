use crate::cfr::profile::profile::Profile;
use crate::cfr::tree::rps::action::Edge;
use crate::cfr::tree::rps::info::Info;
use crate::cfr::tree::rps::tree::Tree;
use crate::Probability;
use crate::Utility;

pub struct Minimizer {
    time: usize,
    regrets: Profile,
    current: Profile,
    average: Profile,
}
impl Minimizer {
    pub fn average(&self) -> &Profile {
        &self.average
    }
    pub fn current(&self) -> &Profile {
        &self.current
    }
    pub fn new(tree: &Tree) -> Self {
        let mut regrets = Profile::new();
        let mut average = Profile::new();
        let mut current = Profile::new();
        for info in tree.infosets() {
            let actions = info.sample().outgoing();
            let bucket = info.sample().bucket();
            let weight = 1.0 / actions.len() as Probability;
            let regret = 0.0;
            for action in actions {
                regrets.set_val(*bucket, *action, regret);
                average.set_val(*bucket, *action, weight);
                current.set_val(*bucket, *action, weight);
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
    pub fn update_regret(&mut self, info: &Info) {
        for (ref action, regret) in self.regret_vector(info) {
            let bucket = info.sample().bucket();
            let running = self.regrets.get_mut(bucket, action);
            *running = regret;
        }
    }
    pub fn update_policy(&mut self, info: &Info) {
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
    fn policy_vector(&self, info: &Info) -> Vec<(Edge, Probability)> {
        let regrets = info
            .sample()
            .outgoing()
            .iter()
            .map(|action| (**action, self.running_regret(info, action)))
            .map(|(a, r)| (a, r.max(Utility::MIN_POSITIVE)))
            .collect::<Vec<(Edge, Probability)>>();
        let sum = regrets.iter().map(|(_, r)| r).sum::<Utility>();
        let policy = regrets.into_iter().map(|(a, r)| (a, r / sum)).collect();
        policy
    }
    // regret calculation via regret matching +
    fn regret_vector(&self, info: &Info) -> Vec<(Edge, Utility)> {
        info.sample()
            .outgoing()
            .iter()
            .map(|action| (**action, self.matched_regret(info, action)))
            .collect()
    }
    // regret storge and calculation
    fn matched_regret(&self, info: &Info, action: &Edge) -> Utility {
        let running = self.running_regret(info, action);
        let instant = self.instant_regret(info, action);
        (running + instant).max(Utility::MIN_POSITIVE)
    }
    fn running_regret(&self, info: &Info, action: &Edge) -> Utility {
        let bucket = info.sample().bucket();
        let regret = self.regrets.get_ref(bucket, action);
        *regret
    }
    fn instant_regret(&self, info: &Info, action: &Edge) -> Utility {
        info.roots()
            .iter()
            .map(|root| self.current().gain(root, action))
            .sum::<Utility>()
    }
}
