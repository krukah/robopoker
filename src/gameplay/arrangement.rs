use super::*;
use crate::cards::*;
use crate::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "client", derive(serde::Serialize, serde::Deserialize))]
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
    pub fn empty() -> Self {
        Self([None; 7])
    }

    pub fn vec(&self) -> Vec<Card> {
        self.0.iter().filter_map(|&c| c).collect()
    }

    pub fn len(&self) -> usize {
        self.0.iter().filter_map(|&c| c).count()
    }

    pub fn get(&self, index: usize) -> Option<Card> {
        self.0.get(index).and_then(|&c| c)
    }

    pub fn contains(&self, card: &Card) -> bool {
        self.0.iter().any(|&c| c == Some(*card))
    }

    pub fn iter(&self) -> impl Iterator<Item = Card> + '_ {
        self.0.iter().filter_map(|&c| c)
    }

    /// Uses the number of dealt cards to determine the street.
    pub fn street(&self) -> Street {
        Street::from(self.vec().len())
    }

    /// Converts itself into normal form via Isomorphism round trip
    pub fn normalize(&self) -> Self {
        Self::from(Observation::from(Isomorphism::from(Observation::from(
            self.clone(),
        ))))
    }

    pub fn draws(&self) -> impl Iterator<Item = Action> + '_ {
        Street::all()
            .into_iter()
            .skip(1)
            .take_while(|s| s.clone() <= self.street())
            .map(|s| self.revealed(s))
            .map(Hand::from)
            .map(Action::Draw)
    }

    /// Truncates or extends observed Cards to the desired street length, preserving order
    pub fn justify(&self, street: Street) -> Self {
        Self::from(
            self.vec()
                .into_iter()
                .chain(self.deck())
                .take(street.n_observed())
                .collect::<Vec<Card>>(),
        )
    }

    pub fn revealed(&self, street: Street) -> Vec<Card> {
        self.vec()
            .into_iter()
            .skip(street.n_observed() - street.n_revealed())
            .take(street.n_revealed())
            .collect()
    }

    pub fn public(&self) -> Vec<Card> {
        self.vec()
            .into_iter()
            .skip(Street::Pref.n_observed())
            .collect()
    }

    pub fn pocket(&self) -> Vec<Card> {
        self.vec()
            .into_iter()
            .take(Street::Pref.n_observed())
            .collect()
    }

    pub fn deck(&self) -> Deck {
        Deck::from(Hand::from(self.vec()).complement())
    }

    pub fn observation(&self) -> Observation {
        Observation::try_from(self.vec()).expect("valid observation from arrangement")
    }

    pub fn isomorphism(&self) -> Isomorphism {
        Isomorphism::from(self.observation())
    }

    pub fn permute(self) -> Self {
        Self::from(Permutation::random().permute(self.observation())).reorder()
    }

    pub fn reorder(self) -> Self {
        std::iter::empty()
            .chain(self.observation().pocket().shuffle())
            .chain(self.observation().public().shuffle())
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
            .chain(obs.pocket().clone())
            .chain(obs.public().clone())
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

impl crate::Arbitrary for Arrangement {
    fn random() -> Self {
        Self::from(Observation::random()).permute()
    }
}

impl std::fmt::Display for Arrangement {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let cards = self
            .vec()
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        write!(f, "{}", cards)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shuffle() {
        let random = Arrangement::random();
        assert!(random.observation() == random.reorder().observation());
    }

    #[test]
    fn permute() {
        let random = Arrangement::random();
        assert!(random.isomorphism() == random.permute().isomorphism());
    }

    #[test]
    // only testing idempotence for now
    fn justify() {
        let random = Arrangement::random();
        assert!(random == random.justify(random.street()));
    }
}
