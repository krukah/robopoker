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
        }
    }
}

impl ObservationIterator {
    pub fn size(&self) -> usize {
        let outer = self.pocket.combinations();
        let inner = self.public.combinations();
        outer * inner
    }
    fn public(&mut self, public: Hand) -> Option<Observation> {
        let pocket = self.pocket.look();
        Some(Observation::from((pocket, public)))
    }
    fn pocket(&mut self, pocket: Hand) -> Option<Observation> {
        let n = self.street.n_observed();
        self.public = HandIterator::from((n, pocket));
        self.public
            .next()
            .map(|public| Observation::from((pocket, public)))
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
