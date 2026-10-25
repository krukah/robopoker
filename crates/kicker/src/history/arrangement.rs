use super::*;
use deuce::*;
use pokerkit::*;

/// An ordered sequence of up to 7 cards as dealt.
///
/// Unlike [`Hand`] which is a set, `Arrangement` preserves dealing order:
/// the first two cards are the hole cards, followed by flop (3), turn (1),
/// and river (1). This is essential for UI display and card animation.
///
/// Empty slots are `None`, enabling witness deals (e.g., preflop-only).
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct Arrangement([Option<Card>; 7]);

impl Default for Arrangement {
    fn default() -> Self {
        Self([
            Some(Card::try_from("2c").unwrap()),
            Some(Card::try_from("2d").unwrap()),
            None,
            None,
            None,
            None,
            None,
        ])
    }
}

impl Arrangement {
    /// Creates an arrangement with no cards.
    pub fn empty() -> Self {
        Self([None; 7])
    }
    /// Collects dealt cards into a vector.
    pub fn vec(&self) -> Vec<Card> {
        self.0.iter().filter_map(|&c| c).collect()
    }
    /// Number of dealt cards.
    pub fn len(&self) -> usize {
        self.0.iter().filter_map(|&c| c).count()
    }
    /// True iff no cards have been dealt.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Card at a specific index.
    pub fn get(&self, index: usize) -> Option<Card> {
        self.0.get(index).and_then(|&c| c)
    }
    /// True if the card is in this arrangement.
    pub fn contains(&self, card: &Card) -> bool {
        self.0.contains(&Some(*card))
    }
    /// Iterates over dealt cards.
    pub fn iter(&self) -> impl Iterator<Item = Card> + '_ {
        self.0.iter().filter_map(|&c| c)
    }
    /// Infers street from number of dealt cards.
    pub fn street(&self) -> Street {
        Street::from(self.vec().len())
    }
    /// Converts to canonical form (normalized suits and order).
    pub fn normalize(&self) -> Self {
        Self::from(Observation::from(Isomorphism::from(Observation::from(*self))))
    }

    pub fn draws(&self) -> impl Iterator<Item = Action> + '_ {
        Street::all()
            .into_iter()
            .skip(1)
            .take_while(|s| *s <= self.street())
            .map(|s| self.revealed(s))
            .map(Hand::from)
            .map(Action::Draw)
    }
    /// Extends or truncates to the specified street.
    pub fn justify(&self, street: Street) -> Self {
        Self::from(
            self.vec()
                .into_iter()
                .chain(self.deck())
                .take(street.n_observed())
                .collect::<Vec<Card>>(),
        )
    }
    /// Cards revealed on a specific street.
    pub fn revealed(&self, street: Street) -> Vec<Card> {
        self.vec()
            .into_iter()
            .skip(street.n_observed() - street.n_revealed())
            .take(street.n_revealed())
            .collect()
    }
    /// Hole, flop, turn, river as sets; a street not yet dealt is empty.
    pub fn streets(&self) -> [Hand; 4] {
        Street::all().map(|street| Hand::from(self.revealed(street)))
    }
    /// Community cards (flop + turn + river).
    pub fn public(&self) -> Vec<Card> {
        self.vec().into_iter().skip(Street::Pref.n_observed()).collect()
    }
    /// Hole cards (first two).
    pub fn pocket(&self) -> Vec<Card> {
        self.vec().into_iter().take(Street::Pref.n_observed()).collect()
    }
    /// Remaining deck (cards not in arrangement).
    pub fn deck(&self) -> Deck {
        Deck::from(Hand::from(self.vec()).complement())
    }
    /// Converts to an observation (set-based, order-independent).
    pub fn observation(&self) -> Observation {
        Observation::try_from(self.vec()).expect("valid observation from arrangement")
    }
    /// Converts to canonical isomorphism.
    pub fn isomorphism(&self) -> Isomorphism {
        Isomorphism::from(self.observation())
    }
    /// Applies a random suit permutation and shuffles within positions.
    pub fn permute(self) -> Self {
        self.permute_by(&Permutation::random()).shuffle()
    }
    /// A representative of this isomorphism class drawn entirely from `deck` — the same hand,
    /// relabelled onto suits the table has not spoken for.
    ///
    /// The database answers with a canonical observation, and the suit labels on it are
    /// arbitrary: every one of the 24 relabellings is the same strategic hand.
    pub fn permute_within(self, deck: Deck) -> Option<Self> {
        let seats = Permutation::exhaust()
            .into_iter()
            .map(|perm| self.permute_by(&perm))
            .filter(|arr| arr.iter().all(|card| deck.contains(&card)))
            .collect::<Vec<Self>>();
        seats
            .get(rand::random_range(0..seats.len().max(1)))
            .copied()
            .map(Self::shuffle)
    }

    /// Applies a specific suit permutation, preserving order.
    pub fn permute_by(&self, perm: &Permutation) -> Self {
        Self(self.0.map(|opt| {
            opt.map(|c| (c.rank(), c.suit()))
                .map(|(r, s)| Card::from((r, perm.map(&s))))
        }))
    }

    fn shuffle(self) -> Self {
        std::iter::empty()
            .chain(self.observation().pocket().shuffle())
            .chain(self.observation().public().shuffle())
            .collect::<Vec<Card>>()
            .into()
    }
    /// Applies suit canonicalization and re-sorts to maintain canonical order.
    pub fn normalize_suits(&self) -> Self {
        self.permute_by(&Permutation::from(&self.observation()))
    }
    /// [`Self::normalize_suits`], then each street sorted: suit names and order within a street
    /// carry no information, street order does. Suits interchangeable in the observation but
    /// split across streets can land two ways.
    pub fn canonical(&self) -> Self {
        let suited = self.normalize_suits();
        Street::all()
            .into_iter()
            .take_while(|street| *street <= suited.street())
            .flat_map(|street| Hand::from(suited.revealed(street)))
            .collect::<Vec<Card>>()
            .into()
    }
    /// Applies only sorting canonicalization, preserving suits.
    pub fn normalize_sorts(&self) -> Self {
        Self::from(self.observation())
    }
}

impl Game {
    /// The acting seat's hole cards, then the board as dealt.
    pub fn arrangement(&self) -> Arrangement {
        Vec::<Card>::from(Hand::from(self.actor().cards()))
            .into_iter()
            .chain(self.board().cards())
            .collect::<Vec<Card>>()
            .into()
    }
}

impl From<Arrangement> for Hand {
    fn from(cards: Arrangement) -> Hand {
        Hand::from(cards.vec())
    }
}

impl From<Arrangement> for Vec<Card> {
    fn from(history: Arrangement) -> Self {
        history.vec()
    }
}

impl From<Arrangement> for Observation {
    fn from(history: Arrangement) -> Self {
        Observation::try_from(history.vec()).expect("convert CardHistory -> Observation")
    }
}

impl From<Observation> for Arrangement {
    fn from(obs: Observation) -> Self {
        std::iter::empty()
            .chain(*obs.pocket())
            .chain(*obs.public())
            .collect::<Vec<Card>>()
            .into()
    }
}

impl From<Vec<Card>> for Arrangement {
    fn from(cards: Vec<Card>) -> Self {
        let mut arr = [None; 7];
        cards
            .into_iter()
            .take(7)
            .enumerate()
            .for_each(|(i, card)| arr[i] = Some(card));
        Self(arr)
    }
}

impl From<Street> for Arrangement {
    fn from(street: Street) -> Self {
        Self::from(Observation::from(street)).permute()
    }
}

impl pokerkit::Arbitrary for Arrangement {
    fn random() -> Self {
        Self::from(Observation::random()).permute()
    }
}

impl std::fmt::Display for Arrangement {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let cards = self.vec().iter().map(ToString::to_string).collect::<Vec<_>>().join(" ");
        write!(f, "{cards}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shuffle() {
        let random = Arrangement::random();
        assert_eq!(random.observation(), random.shuffle().observation());
    }

    #[test]
    fn permute() {
        let random = Arrangement::random();
        assert_eq!(random.isomorphism(), random.permute().isomorphism());
    }

    /// The same hand, on suits the caller still has.
    #[test]
    fn permute_within_the_deck() {
        let arr = Arrangement::random();
        let deck = arr.deck();
        let seated = Arrangement::random().permute_within(deck).expect("a free relabelling");
        assert!(seated.iter().all(|card| deck.contains(&card)), "dealt a card the deck did not have");
    }

    /// A class nobody can be dealt is a real answer, not a panic.
    #[test]
    fn permute_within_nothing() {
        assert_eq!(Arrangement::random().permute_within(Deck::from(Hand::empty())), None);
    }

    #[test]
    fn canonical_matches_isomorphism() {
        (0..1 << 10).map(|_| Arrangement::random()).for_each(|arr| {
            assert_eq!(arr.canonical().observation(), Observation::from(arr.isomorphism()));
            assert_eq!(arr.canonical().canonical(), arr.canonical());
        });
    }

    #[test]
    fn canonical_keeps_street_order() {
        let flop = Arrangement::from(Card::parse("Ac Ks Td Jh Qh 2c").unwrap());
        let turn = Arrangement::from(Card::parse("Ac Ks Td Jh 2c Qh").unwrap());
        assert_ne!(flop.canonical(), turn.canonical());
    }

    #[test]
    fn canonical_ignores_order_within_a_street() {
        let a = Arrangement::from(Card::parse("Ac Ks Td Jh Qh 2c").unwrap());
        let b = Arrangement::from(Card::parse("Ks Ac Qh Td Jh 2c").unwrap());
        assert_eq!(a.canonical(), b.canonical());
    }

    #[test]
    fn canonical_ignores_suit_names() {
        let arr = Arrangement::from(Card::parse("Ac Ks Td Jh Qh 2c 7d").unwrap());
        Permutation::exhaust()
            .iter()
            .for_each(|perm| assert_eq!(arr.permute_by(perm).canonical(), arr.canonical(), "{perm}"));
    }

    #[test]
    fn justify() {
        let random = Arrangement::random();
        assert_eq!(random, random.justify(random.street()));
    }

    #[test]
    fn normalize_sorts_preserves_observation() {
        let arr = Arrangement::random();
        assert_eq!(arr.observation(), arr.normalize_sorts().observation());
    }

    #[test]
    fn normalize_sorts_idempotent() {
        let arr = Arrangement::random();
        assert_eq!(arr.normalize_sorts(), arr.normalize_sorts().normalize_sorts());
    }

    #[test]
    fn normalize_suits_preserves_isomorphism() {
        let arr = Arrangement::random();
        assert_eq!(arr.isomorphism(), arr.normalize_suits().isomorphism());
    }

    #[test]
    fn normalize_suits_idempotent() {
        let arr = Arrangement::random();
        assert_eq!(arr.normalize_suits(), arr.normalize_suits().normalize_suits());
    }

    #[test]
    fn normalize_suits_preserves_ranks() {
        let arr = Arrangement::random();
        let norm = arr.normalize_suits();
        let mut arr_ranks: Vec<_> = arr.iter().map(|c| c.rank()).collect();
        let mut norm_ranks: Vec<_> = norm.iter().map(|c| c.rank()).collect();
        arr_ranks.sort();
        norm_ranks.sort();
        assert_eq!(arr_ranks, norm_ranks);
    }

    #[test]
    fn normalize_sorts_preserves_isomorphism() {
        let arr = Arrangement::random();
        assert_eq!(arr.isomorphism(), arr.normalize_sorts().isomorphism());
    }

    #[test]
    fn same_observation_same_sorts_normalization() {
        let arr = Arrangement::random();
        let reordered = arr.shuffle();
        assert_eq!(arr.observation(), reordered.observation()); // precondition
        assert_eq!(arr.normalize_sorts(), reordered.normalize_sorts());
    }

    #[test]
    fn same_isomorphism_same_suits_observation() {
        let arr = Arrangement::random();
        let permuted = arr.permute();
        assert_eq!(arr.isomorphism(), permuted.isomorphism()); // precondition
        assert_eq!(arr.normalize_suits().observation(), permuted.normalize_suits().observation());
    }

    #[test]
    fn normalize_equals_composition() {
        let arr = Arrangement::random();
        assert_eq!(arr.normalize(), arr.normalize_sorts().normalize_suits().normalize_sorts());
    }
    /// Arrangement from Observation is deterministic: same Observation
    /// always produces the same per-street card assignment.
    #[test]
    fn from_observation_deterministic() {
        let obs = Observation::from(Street::Rive);
        assert_eq!(Arrangement::from(obs), Arrangement::from(obs));
    }
    /// Round-tripping Observation → Arrangement → Observation preserves
    /// the card set even though within-street order may change.
    #[test]
    fn observation_roundtrip_preserves_cards() {
        let arr = Arrangement::random();
        let roundtrip = Arrangement::from(arr.observation());
        assert_eq!(arr.observation(), roundtrip.observation());
    }
    /// Arrangement from Observation assigns cards to correct street slots:
    /// first 2 = pocket, next 3 = flop, next 1 = turn, next 1 = river.
    #[test]
    fn from_observation_street_assignment() {
        let obs = Observation::from(Street::Rive);
        let arr = Arrangement::from(obs);
        assert_eq!(arr.pocket().len(), 2);
        assert_eq!(arr.revealed(Street::Flop).len(), 3);
        assert_eq!(arr.revealed(Street::Turn).len(), 1);
        assert_eq!(arr.revealed(Street::Rive).len(), 1);
        let all = Hand::from(arr.pocket())
            + Hand::from(arr.revealed(Street::Flop))
            + Hand::from(arr.revealed(Street::Turn))
            + Hand::from(arr.revealed(Street::Rive));
        assert_eq!(all, Hand::from(arr));
    }
}
