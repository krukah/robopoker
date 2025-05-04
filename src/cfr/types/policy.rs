/// this is pre-implemented. it is a distrubtion
/// over policy space. i.e., it is a Density over Edges,
/// presumably at a given Info point.
///
/// an alternative implementatino would use either
/// BTreeMap or HashMap, which yields better asymptotic
/// performance with O(1) lookups. but i feel like in
/// practice, iterating over the vector until we find
/// equality might be more efficient.
///
/// benchmarks from RPS (which has a fixed number 3 of
/// actions) show that this is more efficient than
/// the other two implementations. i'm going to leave this as a Vec for now because
/// it has the lowest overhead.

pub type Policy<E> = Vec<(E, crate::Probability)>;
// pub type Policy<E> = std::collections::HashMap<E, crate::Probability>;
// pub type Policy<E> = std::collections::BTreeMap<E, crate::Probability>;
