use crate::*;

/// Exhaustive iterator over every observation on a street: two composed
/// [`HandIterator`]s, hole cards outer and board cards inner. Drives
/// abstraction generation, where each situation needs a histogram.
pub struct ObservationIterator {
    street: Street,
    pocket: Hand,
    outer: HandIterator,
    inner: HandIterator,
}

impl From<Street> for ObservationIterator {
    fn from(street: Street) -> Self {
        // weird handling of Street::Pref edge. could be coupled with weird handling of
        // HandIterator to be more elegant.
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
        match self.inner_mut().next() {
            Some(public) => Some(Observation::from((self.pocket(), public))),
            None => self.outer_mut().next().and_then(|pocket| self.advance(pocket)),
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
        self.outer().combinations() * self.inner().combinations()
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

    fn pocket(&self) -> Hand {
        self.pocket
    }

    fn pocket_mut(&mut self) -> &mut Hand {
        &mut self.pocket
    }

    fn inner(&self) -> &HandIterator {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut HandIterator {
        &mut self.inner
    }

    fn outer(&self) -> &HandIterator {
        &self.outer
    }

    fn outer_mut(&mut self) -> &mut HandIterator {
        &mut self.outer
    }

    fn advance(&mut self, pocket: Hand) -> Option<Observation> {
        *self.pocket_mut() = pocket;
        match self.street() {
            Street::Pref => Some(Observation::from((pocket, Hand::empty()))),
            street => {
                *self.inner_mut() = HandIterator::from((street.n_observed() - 2, pocket));
                self.inner_mut()
                    .next()
                    .map(|public| Observation::from((pocket, public)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "slow: exhaustive iteration"]
    fn n_pref() {
        let street = Street::Pref;
        let iter = ObservationIterator::from(street);
        assert_eq!(iter.combinations(), street.n_observations());
        assert_eq!(iter.combinations(), iter.count());
    }
    #[test]
    #[ignore = "slow: exhaustive iteration"]
    fn n_flop() {
        let street = Street::Flop;
        let iter = ObservationIterator::from(street);
        assert_eq!(iter.combinations(), street.n_observations());
        assert_eq!(iter.combinations(), iter.count());
    }
    #[test]
    #[ignore = "slow: exhaustive iteration"]
    fn n_turn() {
        let street = Street::Turn;
        let iter = ObservationIterator::from(street);
        assert_eq!(iter.combinations(), street.n_observations());
        assert_eq!(iter.combinations(), iter.count());
    }
    #[test]
    #[ignore = "slow: exhaustive iteration"]
    fn n_rive() {
        let street = Street::Rive;
        let iter = ObservationIterator::from(street);
        assert_eq!(iter.combinations(), street.n_observations());
        assert_eq!(iter.combinations(), iter.count());
    }
}
