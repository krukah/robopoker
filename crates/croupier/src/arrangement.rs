use super::*;
use fulcrum::*;
use kicker::*;

/// An ordered sequence of up to 7 cards as dealt.
///
/// Unlike [`Hand`] which is a set, `Arrangement` preserves dealing order:
/// the first two cards are the hole cards, followed by flop (3), turn (1),
/// and river (1). This is essential for UI display and card animation.
///
/// # Structure
///
/// - Indices 0–1: Hole cards
/// - Indices 2–4: Flop
/// - Index 5: Turn
/// - Index 6: River
///
/// Empty slots are `None`, enabling witness deals (e.g., preflop-only).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
    /// Applies only sorting canonicalization, preserving suits.
    pub fn normalize_sorts(&self) -> Self {
        Self::from(self.observation())
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

impl fulcrum::Arbitrary for Arrangement {
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
        assert!(random.observation() == random.shuffle().observation());
    }

    #[test]
    fn permute() {
        let random = Arrangement::random();
        assert!(random.isomorphism() == random.permute().isomorphism());
    }

    #[test]
    fn justify() {
        let random = Arrangement::random();
        assert!(random == random.justify(random.street()));
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
