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
    secret: Hole,
    public: Hand,
}

impl From<(Hole, Hand)> for Observation {
    fn from((secret, public): (Hole, Hand)) -> Self {
        Observation { secret, public }
    }
}

impl From<Observation> for i64 {
    fn from(observation: Observation) -> Self {
        let x = u64::from(observation.secret);
        let y = u64::from(observation.public);
        (Observation::spread(x) | (Observation::spread(y) << 1)) as i64
    }
}

impl Observation {
    /// Generates all possible successors of the current observation.
    ///
    /// This calculation depends on current street, which is proxied by Hand::size().
    /// We mask over cards that can't be observed, then union with the public cards
    pub fn successors(&self) -> Vec<Self> {
        let hand = self.secret;
        let mask = Hand::add(self.public, hand);
        let size = match self.public.size() {
            4 => 1,
            3 => 1,
            0 => 3,
            _ => panic!("shouldn't be generating successors on river"),
        };
        HandIterator::from((size, mask))
            .into_iter()
            .map(|hand| Observation::from((self.secret, Hand::add(self.public, hand))))
            .collect()
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
            Street::Show => Self::enumerate(2), // 5), // (!)
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
                println!("{}", board);
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
    /// @return Vec<Hole>: A vector containing all 990 possible opponent hole card combinations
    fn opponents(&self) -> Vec<Hole> {
        let size = 2usize;
        let mask = Hand::add(self.secret, self.public);
        HandIterator::from((size, mask)).into_iter().collect()
    }

    /// Calculates the equity of the current observation.
    ///
    /// This calculation integrations across ALL possible opponent hole cards.
    /// I'm not sure this is feasible across ALL 2.8B rivers * ALL 990 opponents.
    /// But it's a one-time calculation so we can afford to be slow
    pub fn equity(&self) -> f32 {
        let hand = self.secret;
        let this = Strength::from(Hand::add(self.public, hand));
        let opponents = self.opponents();
        let n = opponents.len();
        let equity = opponents
            .into_iter()
            .map(|hand| Strength::from(Hand::add(self.public, hand)))
            .map(|that| match &this.cmp(&that) {
                Ordering::Less => 0,
                Ordering::Equal => 1,
                Ordering::Greater => 2,
            })
            .sum::<u32>() as f32
            / n as f32
            / 2 as f32;
        println!("{} | {} | {:2}", self, this, equity);
        equity
    }

    /// (u64, u64) -> u64 mapping that preserves order.
    ///
    /// This is a bijection between two u64s that preserves order. We use
    /// it to identify a combination of
    /// (unordered private cards) x (unordered public cards) as a single integer.
    fn spread(x: u64) -> u64 {
        let mut a = x;
        a &= 0xFFFFFFFF;
        a = (a | (a << 16)) & 0x0000FFFF0000FFFF;
        a = (a | (a << 08)) & 0x00FF00FF00FF00FF;
        a = (a | (a << 04)) & 0x0F0F0F0F0F0F0F0F;
        a = (a | (a << 02)) & 0x3333333333333333;
        a = (a | (a << 01)) & 0x5555555555555555;
        a
    }
}

impl std::fmt::Display for Observation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} | {}", self.secret, self.public)
    }
}

/// Representation of private cards
/// might optimize this into less memory
///  u16      if order does not matter
/// [Card; 2] if order matters
/// in either case, we need impl From<Hold> for Hand to preserve contract
/// this eventual mapping to Hand(u64) then feels like maybe the Hole optimization is futile
/// haven't reasoned about it enough to tell if worth it
type Hole = Hand;
