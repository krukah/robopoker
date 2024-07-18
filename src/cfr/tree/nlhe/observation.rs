use std::cmp::Ordering;

use crate::{
    cards::{board::Street, card::Card},
    evaluation::{
        evaluation::{Evaluator, LazyEvaluator},
        strength::Strength,
    },
};

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct Observation {
    private: [Card; 2],
    publics: [Card; 5], // enum over Street
}

impl Observation {
    /// this is only available for terminal observations
    pub fn equity(&self) -> f32 {
        let ref hero = self.private;
        let ref hero = self.strength(hero);
        let villains = self.villains();
        villains
            .iter()
            .map(|ref hand| self.strength(hand))
            .map(|ref rank| hero.cmp(rank))
            .map(|ref comp| match comp {
                Ordering::Less => 0,
                Ordering::Equal => 1,
                Ordering::Greater => 2,
            })
            .sum::<u32>() as f32
            / villains.len() as f32
            / 2 as f32
    }

    /// this is only available for terminal observations
    fn strength(&self, private: &[Card; 2]) -> Strength {
        LazyEvaluator::strength(
            &Vec::new()
                .iter()
                .chain(private.iter())
                .chain(self.publics.iter())
                .collect::<Vec<&Card>>()[..],
        )
    }

    /// this is only available for terminal observations
    fn villains(&self) -> Vec<[Card; 2]> {
        todo!("terminal: generate all possible villain hands")
    }
}
