use crate::cards::rank::Rank;

use super::card::Card;
use super::hand::Hand;
use super::hands::HandIterator;
use super::street::Street;
use super::strength::Strength;
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
        println!("enumerating all observations at {}", street);
        let n = match street {
            Street::Flop => 3,
            Street::Turn => 4,
            Street::Rive => 5,
            _ => unreachable!("no other transitions"),
        };
        let mut observations = Vec::new(); // TODO make with_capacity, conditional on street
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

    /// Generates all possible successors of the current observation.
    ///
    /// This calculation depends on current street, which is proxied by Hand::size().
    /// We mask over cards that can't be observed, then union with the public cards
    pub fn outnodes(&self) -> Vec<Observation> {
        // LOOP over (2 + street)-handed OBSERVATIONS
        // EXPAND the current observation's BOARD CARDS
        // PRESERVE the current observation's HOLE CARDS
        let excluded = Hand::add(self.public, self.secret);
        let expanded = match self.street() {
            Street::Pref => 3,
            Street::Flop => 1,
            Street::Turn => 1,
            _ => unreachable!("no children for river"),
        };
        HandIterator::from((expanded, excluded))
            .map(|reveal| Hand::add(self.public, reveal))
            .map(|public| Observation::from((self.secret, public)))
            .collect::<Vec<Self>>()
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
}

impl From<(Hand, Hand)> for Observation {
    /// TODO: implement strategic isomorphism
    fn from((secret, public): (Hand, Hand)) -> Self {
        assert!(secret.size() == 2);
        assert!(public.size() <= 5);
        Observation { secret, public }
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

/// conversion to i64 for SQL storage and impl ToSql directly
impl tokio_postgres::types::ToSql for Observation {
    fn to_sql(
        &self,
        ty: &tokio_postgres::types::Type,
        out: &mut bytes::BytesMut,
    ) -> Result<tokio_postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        i64::from(*self).to_sql(ty, out)
    }

    fn accepts(ty: &tokio_postgres::types::Type) -> bool {
        <i64 as tokio_postgres::types::ToSql>::accepts(ty)
    }

    fn to_sql_checked(
        &self,
        ty: &tokio_postgres::types::Type,
        out: &mut bytes::BytesMut,
    ) -> Result<tokio_postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        i64::from(*self).to_sql_checked(ty, out)
    }
}

impl std::fmt::Display for Observation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} + {}", self.secret, self.public)
    }
}
