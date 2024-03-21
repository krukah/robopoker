#[derive(Debug, Clone, Copy)]
pub struct Choice {
    pub action: Action,
    pub weight: u32,
}

#[derive(Debug, Clone)]
pub struct Policy {
    pub choices: Vec<Choice>,
}

impl Policy {
    pub fn choose(&self) -> Action {
        let mut sum = 0;
        let cum = self.choices.iter().map(|p| p.weight).sum();
        let roll = thread_rng().gen_range(0..cum);
        for policy in self.choices.iter() {
            sum += policy.weight;
            if roll < sum {
                return policy.action;
            }
        }
        unreachable!()
    }
}

use crate::gameplay::action::Action;
use rand::{thread_rng, Rng};
