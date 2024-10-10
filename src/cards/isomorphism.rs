use super::observation::Observation;
use super::permutation::Permutation;
use super::street::Street;

/// many Observations are strategically equivalent,
/// so we can canonize to reduce the index space of
/// learned Abstractions.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, PartialOrd, Ord)]
pub struct Isomorphism(Observation);

impl Isomorphism {
    pub fn is_canonical(observation: &Observation) -> bool {
        Permutation::identity() == Permutation::from(observation)
    }
    pub fn enumerate(street: Street) -> Vec<Observation> {
        Observation::enumerate(street)
            .into_iter()
            .filter(|o| Self::is_canonical(o))
            .collect()
    }
}

impl From<Observation> for Isomorphism {
    fn from(ref observation: Observation) -> Self {
        let canonical = Permutation::from(observation);
        let secret = canonical.permute(observation.secret());
        let public = canonical.permute(observation.public());
        Self(Observation::from((secret, public)))
    }
}

impl std::fmt::Display for Isomorphism {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "o")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::card::Card;
    use crate::cards::hand::Hand;
    use crate::cards::rank::Rank;
    use crate::cards::suit::Suit;

    /// TODO
    /// implement from &str or From<IntoStr> or something
    /// to make testing easier to work with
    fn into(hand: Vec<(Rank, Suit)>) -> Observation {
        assert!(hand.len() == 5);
        let secret = Hand::from(
            hand.iter()
                .take(2)
                .map(|(r, s)| Card::from((*r, *s)))
                .collect::<Vec<Card>>(),
        );
        let public = Hand::from(
            hand.iter()
                .skip(2)
                .map(|(r, s)| Card::from((*r, *s)))
                .collect::<Vec<Card>>(),
        );
        Observation::from((secret, public))
    }

    #[test]
    fn test_isomorphic_observations() {
        let iso1 = Isomorphism::from(into(vec![
            (Rank::Ace, Suit::C),
            (Rank::King, Suit::D),
            (Rank::Queen, Suit::H),
            (Rank::Jack, Suit::S),
            (Rank::Nine, Suit::S),
        ]));
        let iso2 = Isomorphism::from(into(vec![
            (Rank::Ace, Suit::H),
            (Rank::King, Suit::C),
            (Rank::Queen, Suit::S),
            (Rank::Jack, Suit::D),
            (Rank::Nine, Suit::D),
        ]));
        assert!(iso1 == iso2, "{} != {}", iso1, iso2);
    }
}
