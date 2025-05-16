use super::card::Card;
use super::deck::Deck;
use super::hand::Hand;
use super::hands::HandIterator;
use super::street::Street;
use super::strength::Strength;
use crate::Arbitrary;
use crate::Equity;
use std::cmp::Ordering;

/// Observation represents the memoryless state of the game in between chance actions.
///
/// We store each set of cards as a Hand which does not preserve dealing order. We can
/// generate successors by considering all possible cards that can be dealt. We can calculate
/// the equity of a given hand by comparing strength all possible villain hands.
/// This could be more memory efficient by using [Card; 2] for pocket Hands,
/// then impl From<[Card; 2]> for Hand. But the convenience of having the same Hand type is worth it.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, PartialOrd, Ord)]
pub struct Observation {
    pocket: Hand, // if memory-bound: could be Hole/u16
    public: Hand, // if memory-bound: could be Board/[Option<Card>; 5]
}

impl Observation {
    pub fn children<'a>(&'a self) -> impl Iterator<Item = Self> + 'a {
        let n = self.street().n_revealed();
        let removed = Hand::from(*self);
        HandIterator::from((n, removed))
            .map(|reveal| Hand::add(self.public, reveal))
            .map(|public| Self::from((self.pocket, public)))
    }
    pub fn equity(&self) -> Equity {
        assert!(self.street() == Street::Rive);
        let hand = Hand::from(*self);
        let hero = Strength::from(hand);
        let (won, sum) = HandIterator::from((2, hand))
            .map(|villain| Hand::add(self.public, villain))
            .map(|villain| Strength::from(villain))
            .map(|villain| hero.cmp(&villain))
            .filter(|&ord| ord != Ordering::Equal)
            .fold((0u32, 0u32), |(wins, total), ord| match ord {
                Ordering::Greater => (wins + 1, total + 1),
                Ordering::Less => (wins, total + 1),
                Ordering::Equal => unreachable!(),
            });
        match sum {
            0 => 0.5, // all draw edge case
            _ => won as Equity / sum as Equity,
        }
    }
    pub fn simulate(&self, _: usize) -> Equity {
        todo!("run out some number of simulations and take equity as average")
    }
    pub fn street(&self) -> Street {
        Street::from(self.public.size())
    }
    pub fn pocket(&self) -> &Hand {
        &self.pocket
    }
    pub fn public(&self) -> &Hand {
        &self.public
    }

    // #[cfg(feature = "entropy")]
    fn shuffle(hand: String) -> String {
        use rand::seq::SliceRandom;
        let ref mut rng = rand::rng();
        let mut cards = hand
            .chars()
            .collect::<Vec<char>>()
            .chunks(2)
            .map(|c| c.iter().collect::<String>())
            .collect::<Vec<String>>();
        cards.shuffle(rng);
        cards.join("")
    }
    pub fn equivalent(&self) -> String {
        super::permutation::Permutation::random()
            .permute(self)
            .to_string()
            .split(Self::SEPARATOR)
            .map(|s| s.to_string())
            .map(|s| s.trim().to_string())
            .map(|s| Self::shuffle(s))
            .collect::<Vec<String>>()
            .join(Self::SEPARATOR)
    }

    const SEPARATOR: &'static str = "~";
}
/// i64 isomorphism
///
/// Packs all the cards in order, starting from LSBs.
/// Good for database serialization. Interchangable with u64
impl From<Observation> for i64 {
    fn from(observation: Observation) -> Self {
        std::iter::empty::<Card>()
            .chain(observation.public.into_iter())
            .chain(observation.pocket.into_iter())
            .map(|card| 1 + u8::from(card) as u64) // distinguish 0x00 and 2c
            .fold(0u64, |acc, card| acc << 8 | card) as i64 // next card
    }
}
impl From<i64> for Observation {
    fn from(bits: i64) -> Self {
        Self::from(
            (0u64..8u64)
                .map(|i| bits >> (i * 8))
                .take_while(|&bits| bits > 0)
                .map(|bits| bits as u8)
                .map(|bits| bits - 1) // distinguish 0x00 and 2c
                .map(Card::from)
                .map(Hand::from)
                .enumerate()
                .fold(
                    (Hand::empty(), Hand::empty()),
                    |(pocket, public), (i, hand)| {
                        if i < 2 {
                            (Hand::add(pocket, hand), public)
                        } else {
                            (pocket, Hand::add(public, hand))
                        }
                    },
                ),
        )
    }
}

/// assemble Observation from private + public Hands
impl From<(Hand, Hand)> for Observation {
    fn from((pocket, public): (Hand, Hand)) -> Self {
        assert!(pocket.size() == 2);
        assert!(public.size() <= 5);
        Self { pocket, public }
    }
}

/// Generate a random observation for a given street
impl From<Street> for Observation {
    fn from(street: Street) -> Self {
        let mut deck = Deck::new();
        let n = street.n_observed();
        let public = (0..n)
            .map(|_| deck.draw())
            .map(u64::from)
            .map(Hand::from)
            .fold(Hand::empty(), Hand::add);
        let pocket = (0..2)
            .map(|_| deck.draw())
            .map(u64::from)
            .map(Hand::from)
            .fold(Hand::empty(), Hand::add);
        Self::from((pocket, public))
    }
}

/// coalesce public + private cards into single Hand
impl From<Observation> for Hand {
    fn from(observation: Observation) -> Self {
        Self::add(observation.pocket, observation.public)
    }
}

impl TryFrom<&str> for Observation {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let (pocket, public) = s
            .trim()
            .split_once(Self::SEPARATOR)
            .unwrap_or((s.trim(), ""));
        let pocket = Hand::try_from(pocket)?;
        let public = Hand::try_from(public)?;
        match (pocket.size(), public.size()) {
            (2, 0) | (2, 3) | (2, 4) | (2, 5) => Ok(Self::from((pocket, public))),
            _ => Err(format!("invalid card counts: {} {}", pocket, public)),
        }
    }
}

impl Arbitrary for Observation {
    fn random() -> Self {
        Self::from(Street::random())
    }
}

/// display Observation as pocket + public
impl std::fmt::Display for Observation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {} {}", self.pocket, Self::SEPARATOR, self.public)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::isomorphism::Isomorphism;

    #[test]
    fn bijective_i64() {
        let random = Observation::random();
        assert!(random == Observation::from(i64::from(random)));
    }

    #[test]
    fn shuffle() {
        let random = Observation::random();
        let swappy = Observation::try_from(random.equivalent().as_str()).unwrap();
        assert!(Isomorphism::from(random) == Isomorphism::from(swappy));
    }
}
