use super::card::Card;
use super::card_seq::CardSeq;
use super::hand::Hand;

/// A compact encoding of card ordering within a [`Hand`].
///
/// Since `Hand` is an unordered bitset, deal-time card ordering is lost.
/// `Perm` records that ordering as a [Lehmer code](https://en.wikipedia.org/wiki/Lehmer_code)
/// packed into a single byte. This suffices for up to 5 cards (5! = 120 < 256).
///
/// # Trait relationships
///
/// - [`FromIterator<Card>`] — collect an ordered card sequence into a `Perm`
/// - [`From<u8>`] / [`Into<u8>`] — serialize the Lehmer code
/// - [`Perm::arrange`] returns a [`CardSeq`] iterator
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Perm(u8);

impl Perm {
    /// The identity permutation (canonical ascending order).
    pub fn identity() -> Self {
        Self(0)
    }
    /// Constructs a `Perm` from an ordered slice of cards.
    ///
    /// The Lehmer code is computed by comparing `Card` u8 encodings
    /// directly — no sorting or intermediate allocation needed, since
    /// `Hand`'s iteration order matches the u8 encoding.
    pub fn of(cards: &[Card]) -> Self {
        cards.iter().copied().collect()
    }
    /// Returns a zero-allocation [`CardSeq`] iterator that yields
    /// cards from `hand` in the order encoded by this permutation.
    pub fn arrange(&self, hand: Hand) -> CardSeq {
        let mut cards = [Card::from(0u8); 5];
        let mut n = 0u8;
        for card in hand {
            cards[n as usize] = card;
            n += 1;
        }
        CardSeq::new(cards, Self::decode(self.0, n), n)
    }
    /// Lehmer code from card ordering. Each position counts how many
    /// subsequent cards have a smaller u8 encoding (= lower canonical rank).
    fn encode(cards: &[Card; 5], n: u8) -> u8 {
        (0..n as usize)
            .map(|i| {
                cards[i + 1..n as usize]
                    .iter()
                    .filter(|c| u8::from(**c) < u8::from(cards[i]))
                    .count() as u8
                    * Self::factorial(n as usize - 1 - i)
            })
            .sum()
    }
    /// Decode Lehmer code to index permutation using a bitmask pool.
    /// The k-th available index is found by scanning set bits, replacing
    /// `Vec::remove` with O(1)-space bit manipulation.
    fn decode(mut code: u8, n: u8) -> [u8; 5] {
        let mut pool = (1u8 << n) - 1;
        let mut order = [0u8; 5];
        for (i, slot) in order.iter_mut().enumerate().take(n as usize) {
            let f = Self::factorial(n as usize - 1 - i);
            let bit = Self::nth_set_bit(pool, code / f);
            *slot = bit;
            pool ^= 1 << bit;
            code %= f;
        }
        order
    }
    /// Index of the k-th set bit in a bitmask.
    fn nth_set_bit(mask: u8, k: u8) -> u8 {
        (0..8u8).filter(|bit| mask & (1 << bit) != 0).nth(k as usize).unwrap()
    }

    pub(crate) fn factorial(n: usize) -> u8 {
        [1, 1, 2, 6, 24, 120][n]
    }
}

impl FromIterator<Card> for Perm {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Card>,
    {
        let mut buf = [Card::from(0u8); 5];
        let mut n = 0u8;
        for card in iter {
            buf[n as usize] = card;
            n += 1;
        }
        debug_assert!(n <= 5);
        Self(Self::encode(&buf, n))
    }
}

impl From<u8> for Perm {
    fn from(code: u8) -> Self {
        Self(code)
    }
}
impl From<Perm> for u8 {
    fn from(perm: Perm) -> Self {
        perm.0
    }
}

impl std::fmt::Display for Perm {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "σ{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_sorted() {
        let cards = Card::parse("2c 3d 4h").unwrap();
        assert_eq!(Perm::of(&cards), Perm::identity());
    }
    #[test]
    fn identity_arrange() {
        let cards = Card::parse("2c 5d Th").unwrap();
        let hand = Hand::from(cards.clone());
        assert_eq!(Perm::identity().arrange(hand).collect::<Vec<_>>(), cards);
    }
    #[test]
    fn round_trip_hole() {
        let cards = Card::parse("Kd 7h").unwrap();
        let hand = Hand::from(cards.clone());
        assert_eq!(Perm::of(&cards).arrange(hand).collect::<Vec<_>>(), cards);
    }
    #[test]
    fn round_trip_board() {
        let cards = Card::parse("Qs 2c Ah Td 5s").unwrap();
        let hand = Hand::from(cards.clone());
        assert_eq!(Perm::of(&cards).arrange(hand).collect::<Vec<_>>(), cards);
    }
    #[test]
    fn reverse_two() {
        let fwd = Card::parse("7h Kd").unwrap();
        let rev = Card::parse("Kd 7h").unwrap();
        assert_eq!(u8::from(Perm::of(&fwd)), 0);
        assert_eq!(u8::from(Perm::of(&rev)), 1);
    }
    #[test]
    fn reverse_max() {
        let cards = Card::parse("As Qs Th 5d 2c").unwrap();
        assert_eq!(u8::from(Perm::of(&cards)), 119);
    }
    /// all n! codes produce distinct arrangements that round-trip, for every valid size
    #[test]
    fn bijective_all_sizes() {
        let hands = [
            Hand::empty(),
            Hand::try_from("Ac").unwrap(),
            Hand::try_from("7h Kd").unwrap(),
            Hand::try_from("2c 5d Th").unwrap(),
            Hand::try_from("Js Qh 3c Ad").unwrap(),
            Hand::try_from("2c 5d Th Qs As").unwrap(),
        ];
        for (n, hand) in hands.iter().enumerate() {
            let f = Perm::factorial(n);
            let mut seen = std::collections::HashSet::new();
            for code in 0..f {
                let arranged = Perm::from(code).arrange(*hand).collect::<Vec<_>>();
                seen.insert(arranged.clone());
                assert_eq!(u8::from(Perm::of(&arranged)), code);
            }
            assert_eq!(seen.len(), f as usize);
        }
    }
    /// identity is always code 0, reverse is always code n!-1
    #[test]
    fn boundary_codes() {
        let hands = [
            Hand::empty(),
            Hand::try_from("Ac").unwrap(),
            Hand::try_from("7h Kd").unwrap(),
            Hand::try_from("2c 5d Th").unwrap(),
            Hand::try_from("Js Qh 3c Ad").unwrap(),
            Hand::try_from("2c 5d Th Qs As").unwrap(),
        ];
        for (n, hand) in hands.iter().enumerate() {
            let sorted = Vec::<Card>::from(*hand);
            let reversed = sorted.iter().rev().copied().collect::<Vec<_>>();
            assert_eq!(u8::from(Perm::of(&sorted)), 0);
            assert_eq!(u8::from(Perm::of(&reversed)), Perm::factorial(n) - 1);
        }
    }
    /// identity always reproduces Hand's canonical iteration order
    #[test]
    fn identity_is_canonical() {
        let hands = [
            Hand::empty(),
            Hand::try_from("Ac").unwrap(),
            Hand::try_from("Kd 7h").unwrap(),
            Hand::try_from("2c 5d Th Qs As").unwrap(),
        ];
        for hand in hands {
            let canonical = Vec::<Card>::from(hand);
            let arranged = Perm::identity().arrange(hand).collect::<Vec<_>>();
            assert_eq!(arranged, canonical);
        }
    }
    /// the set of cards is preserved regardless of permutation
    #[test]
    fn arrange_preserves_hand() {
        let hand = Hand::try_from("3c 7d Jh Ks").unwrap();
        for code in 0..24u8 {
            let arranged = Perm::from(code).arrange(hand);
            assert_eq!(arranged.collect::<Hand>(), hand);
        }
    }
    /// trivial sizes: empty and singleton always produce identity
    #[test]
    fn degenerate_sizes() {
        assert_eq!(Perm::of(&[]), Perm::identity());
        let card = Card::try_from("Ah").unwrap();
        assert_eq!(Perm::of(&[card]), Perm::identity());
        assert_eq!(Perm::from(0).arrange(Hand::from(card)).collect::<Vec<_>>(), vec![card]);
    }
    /// FromIterator<Card> agrees with Perm::of
    #[test]
    fn from_iterator() {
        let cards = Card::parse("Qs 2c Ah Td 5s").unwrap();
        let collected = cards.iter().copied().collect::<Perm>();
        assert_eq!(collected, Perm::of(&cards));
    }
}
