use super::isomorphism::Isomorphism;
use super::observations::ObservationIterator;
use super::street::Street;

pub struct IsomorphismIterator(ObservationIterator);

impl Iterator for IsomorphismIterator {
    type Item = Isomorphism;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(observation) = self.0.next() {
            if Isomorphism::is_canonical(&observation) {
                return Some(Isomorphism::from(observation));
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.0.street().n_isomorphisms();
        (n, Some(n))
    }
}

impl From<Street> for IsomorphismIterator {
    fn from(street: Street) -> Self {
        Self(ObservationIterator::from(street))
    }
}
