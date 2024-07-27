use super::hands::HandIterator;
use crate::cards::hand::Hand;
use crate::cards::street::Street;
use crate::cards::strength::Strength;
use std::cmp::Ordering;

/// Observation represents the memoryless state of the game in between chance actions.
///
/// We store each set of cards as a Hand which does not preserve dealing order. We can
/// generate successors by considering all possible cards that can be dealt. We can calculate
/// the equity of a given hand by comparing strength all possible opponent hands.
/// This could be more memory efficient by using [Card; 2] for secret Hands,
/// then impl From<[Card; 2]> for Hand. But the convenience of having the same Hand type is worth it.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, PartialOrd, Ord)]
pub struct Observation {
    secret: Hand,
    public: Hand,
}

impl Observation {
    pub fn all(street: Street) -> Vec<Observation> {
        println!("Generating all {} observations...", street);
        let n = match street {
            Street::Flop => 3,
            Street::Turn => 4,
            Street::Rive => 5,
            _ => panic!("no other transitions"),
        };
        let mut observations = Vec::new();
        let secrets = HandIterator::from((2usize, Hand::from(0u64)));
        for secret in secrets {
            let publics = HandIterator::from((n, secret));
            for public in publics {
                observations.push(Observation::from((secret, public)));
            }
        }
        observations
    }
    /// Calculates the equity of the current observation.
    ///
    /// This calculation integrations across ALL possible opponent hole cards.
    /// I'm not sure this is feasible across ALL 2.8B rivers * ALL 990 opponents.
    /// But it's a one-time calculation so we can afford to be slow
    pub fn equity(&self) -> f32 {
        assert!(self.street() == Street::Rive);
        let observed = self.observed();
        let hero = Strength::from(observed);
        let opponents = HandIterator::from((2usize, observed));
        let n = opponents.combinations();
        opponents
            .map(|oppo| Hand::add(self.public, oppo))
            .map(|hand| Strength::from(hand))
            .map(|oppo| match &hero.cmp(&oppo) {
                Ordering::Less => 0,
                Ordering::Equal => 1,
                Ordering::Greater => 2,
            })
            .sum::<u32>() as f32
            / n as f32
            / 2 as f32
    }

    /// Generates all possible successors of the current observation.
    ///
    /// This calculation depends on current street, which is proxied by Hand::size().
    /// We mask over cards that can't be observed, then union with the public cards
    pub fn outnodes(&self) -> impl IntoIterator<Item = Observation> + '_ {
        let excluded = self.observed();
        let n_revealed = match self.street() {
            Street::Pref => 3,
            Street::Flop => 1,
            Street::Turn => 1,
            _ => panic!("no children for river"),
        };
        // BIG ITERATOR
        // LOOP over (2 + street)-handed OBSERVATIONS
        // EXPAND the current observation's BOARD CARDS
        // PRESERVE the current observation's HOLE CARDS
        HandIterator::from((n_revealed, excluded))
            .map(|reveal| Hand::add(self.public, reveal))
            .map(|public| Observation::from((self.secret, public)))
    }

    fn street(&self) -> Street {
        match self.public.size() {
            0 => Street::Pref,
            3 => Street::Flop,
            4 => Street::Turn,
            5 => Street::Rive,
            _ => panic!("no other sizes"),
        }
    }

    /// Generate mask conditional on .secret, .public
    fn observed(&self) -> Hand {
        Hand::add(self.secret, self.public)
    }
}

impl From<(Hand, Hand)> for Observation {
    fn from((secret, public): (Hand, Hand)) -> Self {
        assert!(secret.size() == 2);
        assert!(public.size() <= 5);
        Observation { secret, public }
    }
}

impl From<Observation> for i64 {
    fn from(observation: Observation) -> Self {
        // big prime numbers help us with pseudo-hasing while preserving order
        let x = u64::from(observation.secret).wrapping_mul(0x9e3779b97f4a7c15);
        let y = u64::from(observation.public).wrapping_mul(0x517cc1b727220a95);
        let i = x.wrapping_add(y);
        i as i64
    }
}

impl std::fmt::Display for Observation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} + {}", self.secret, self.public)
    }
}
