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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn n_pref() {
        let pref = Street::Pref;
        let iter = IsomorphismIterator::from(pref);
        assert_eq!(iter.count(), pref.n_isomorphisms());
    }

    #[test]
    #[ignore]
    fn n_flop() {
        let flop = Street::Flop;
        let iter = IsomorphismIterator::from(flop);
        assert_eq!(iter.count(), flop.n_isomorphisms());
    }

    #[test]
    #[ignore]
    fn n_turn() {
        let turn = Street::Turn;
        let iter = IsomorphismIterator::from(turn);
        assert_eq!(iter.count(), turn.n_isomorphisms());
    }

    #[test]
    #[ignore]
    fn n_rive() {
        let rive = Street::Rive;
        let iter = IsomorphismIterator::from(rive);
        assert_eq!(iter.count(), rive.n_isomorphisms());
    }
}
