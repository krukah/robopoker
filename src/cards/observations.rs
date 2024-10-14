use super::hand::Hand;
use super::hands::HandIterator;
use super::observation::Observation;
use super::street::Street;

/// ObservationIterator is an iterator over all possible Observations for a given Street.
///
/// composing Iterators like this helps when we get to
/// lazily generating Isomorphisms from Observations, or
/// want to do other FP tricks or sharding.
pub struct ObservationIterator {
    last: Hand,
    pocket: HandIterator,
    public: HandIterator,
    street: Street,
}

impl From<Street> for ObservationIterator {
    fn from(street: Street) -> Self {
        Self {
            street,
            pocket: HandIterator::from((2, Hand::empty())),
            public: HandIterator::from((street.n_observed(), Hand::from(0b11))),
            last: Hand::from(0b11),
        }
    }
}

impl ObservationIterator {
    pub fn size(&self) -> usize {
        let outer = self.pocket.combinations();
        let inner = self.public.combinations();
        outer * inner
    }
    fn public(&mut self, next: Hand) -> Option<Observation> {
        Some(Observation::from((self.last, next)))
    }
    fn pocket(&mut self, next: Hand) -> Option<Observation> {
        self.last = next;
        match self.street {
            Street::Pref => Some(Observation::from((next, Hand::empty()))),
            street @ _ => {
                self.public = HandIterator::from((street.n_observed(), next));
                self.public
                    .next()
                    .map(|public| Observation::from((next, public)))
            }
        }
    }
}

impl Iterator for ObservationIterator {
    type Item = Observation;
    fn next(&mut self) -> Option<Self::Item> {
        match self.public.next() {
            Some(next) => self.public(next),
            None => match self.pocket.next() {
                Some(next) => self.pocket(next),
                None => None,
            },
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.size();
        (n, Some(n))
    }
}
