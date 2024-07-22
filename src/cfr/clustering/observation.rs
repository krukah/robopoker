use crate::cards::hand::{Hand, HandIterator};
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
    /// Generates all possible successors of the current observation.
    ///
    /// This calculation depends on current street, which is proxied by Hand::size().
    /// We mask over cards that can't be observed, then union with the public cards
    pub fn successors(&self) -> impl Iterator<Item = Observation> + '_ {
        let mask = self.hand();
        let size = match self.public.size() {
            4 => 1,
            3 => 1,
            0 => 3,
            _ => panic!("shouldn't be generating successors on river"),
        };
        HandIterator::from((size, mask))
            .into_iter()
            .map(|hand| Observation::from((self.secret, Hand::add(self.public, hand))))
    }

    /// Generates all possible predecessors of a given street.
    ///
    /// We lazily enumerate every possible position across all streets. That's literally every possible poker hand!
    ///
    /// In total we have ~3B distinct "situations". Many of them are strategically isomorphic.
    /// ```
    ///   2_809_475_760
    ///   + 305_377_800
    ///   +  25_989_600
    ///   +       1_326
    ///   _____________
    ///   3_141_852_486
    /// ```
    pub fn predecessors(street: Street) -> Vec<Self> {
        match street {
            Street::Pref => panic!("no previous street"),
            Street::Flop => Self::enumerate(2),
            Street::Turn => Self::enumerate(3),
            Street::Rive => Self::enumerate(4),
            Street::Show => Self::enumerate(5), // (!)
        }
    }

    /// Generates all possible situations as a function of street.
    ///
    /// This method calculates all combinations of hole cards (secret) and community cards (public):
    ///
    /// 1. Secret cards
    ///    - Preflop                    1_326
    ///    - Combinations: C(52,2) =    1_326
    ///
    /// 2. Public cards (for each secret combination)
    ///    - River                      2_809_475_760
    ///    - Combinations: C(50,5) =    2_118_760
    ///
    ///    - Turn                       305_377_800
    ///    - Combinations: C(50,4) =    230_300
    ///
    ///    - Flop                       25_989_600
    ///    - Combinations: C(50,3) =    19_600
    ///
    ///
    /// 3. Total unique river situations:
    ///    - 1,326 * 2,118,760 = 2,809,475,760
    ///
    /// The method uses nested iterations:
    ///   - Outer loop: Generates all possible secret hands (hole cards)
    ///   - Inner loop: For each secret hand, generates all possible public hands (community cards)
    /// There could be consideration for breaking symmetry and reducing Hands up-to-stategic-isomorphism. This only reduces 2.8B > 2.4B in practice, maybe not worth it.
    fn enumerate(count: usize) -> Vec<Self> {
        let size = 2usize;
        let mask = Hand::from(0u64);
        let secrets = HandIterator::from((size, mask));
        let permutations: usize = match count {
            2 => 1_326,
            3 => 25_989_600,
            4 => 305_377_800,
            5 => 2_809_475_760,
            _ => panic!("invalid count"),
        };
        let mut boards = Vec::with_capacity(permutations);
        for secret in secrets {
            let size = count;
            let mask = secret;
            let publics = HandIterator::from((size, mask));
            for public in publics {
                let board = Observation::from((secret, public));
                boards.push(board);
            }
        }
        boards
    }

    /// Enumerates all possible opponent hole cards given the current observation.
    ///
    /// This enumeration is crucial for calculating hand equity. It calculates all potential 2-card combinations an opponent might hold,
    /// considering the known cards (our hole cards and the community cards):
    ///
    /// 1. Opponent's hole cards:
    ///    - Choose 2 cards from the remaining 45 cards
    ///    - Remaining cards = 52 - (2 our hole cards + 5 community cards)
    ///    - Combinations: C(45,2) = 990
    ///
    /// The calculation excludes cards that are:
    ///   - In our own hole cards (self.secret)
    ///   - Visible as community cards (self.public)
    ///
    ///
    /// @return Vec<Hand>: A vector containing all 990 possible opponent hole card combinations
    fn opponents(&self) -> HandIterator {
        let size = 2usize;
        let mask = self.hand();
        HandIterator::from((size, mask))
    }

    /// Generate mask conditional on .secret, .public
    fn hand(&self) -> Hand {
        Hand::add(self.secret, self.public)
    }

    /// Calculates the equity of the current observation.
    ///
    /// This calculation integrations across ALL possible opponent hole cards.
    /// I'm not sure this is feasible across ALL 2.8B rivers * ALL 990 opponents.
    /// But it's a one-time calculation so we can afford to be slow
    pub fn equity(&self) -> f32 {
        let ours = Strength::from(self.hand());
        let opponents = self.opponents();
        let n = opponents.combinations();
        let equity = opponents
            .map(|hand| Strength::from(Hand::add(self.public, hand)))
            .map(|hers| match &ours.cmp(&hers) {
                Ordering::Less => 0,
                Ordering::Equal => 1,
                Ordering::Greater => 2,
            })
            .sum::<u32>() as f32
            / n as f32
            / 2 as f32;
        if self.select() {
            println!("Equity Calc {} | {} | {:2}", self, ours, equity);
        }
        equity
    }

    /// Determines whether this Observation is selected for logging
    fn select(&self) -> bool {
        i64::from(*self) % (3331333) == 0
    }
}

impl From<(Hand, Hand)> for Observation {
    fn from((secret, public): (Hand, Hand)) -> Self {
        Observation { secret, public }
    }
}

impl From<Observation> for i64 {
    fn from(observation: Observation) -> Self {
        let x = u64::from(observation.secret).wrapping_mul(0x9e3779b97f4a7c15);
        let y = u64::from(observation.public).wrapping_mul(0x517cc1b727220a95);
        let i = x.wrapping_add(y);
        i as i64
    }
}

impl std::fmt::Display for Observation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} | {}", self.secret, self.public)
    }
}
