/// this is pre-implemented. it is a distrubtion
/// over policy space. i.e., it is a Density over Edges,
/// presumably at a given Info point.
pub type Policy<E> = Vec<(E, crate::Probability)>;
