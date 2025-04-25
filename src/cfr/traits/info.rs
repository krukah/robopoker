/// the information bucket is fully abstracted. it must be implemented
/// by the consumer of this MCCFR API.
///
/// the implementation must be able to determine:
///  what possible Edges may emerge from this Node (Decision)
///
/// the generation of this information is the responsibility of the Encoder,
/// which has global tree context and may make probabilistic or path-dependent decisions
pub trait Info: Clone + Copy + PartialEq + Eq + std::hash::Hash + std::fmt::Debug {
    type E: super::edge::Edge;
    type T: super::turn::Turn;
    fn choices(&self) -> Vec<Self::E>;
}
