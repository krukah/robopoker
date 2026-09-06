use super::*;
use pokerkit::Arbitrary;
use pokerkit::Probability;
use std::cmp::Ordering;

/// A player's view of the game: hole cards plus visible board — the atomic
/// unit of poker abstraction. Carries every card a player can see and nothing
/// about action history, which the game tree tracks separately.
///
/// Serializes to `i64` by packing cards into bytes for database storage; the
/// string form separates hole from board with `~`.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, PartialOrd, Ord)]
pub struct Observation {
    pocket: Hand,
    public: Hand,
}

impl Observation {
    /// All possible next-street observations — deal 3 cards for the flop, 1
    /// for turn/river, from the remaining deck.
    pub fn children(&self) -> impl Iterator<Item = Self> + '_ {
        let n = self.street().next().n_revealed();
        HandIterator::from((n, Hand::from(*self)))
            .map(|reveal| Hand::add(self.public, reveal))
            .map(|public| Self::from((self.pocket, public)))
    }
    /// Exact equity against the uniform distribution of opponent hands: the
    /// fraction of all villain holdings we beat, ties excluded. River only.
    pub fn equity(&self) -> Probability {
        debug_assert_eq!(self.street(), Street::Rive);
        let hero = Strength::from(Hand::from(*self));
        let (won, sum) = self
            .opponents()
            .map(Hand::from)
            .map(Strength::from)
            .map(|villain| hero.cmp(&villain))
            .fold((0u32, 0u32), |(wins, total), ord| match ord {
                Ordering::Greater => (wins + 1, total + 1),
                Ordering::Equal => (wins, total),
                Ordering::Less => (wins, total + 1),
            });
        match sum {
            0 => 0.5,
            _ => won as Probability / sum as Probability,
        }
    }
    /// Monte Carlo equity estimation (not yet implemented).
    pub fn simulate(&self, _: usize) -> Probability {
        todo!("run out some number of simulations and take equity as average")
    }
    /// Equity of `self` vs a specific villain pocket on the shared board:
    /// `trials` Monte Carlo runouts to the river, wins (ties = 0.5) over
    /// trials. On the river the comparison is exact and `trials` is ignored.
    pub fn equity_vs(&self, villain: Hand, trials: usize) -> Probability {
        use rand::seq::IndexedRandom;
        let need = 5 - self.public.size();
        if need == 0 {
            let h = Strength::from(Hand::from(*self));
            let v = Strength::from(Hand::add(villain, self.public));
            return match h.cmp(&v) {
                Ordering::Greater => 1.0,
                Ordering::Equal => 0.5,
                Ordering::Less => 0.0,
            };
        }
        let used = Hand::add(Hand::from(*self), villain);
        let candidates: Vec<Card> = used.complement().collect();
        let ref mut rng = rand::rng();
        let total: Probability = (0..trials)
            .map(|_| {
                let runout: Hand = candidates.choose_multiple(rng, need).copied().collect();
                let board = Hand::add(self.public, runout);
                let h = Strength::from(Hand::add(self.pocket, board));
                let v = Strength::from(Hand::add(villain, board));
                match h.cmp(&v) {
                    Ordering::Greater => 1.0,
                    Ordering::Equal => 0.5,
                    Ordering::Less => 0.0,
                }
            })
            .sum();
        total / trials as Probability
    }
    /// Infers the street from total observed cards.
    pub fn street(&self) -> Street {
        Street::from(self.public().size() + self.pocket().size())
    }
    /// The player's hole cards.
    pub fn pocket(&self) -> &Hand {
        &self.pocket
    }
    /// The community board cards.
    pub fn public(&self) -> &Hand {
        &self.public
    }
    /// All possible opponent observations: same board, hole cards drawn from
    /// what hero cannot see. Equal implicit weight each, so this is the
    /// uniform prior over villain holdings — C(45, 2) = 990 on the river.
    pub fn opponents(&self) -> impl Iterator<Item = Self> + '_ {
        HandIterator::from((2, Hand::from(*self)))
            .map(|hole| (hole, self.public))
            .map(Self::from)
    }
    /// String separator between hole and board in display format.
    pub const SEPARATOR: &'static str = "~";
}
/// i64 isomorphism: packs all cards in order from the LSBs up. Interchangeable
/// with u64, and what the database stores.
impl From<Observation> for i64 {
    fn from(observation: Observation) -> Self {
        std::iter::empty::<Card>()
            .chain(observation.public)
            .chain(observation.pocket)
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
                .fold((Hand::empty(), Hand::empty()), |(pocket, public), (i, hand)| {
                    if i < 2 {
                        (Hand::add(pocket, hand), public)
                    } else {
                        (pocket, Hand::add(public, hand))
                    }
                }),
        )
    }
}

/// assemble Observation from private + public Hands
impl From<(Hand, Hand)> for Observation {
    fn from((pocket, public): (Hand, Hand)) -> Self {
        debug_assert_eq!(pocket.size(), 2);
        debug_assert!(public.size() <= 5);
        Self { pocket, public }
    }
}

/// Generate a random observation for a given street
impl From<Street> for Observation {
    fn from(street: Street) -> Self {
        let mut deck = Deck::new();
        let n = street.n_observed();
        let pocket = (0..2)
            .map(|_| deck.draw())
            .map(u64::from)
            .map(Hand::from)
            .fold(Hand::empty(), Hand::add);
        let public = (2..n)
            .map(|_| deck.draw())
            .map(u64::from)
            .map(Hand::from)
            .fold(Hand::empty(), Hand::add);
        Self::from((pocket, public))
    }
}

/// what is our belief of the deck from this perspective
impl From<Observation> for Deck {
    fn from(observation: Observation) -> Self {
        Self::from(Hand::from(observation).complement())
    }
}

/// coalesce public + private cards into single Hand
impl From<Observation> for Hand {
    fn from(observation: Observation) -> Self {
        Self::add(observation.pocket, observation.public)
    }
}

impl From<(Hole, Board)> for Observation {
    fn from((hole, board): (Hole, Board)) -> Self {
        Self::from((Hand::from(hole), Hand::from(board)))
    }
}

/// losing ordering information, reduce revealed cards into Observation
impl TryFrom<Vec<Card>> for Observation {
    type Error = String;

    fn try_from(cards: Vec<Card>) -> Result<Self, Self::Error> {
        if cards.iter().collect::<std::collections::BTreeSet<_>>().len() == cards.len() {
            match cards.len() {
                2 | 5 | 6 | 7 => Ok(Self::from((Hand::from(cards[..2].to_vec()), Hand::from(cards[2..].to_vec())))),
                _ => Err(format!("invalid card count: {}", cards.len())),
            }
        } else {
            Err(format!("duplicate cards: {}", cards.len()))
        }
    }
}

impl TryFrom<&str> for Observation {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let (pocket, public) = s.trim().split_once(Self::SEPARATOR).unwrap_or((s.trim(), ""));
        let pocket = Hand::try_from(pocket)?;
        let public = Hand::try_from(public)?;
        if Hand::overlaps(&pocket, &public) {
            return Err("duplicate cards between pocket and board".to_string());
        }
        match (pocket.size(), public.size()) {
            (2, 0 | 3 | 4 | 5) => Ok(Self::from((pocket, public))),
            _ => Err(format!("invalid card counts: {pocket} {public}")),
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

impl serde::Serialize for Observation {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&self.to_string())
    }
}
impl<'de> serde::Deserialize<'de> for Observation {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = serde::Deserialize::deserialize(d)?;
        Self::try_from(s.as_str()).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bijective_i64() {
        let random = Observation::random();
        assert_eq!(random, Observation::from(i64::from(random)));
    }

    /// Counts assume the full 52-card deck. Under `shortdeck` the
    /// candidate pool drops to 36, so C(45,2) becomes C(29,2) etc. —
    /// gated rather than parameterized since the nearby HandIterator
    /// tests follow the same convention.
    #[cfg(not(feature = "shortdeck"))]
    #[test]
    fn opponents_count() {
        assert_eq!(Observation::from(Street::Rive).opponents().count(), 0990); // C(45, 2)
        assert_eq!(Observation::from(Street::Turn).opponents().count(), 1035); // C(46, 2)
        assert_eq!(Observation::from(Street::Flop).opponents().count(), 1081); // C(47, 2)
        assert_eq!(Observation::from(Street::Pref).opponents().count(), 1225); // C(50, 2)
    }
}
