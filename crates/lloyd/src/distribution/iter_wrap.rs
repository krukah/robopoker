/// Helper enum to unify different support iterators
pub(crate) enum IterWrap<A, B, C, D> {
    Pref(A),
    Flop(B),
    Turn(C),
    Rive(D),
}
