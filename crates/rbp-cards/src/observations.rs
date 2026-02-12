use super::hand::Hand;
use super::hands::HandIterator;
use super::observation::Observation;
use super::street::Street;

/// Exhaustive iterator over all observations for a street.
///
/// Composes two [`HandIterator`]s: one for hole cards (outer) and one for
/// board cards (inner). For each hole card combination, iterates through
/// all valid board combinations.
///
/// # Usage
///
/// Used during abstraction generation to enumerate all possible situations
/// on a given street, enabling histogram computation and clustering.
pub struct ObservationIterator {
    street: Street,
    pocket: Hand,
    outer: HandIterator,
    inner: HandIterator,
}

impl From<Street> for ObservationIterator {
    fn from(street: Street) -> Self {
        // weird handling of Street::Pref edge. could be coupled with
        // weird handling of HandIterator to be more elegant.
        // think i need Option<Hand> in HandIterator rather than store last.
        // need to make it work with Street::Pref (Hand::empty())
        // and it should compose well with a separate HandIterator, so
        // ObsIterator can reap the benefit

        // start with first card
        let pocket = Self::start();
        let inner = HandIterator::from((street.n_observed() - 2, pocket));
        let mut outer = HandIterator::from((2, Hand::empty()));
        match street {
            Street::Pref => None,
            _ => outer.next(),
        };
        Self {
            street,
            pocket,
            outer,
            inner,
        }
    }
}

impl Iterator for ObservationIterator {
    type Item = Observation;
    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            Some(next) => self.inner(next),
            None => match self.outer.next() {
                Some(next) => self.outer(next),
                None => None,
            },
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.combinations();
        (n, Some(n))
    }
}

impl ObservationIterator {
    /// Total number of observations for this street.
    pub fn combinations(&self) -> usize {
        self.outer.combinations() * self.inner.combinations()
    }
    /// The street being enumerated.
    pub fn street(&self) -> Street {
        self.street
    }
    fn start() -> Hand {
        // 2c 2d
        #[cfg(not(feature = "shortdeck"))]
        let pocket = Hand::from(0x3);
        // 6c 6d
        #[cfg(feature = "shortdeck")]
        let pocket = Hand::from(0x30000);
        pocket
    }
    fn inner(&mut self, public: Hand) -> Option<Observation> {
        Some(Observation::from((self.pocket, public)))
    }
    fn outer(&mut self, pocket: Hand) -> Option<Observation> {
        self.pocket = pocket;
        match self.street {
            Street::Pref => Some(Observation::from((self.pocket, Hand::empty()))),
            street => {
                self.inner = HandIterator::from((street.n_observed() - 2, self.pocket));
                self.inner
                    .next()
                    .map(|public| Observation::from((self.pocket, public)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn n_pref() {
        let street = Street::Pref;
        let iter = ObservationIterator::from(street);
        assert_eq!(iter.combinations(), street.n_observations());
        assert_eq!(iter.combinations(), iter.count());
    }
    #[test]
    #[ignore]
    fn n_flop() {
        let street = Street::Flop;
        let iter = ObservationIterator::from(street);
        assert_eq!(iter.combinations(), street.n_observations());
        assert_eq!(iter.combinations(), iter.count());
    }
    #[test]
    #[ignore]
    fn n_turn() {
        let street = Street::Turn;
        let iter = ObservationIterator::from(street);
        assert_eq!(iter.combinations(), street.n_observations());
        assert_eq!(iter.combinations(), iter.count());
    }
    #[test]
    #[ignore]
    fn n_rive() {
        let street = Street::Rive;
        let iter = ObservationIterator::from(street);
        assert_eq!(iter.combinations(), street.n_observations());
        assert_eq!(iter.combinations(), iter.count());
    }
}
