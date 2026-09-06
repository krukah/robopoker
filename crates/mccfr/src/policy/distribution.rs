/// A distribution over edges — used for the current strategy, regret
/// increments, and the accumulated average strategy alike.
///
/// A `Vec`, not a map: action counts are small (2-10 in poker), so cache
/// locality beats O(1) lookup, and iteration dominates CFR anyway. RPS
/// benchmarks confirm Vec outperforms map-based versions.
pub type Policy<E> = Vec<(E, pokerkit::Probability)>;
