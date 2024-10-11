use super::card::Card;
use super::deck::Deck;
use super::hand::Hand;
use super::hands::HandIterator;
// use super::isomorphism::Isomorphism;
use super::rank::Rank;
use super::street::Street;
use super::strength::Strength;
use super::suit::Suit;
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
    /// Generate all possible observations for a given street
    pub fn enumerate(street: Street) -> Vec<Self> {
        let n = Self::observable(street);
        let inner = HandIterator::from((n, Hand::from(0b11))).combinations();
        let outer = HandIterator::from((2, Hand::from(0b00))).combinations();
        let space = outer * inner;
        let mut observations = Vec::with_capacity(space);
        for hole in HandIterator::from((2, Hand::from(0b00))) {
            for board in HandIterator::from((n, hole)) {
                // if Isomorphism::is_canonical(&Self::from((hole, board))) {
                observations.push(Self::from((hole, board)));
                // }
            }
        }
        observations
    }

    /// Generates all possible successors of the current observation.
    /// LOOP over (2 + street)-handed OBSERVATIONS
    /// EXPAND the current observation's BOARD CARDS
    /// PRESERVE the current observation's HOLE CARDS
    pub fn outnodes(&self) -> Vec<Self> {
        let n = self.revealable();
        let excluded = Hand::add(self.public, self.secret);
        HandIterator::from((n, excluded))
            .map(|reveal| Hand::add(self.public, reveal))
            .map(|public| Observation::from((self.secret, public)))
            // .filter(|obs| Isomorphism::is_canonical(obs))
            .collect::<Vec<Self>>()
    }

    /// Calculates the equity of the current observation.
    ///
    /// This calculation integrations across ALL possible opponent hole cards.
    /// I'm not sure this is feasible across ALL 2.8B rivers * ALL 990 opponents.
    /// But it's a one-time calculation so we can afford to be slow
    pub fn equity(&self) -> f32 {
        assert!(self.street() == Street::Rive);
        let hand = Hand::add(self.public, self.secret);
        let hero = Strength::from(hand);
        let opponents = HandIterator::from((2usize, hand));
        let n = opponents.combinations();
        opponents
            .map(|oppo| Hand::add(self.public, oppo))
            .map(|hand| Strength::from(hand))
            .map(|oppo| match &hero.cmp(&oppo) {
                Ordering::Greater => 2,
                Ordering::Equal => 1,
                Ordering::Less => 0,
            })
            .sum::<u32>() as f32
            / n as f32
            / 2 as f32
    }

    pub fn street(&self) -> Street {
        match self.public.size() {
            0 => Street::Pref,
            3 => Street::Flop,
            4 => Street::Turn,
            5 => Street::Rive,
            _ => unreachable!("no other sizes"),
        }
    }
    pub fn secret(&self) -> &Hand {
        &self.secret
    }
    pub fn public(&self) -> &Hand {
        &self.public
    }

    fn observable(street: Street) -> usize {
        match street {
            Street::Flop => 3,
            Street::Turn => 4,
            Street::Rive => 5,
            _ => unreachable!("no other transitions"),
        }
    }

    fn revealable(&self) -> usize {
        match self.street() {
            Street::Pref => 3,
            Street::Flop => 1,
            Street::Turn => 1,
            _ => unreachable!("no children for river"),
        }
    }
}

/// i64 isomorphism
///
/// Packs all the cards in order, starting from LSBs.
/// Good for database serialization. Interchangable with u64
impl From<Observation> for i64 {
    fn from(observation: Observation) -> Self {
        Vec::<Card>::from(observation.public)
            .iter()
            .chain(Vec::<Card>::from(observation.secret).iter())
            .copied()
            .map(|card| 1 + u8::from(card) as u64) // distinguish between 0x00 and 2c
            .fold(0u64, |acc, card| acc << 8 | card) as i64
    }
}
impl From<i64> for Observation {
    fn from(bits: i64) -> Self {
        let mut i = 0;
        let mut bits = bits as u64;
        let mut secret = Hand::from(0u64);
        let mut public = Hand::from(0u64);
        while bits > 0 {
            let card = ((bits & Rank::MASK as u64) - 1) as u8;
            let hand = Hand::from(u64::from(Card::from(card)));
            if i < 2 {
                secret = Hand::add(secret, hand);
            } else {
                public = Hand::add(public, hand);
            }
            i += 1;
            bits >>= 8;
        }
        assert!(secret.size() == 2);
        assert!(public.size() <= 5);
        Observation { secret, public }
    }
}

/// assemble Observation from private + public Hands
impl From<(Hand, Hand)> for Observation {
    /// TODO: implement strategic isomorphism
    fn from((secret, public): (Hand, Hand)) -> Self {
        assert!(secret.size() == 2);
        assert!(public.size() <= 5);
        Observation { secret, public }
    }
}

/// Generate a random observation for a given street
impl From<Street> for Observation {
    fn from(street: Street) -> Self {
        let n = match street {
            Street::Pref => 0,
            Street::Flop => 3,
            Street::Turn => 4,
            Street::Rive => 5,
        };
        let mut deck = Deck::new();
        let public = Hand::from((0..n).map(|_| deck.draw()).collect::<Vec<Card>>());
        let secret = Hand::from((0..2).map(|_| deck.draw()).collect::<Vec<Card>>());
        Self::from((secret, public))
    }
}

/// coalesce public + private cards into single Hand
impl From<Observation> for Hand {
    fn from(observation: Observation) -> Self {
        Hand::add(observation.secret, observation.public)
    }
}

/// a bit of a reach to impl Iterator<Suit> for Observation
/// but i want a way to lazily get the Suit
/// of the next highest card, from hands in Obs
impl Iterator for Observation {
    type Item = Suit;
    fn next(&mut self) -> Option<Self::Item> {
        None.or_else(|| self.secret.next())
            .or_else(|| self.public.next())
            .map(|card| card.suit())
            .map(|suit| {
                self.secret = Hand::from(u64::from(self.secret) & !u64::from(suit));
                self.public = Hand::from(u64::from(self.public) & !u64::from(suit));
                suit
            })
    }
}

/// display Observation as secret + public
impl std::fmt::Display for Observation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} + {}", self.secret, self.public)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suit_iterator() {
        let mut iter = Observation::from((Hand::from("6c Td"), Hand::from("Jc 7h Ks"))).into_iter();
        assert!(iter.next() == Some(Suit::D));
        assert!(iter.next() == Some(Suit::C));
        assert!(iter.next() == Some(Suit::S));
        assert!(iter.next() == Some(Suit::H));
        assert!(iter.next() == None);
    }

    #[test]
    fn bijective_i64() {
        let random = Observation::from(Street::Flop);
        assert!(random == Observation::from(i64::from(random)));
    }

    #[test]
    fn bijective_canonical() {}

    #[test]
    fn injective_canonical() {}
}
