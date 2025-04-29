/// this is pre-implemented. it is a distrubtion
/// over policy space. i.e., it is a Density over Edges,
/// presumably at a given Info point.
///
/// an alternative implementatino would use either
/// BTreeMap or HashMap, which yields better asymptotic
/// performance with O(1) lookups. but i feel like in
/// practice, iterating over the vector until we find
/// equality might be more efficient. vibes-based
/// analysis to be clear, not at all benchmarked yet.
pub type Policy<E> = Vec<(E, crate::Probability)>;
