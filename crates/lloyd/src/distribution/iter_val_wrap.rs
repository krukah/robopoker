use pokerkit::*;

/// Helper enum to unify different value iterators
pub(crate) enum IterValWrap<A, B, C, D> {
    Pref(A),
    Flop(B),
    Turn(C),
    Rive(D),
}

impl<A, B, C, D> Iterator for IterValWrap<A, B, C, D>
where
    A: Iterator<Item = Entropy>,
    B: Iterator<Item = Entropy>,
    C: Iterator<Item = Entropy>,
    D: Iterator<Item = Entropy>,
{
    type Item = Entropy;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IterValWrap::Pref(i) => i.next(),
            IterValWrap::Flop(i) => i.next(),
            IterValWrap::Turn(i) => i.next(),
            IterValWrap::Rive(i) => i.next(),
        }
    }
}
